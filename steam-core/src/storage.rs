use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior};

use crate::error::{Result, SteamError};

// ! One written by a newer build is refused, never replaced by `UPGRADES`, because it may hold Steam Guard secrets that exist nowhere else.
pub const SCHEMA_VERSION: i64 = 4;

const SCHEMA: &str = "
CREATE TABLE setting (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT;

CREATE TABLE folder (
    id       TEXT PRIMARY KEY,
    position INTEGER NOT NULL,
    name     TEXT NOT NULL,
    color    TEXT,
    icon     TEXT
) STRICT;

CREATE TABLE account_folder (
    steam_id  TEXT PRIMARY KEY,
    folder_id TEXT NOT NULL REFERENCES folder(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE account_color (
    steam_id TEXT PRIMARY KEY,
    color    TEXT NOT NULL
) STRICT;

CREATE TABLE installation (
    root   TEXT PRIMARY KEY,
    hidden INTEGER NOT NULL DEFAULT 0,
    custom INTEGER NOT NULL DEFAULT 0
) STRICT;

CREATE TABLE retained_account (
    steam_id     TEXT PRIMARY KEY,
    account_name TEXT NOT NULL,
    persona_name TEXT NOT NULL,
    avatar_path  TEXT,
    timestamp    INTEGER
) STRICT;

CREATE TABLE rank_choice (
    steam_id     TEXT PRIMARY KEY,
    account_name TEXT NOT NULL,
    persona_name TEXT NOT NULL,
    template     TEXT NOT NULL,
    rank         TEXT NOT NULL
) STRICT;

-- The whole vault payload: encrypted as one blob, or plain JSON when no passphrase is set.
CREATE TABLE vault (
    id   INTEGER PRIMARY KEY CHECK (id = 1),
    body TEXT NOT NULL
) STRICT;

CREATE TABLE playtime_source (
    installation TEXT PRIMARY KEY,
    device       INTEGER NOT NULL,
    inode        INTEGER NOT NULL,
    modified     REAL NOT NULL,
    offset       INTEGER NOT NULL,
    head         BLOB
) STRICT;

CREATE TABLE playtime_meta (
    installation       TEXT PRIMARY KEY,
    tracking_started_at REAL NOT NULL,
    history_started_at  REAL NOT NULL
) STRICT;

CREATE TABLE playtime_account (
    installation  TEXT NOT NULL,
    steam_id      TEXT NOT NULL,
    account_name  TEXT NOT NULL,
    persona_name  TEXT NOT NULL,
    total_seconds REAL NOT NULL,
    PRIMARY KEY (installation, steam_id)
) STRICT;

CREATE TABLE playtime_game (
    installation   TEXT NOT NULL,
    account_id     TEXT NOT NULL,
    app_id         TEXT NOT NULL,
    name           TEXT NOT NULL,
    total_seconds  REAL NOT NULL,
    sessions       INTEGER NOT NULL,
    last_played_at REAL,
    PRIMARY KEY (installation, account_id, app_id)
) STRICT;

-- Finished and running sessions share one table; `active` tells them apart and `id` keeps the order they were recorded in.
CREATE TABLE playtime_session (
    id                   INTEGER PRIMARY KEY AUTOINCREMENT,
    installation         TEXT NOT NULL,
    active               INTEGER NOT NULL,
    slot                 TEXT,
    session_id           TEXT NOT NULL,
    account_id           TEXT NOT NULL,
    account_name         TEXT NOT NULL,
    persona_name         TEXT NOT NULL,
    app_id               TEXT NOT NULL,
    game_name            TEXT NOT NULL,
    root_pid             INTEGER,
    process_create_time  REAL,
    launch_timestamp     REAL NOT NULL,
    detected_at          REAL NOT NULL,
    started_at           REAL NOT NULL,
    tracked_started_at   REAL,
    last_seen_at         REAL,
    ended_at             REAL,
    duration_seconds     REAL,
    tracking_mode        TEXT NOT NULL,
    end_reason           TEXT NOT NULL,
    exit_code            INTEGER,
    launch_command       TEXT NOT NULL
) STRICT;

CREATE INDEX playtime_session_installation ON playtime_session (installation, active);
CREATE INDEX playtime_session_account ON playtime_session (installation, account_id, started_at);

-- Write-ahead record for a link/finalize/transfer step, kept until the resulting account is durably saved in `vault`. `accepted` only turns on after Steam has confirmed the step; `link_wal::recover` uses it to tell an aborted attempt (unsafe to recover: Steam's answer is unknown) from one Steam already accepted (recoverable from this row alone).
CREATE TABLE link_wal (
    steam_id  TEXT PRIMARY KEY,
    encrypted INTEGER NOT NULL,
    nonce     TEXT,
    body      TEXT NOT NULL,
    accepted  INTEGER NOT NULL
) STRICT;
";

// ! `UPGRADES[n - 1]` takes a database from version `n` to `n + 1`. Step 1 marks the point `secure_delete` turned on, which `connect` follows with one `VACUUM` to purge whatever plaintext an earlier write may have left in a freed page. Each step uses `IF NOT EXISTS` so it stays re-runnable after an interrupted upgrade.
const UPGRADES: [&str; 3] = [
    "
CREATE TABLE IF NOT EXISTS link_wal (
    steam_id  TEXT PRIMARY KEY,
    encrypted INTEGER NOT NULL,
    nonce     TEXT,
    body      TEXT NOT NULL,
    accepted  INTEGER NOT NULL
) STRICT;
",
    "
UPDATE setting SET value = '\"NEVER\"' WHERE key = 'update_interval' AND value = '\"never\"';
UPDATE setting SET value = '\"STARTUP\"' WHERE key = 'update_interval' AND value = '\"startup\"';
UPDATE setting SET value = '\"FIFTEEN_MINUTES\"' WHERE key = 'update_interval' AND value = '\"15m\"';
UPDATE setting SET value = '\"THIRTY_MINUTES\"' WHERE key = 'update_interval' AND value = '\"30m\"';
UPDATE setting SET value = '\"ONE_HOUR\"' WHERE key = 'update_interval' AND value = '\"1h\"';
UPDATE setting SET value = '\"SIX_HOURS\"' WHERE key = 'update_interval' AND value = '\"6h\"';
UPDATE setting SET value = '\"ONE_DAY\"' WHERE key = 'update_interval' AND value = '\"1d\"';
UPDATE setting SET value = '\"ONE_WEEK\"' WHERE key = 'update_interval' AND value = '\"1w\"';
UPDATE setting SET value = '\"DISPLAY_NAME\"' WHERE key = 'search_scope' AND value = '\"display_name\"';
UPDATE setting SET value = '\"IDENTIFIER\"' WHERE key = 'search_scope' AND value = '\"identifier\"';
UPDATE setting SET value = '\"BOTH\"' WHERE key = 'search_scope' AND value = '\"both\"';
UPDATE setting SET value = '\"STARTUP\"' WHERE key = 'vault_lock_interval' AND value = '\"startup\"';
UPDATE setting SET value = '\"FIVE_MINUTES\"' WHERE key = 'vault_lock_interval' AND value = '\"5m\"';
UPDATE setting SET value = '\"TEN_MINUTES\"' WHERE key = 'vault_lock_interval' AND value = '\"10m\"';
UPDATE setting SET value = '\"FIFTEEN_MINUTES\"' WHERE key = 'vault_lock_interval' AND value = '\"15m\"';
UPDATE setting SET value = '\"TWENTY_MINUTES\"' WHERE key = 'vault_lock_interval' AND value = '\"20m\"';
UPDATE setting SET value = '\"THIRTY_MINUTES\"' WHERE key = 'vault_lock_interval' AND value = '\"30m\"';
UPDATE setting SET value = '\"ONE_HOUR\"' WHERE key = 'vault_lock_interval' AND value = '\"1h\"';
UPDATE setting SET value = '\"TWO_HOURS\"' WHERE key = 'vault_lock_interval' AND value = '\"2h\"';
UPDATE setting SET value = '\"FOUR_HOURS\"' WHERE key = 'vault_lock_interval' AND value = '\"4h\"';
",
    "
CREATE TABLE IF NOT EXISTS playtime_session (
    id                   INTEGER PRIMARY KEY AUTOINCREMENT,
    installation         TEXT NOT NULL,
    active               INTEGER NOT NULL,
    slot                 TEXT,
    session_id           TEXT NOT NULL,
    account_id           TEXT NOT NULL,
    account_name         TEXT NOT NULL,
    persona_name         TEXT NOT NULL,
    app_id               TEXT NOT NULL,
    game_name            TEXT NOT NULL,
    root_pid             INTEGER,
    process_create_time  REAL,
    launch_timestamp     REAL NOT NULL,
    detected_at          REAL NOT NULL,
    started_at           REAL NOT NULL,
    tracked_started_at   REAL,
    last_seen_at         REAL,
    ended_at             REAL,
    duration_seconds     REAL,
    tracking_mode        TEXT NOT NULL,
    end_reason           TEXT NOT NULL,
    exit_code            INTEGER,
    launch_command       TEXT NOT NULL
) STRICT;
CREATE INDEX IF NOT EXISTS playtime_session_installation ON playtime_session (installation, active);
CREATE INDEX IF NOT EXISTS playtime_session_account ON playtime_session (installation, account_id, started_at);
UPDATE playtime_session SET tracking_mode = 'PARTIAL' WHERE tracking_mode = 'partial';
UPDATE playtime_session SET tracking_mode = 'FULL' WHERE tracking_mode = 'full';
UPDATE playtime_session SET end_reason = 'REPLACED_BY_NEW_LAUNCH' WHERE end_reason = 'replaced_by_new_launch';
UPDATE playtime_session SET end_reason = 'ROOT_PROCESS_EXIT_LOG' WHERE end_reason = 'root_process_exit_log';
UPDATE playtime_session SET end_reason = 'PROCESS_MISSING_AFTER_RESTART' WHERE end_reason = 'process_missing_after_restart';
UPDATE playtime_session SET end_reason = 'PROCESS_EXIT' WHERE end_reason = 'process_exit';
UPDATE playtime_session SET end_reason = 'TRACKING_DISABLED' WHERE end_reason = 'tracking_disabled';
UPDATE playtime_session SET end_reason = 'STEAM_RUNNING_LIST' WHERE end_reason = 'steam_running_list';
",
];

// ! Cloning shares the same connection.
#[derive(Clone)]
pub struct Database {
    connection: Arc<Mutex<Connection>>,
    path: PathBuf,
}

impl std::fmt::Debug for Database {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Database")
            .field("path", &self.path)
            .finish()
    }
}

// ! `moved_aside` is set when a corrupt file was found and kept next to the fresh database under a name that is never reused.
pub struct Opened {
    pub database: Database,
    pub moved_aside: Option<PathBuf>,
}

// ! Only corruption justifies moving the file aside: a locked file, a permission error or a schema from another version must leave the data (which may hold Steam Guard secrets) exactly where it is.
#[derive(Debug)]
enum OpenFailure {
    // ! Not a SQLite file, or one that fails its integrity check.
    Corrupt(String),
    Version(i64),
    // ! Locked, unreadable, out of disk space, and so on.
    Other(rusqlite::Error),
}

impl OpenFailure {
    fn classify(error: rusqlite::Error) -> Self {
        match &error {
            rusqlite::Error::SqliteFailure(failure, _)
                if matches!(
                    failure.code,
                    rusqlite::ErrorCode::DatabaseCorrupt | rusqlite::ErrorCode::NotADatabase
                ) =>
            {
                OpenFailure::Corrupt(error.to_string())
            }
            _ => OpenFailure::Other(error),
        }
    }

    fn into_error(self, context: &'static str) -> SteamError {
        match self {
            OpenFailure::Version(found) => {
                SteamError::schema_version(context, found, SCHEMA_VERSION)
            }
            OpenFailure::Corrupt(message) => SteamError::io(context, io::Error::other(message)),
            OpenFailure::Other(error) => SteamError::io(context, error_as_io(&error)),
        }
    }
}

impl Database {
    pub fn open(path: &Path) -> Result<Opened> {
        if let Some(parent) = path.parent() {
            crate::util::create_private_dir_all(parent)
                .map_err(|error| SteamError::io("Create the application data folder", error))?;
        }
        match Self::connect(path) {
            Ok(connection) => Ok(Opened {
                database: Self::wrap(connection, path),
                moved_aside: None,
            }),
            Err(OpenFailure::Corrupt(_)) if path.exists() => {
                let backup = unused_backup_path(path);
                move_aside(path, &backup)?;
                let connection = Self::connect(path)
                    .map_err(|error| error.into_error("Create the application database"))?;
                Ok(Opened {
                    database: Self::wrap(connection, path),
                    moved_aside: Some(backup),
                })
            }
            Err(error) => Err(error.into_error("Open the application database")),
        }
    }

    pub fn transaction<T>(
        &self,
        context: &'static str,
        work: impl FnOnce(&Transaction<'_>) -> rusqlite::Result<T>,
    ) -> Result<T> {
        let mut connection = self.lock();
        // ! Taking the write lock up front avoids a failed upgrade when a save reads before it writes.
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|error| SteamError::io(context, error_as_io(&error)))?;
        let value =
            work(&transaction).map_err(|error| SteamError::io(context, error_as_io(&error)))?;
        transaction
            .commit()
            .map_err(|error| SteamError::io(context, error_as_io(&error)))?;
        Ok(value)
    }

    pub fn read<T>(
        &self,
        context: &'static str,
        work: impl FnOnce(&Connection) -> rusqlite::Result<T>,
    ) -> Result<T> {
        let connection = self.lock();
        work(&connection).map_err(|error| SteamError::io(context, error_as_io(&error)))
    }

    fn lock(&self) -> MutexGuard<'_, Connection> {
        self.connection
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn wrap(connection: Connection, path: impl Into<PathBuf>) -> Self {
        Self {
            connection: Arc::new(Mutex::new(connection)),
            path: path.into(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn connect(path: &Path) -> std::result::Result<Connection, OpenFailure> {
        let connection = Connection::open(path).map_err(OpenFailure::classify)?;
        restrict_permissions(path);
        let connection = Self::prepare(connection)?;
        if vacuum_pending(&connection).map_err(OpenFailure::classify)? {
            // ! `VACUUM` cannot run inside a transaction, so this waits until any upgrade step above has committed; a failure here (e.g. low disk space) leaves the flag set rather than refusing the start.
            if connection.execute_batch("VACUUM").is_ok() {
                clear_vacuum_pending(&connection).map_err(OpenFailure::classify)?;
            }
        }
        Ok(connection)
    }

    // ! Crossing the `secure_delete` boundary (version 2) sets a persisted flag so `connect`'s `VACUUM` rewrites the pages it left behind.
    fn prepare(mut connection: Connection) -> std::result::Result<Connection, OpenFailure> {
        connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(OpenFailure::classify)?;
        connection
            .busy_timeout(std::time::Duration::from_secs(5))
            .map_err(OpenFailure::classify)?;
        // ! Every write and delete from here on zeroes its freed pages instead of leaving the previous plaintext in the file.
        connection
            .pragma_update(None, "secure_delete", true)
            .map_err(OpenFailure::classify)?;
        // ! The integrity check runs first: a damaged file must be recognised as such rather than reported as a version or query error.
        let check: String = connection
            .query_row("PRAGMA quick_check", [], |row| row.get(0))
            .map_err(OpenFailure::classify)?;
        if check != "ok" {
            return Err(OpenFailure::Corrupt(format!("quick_check: {check}")));
        }
        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(OpenFailure::classify)?;
        if version == 0 {
            connection
                .execute_batch(SCHEMA)
                .map_err(OpenFailure::classify)?;
            connection
                .pragma_update(None, "user_version", SCHEMA_VERSION)
                .map_err(OpenFailure::classify)?;
        } else if version < SCHEMA_VERSION {
            let crossed_secure_delete = version < 2;
            Self::upgrade(&mut connection, version)?;
            if crossed_secure_delete {
                set_vacuum_pending(&connection).map_err(OpenFailure::classify)?;
            }
        } else if version != SCHEMA_VERSION {
            return Err(OpenFailure::Version(version));
        }
        Ok(connection)
    }

    // ! Each step and its version bump commit together, so a crash midway leaves the file at the version it reports.
    fn upgrade(connection: &mut Connection, from: i64) -> std::result::Result<(), OpenFailure> {
        for (index, step) in UPGRADES.iter().enumerate() {
            let target = index as i64 + 2;
            if target <= from {
                continue;
            }
            let transaction = connection
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(OpenFailure::classify)?;
            transaction
                .execute_batch(step)
                .map_err(OpenFailure::classify)?;
            transaction
                .pragma_update(None, "user_version", target)
                .map_err(OpenFailure::classify)?;
            transaction.commit().map_err(OpenFailure::classify)?;
        }
        Ok(())
    }

    // ! Rewrites free pages so secrets freed by an earlier plaintext write cannot be read back from the file. Must not run inside a transaction.
    pub fn vacuum(&self, context: &'static str) -> Result<()> {
        let connection = self.lock();
        connection
            .execute_batch("VACUUM")
            .map_err(|error| SteamError::io(context, error_as_io(&error)))
    }
}

// ! Persists whether crossing the `secure_delete` migration boundary still owes its `VACUUM`, kept in `setting` under a key no app-level preference uses.
const VACUUM_PENDING_KEY: &str = "_storage_vacuum_pending";

fn vacuum_pending(connection: &Connection) -> rusqlite::Result<bool> {
    connection
        .query_row(
            "SELECT 1 FROM setting WHERE key = ?1",
            [VACUUM_PENDING_KEY],
            |_| Ok(()),
        )
        .optional()
        .map(|row| row.is_some())
}

fn set_vacuum_pending(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute(
        "INSERT OR REPLACE INTO setting (key, value) VALUES (?1, '1')",
        [VACUUM_PENDING_KEY],
    )?;
    Ok(())
}

fn clear_vacuum_pending(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute("DELETE FROM setting WHERE key = ?1", [VACUUM_PENDING_KEY])?;
    Ok(())
}

// ! `<name>.db.bak`, or `<name>.db.<unix time>[-n].bak` when that exists already: a backup made by an earlier recovery is never overwritten.
fn unused_backup_path(path: &Path) -> PathBuf {
    let first = path.with_extension("db.bak");
    if !first.exists() {
        return first;
    }
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let mut attempt = 0u32;
    loop {
        let suffix = if attempt == 0 {
            format!("db.{stamp}.bak")
        } else {
            format!("db.{stamp}-{attempt}.bak")
        };
        let candidate = path.with_extension(suffix);
        if !candidate.exists() {
            return candidate;
        }
        attempt += 1;
    }
}

// ! Moves the database and the journal files SQLite keeps next to it, so the fresh database does not pick up a write-ahead log from the damaged one.
fn move_aside(path: &Path, backup: &Path) -> Result<()> {
    fs::rename(path, backup)
        .map_err(|error| SteamError::io("Keep the unreadable database aside", error))?;
    for suffix in ["-wal", "-shm", "-journal"] {
        let sibling = PathBuf::from(format!("{}{suffix}", path.display()));
        if sibling.exists() {
            let target = PathBuf::from(format!("{}{suffix}", backup.display()));
            let _ = fs::rename(&sibling, &target);
        }
    }
    Ok(())
}

// ! The database holds Steam Guard secrets, so only the owner may read it. SQLite's journal files copy the database's permissions.
fn restrict_permissions(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }
    #[cfg(not(unix))]
    let _ = path;
}

fn error_as_io(error: &rusqlite::Error) -> std::io::Error {
    std::io::Error::other(error.to_string())
}
