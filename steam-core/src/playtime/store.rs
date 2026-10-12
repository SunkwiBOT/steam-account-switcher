use super::model::{AccountState, GameState, LogCursor, PlaytimeState, SessionRecord};
use crate::error::Result;
use crate::storage::Database;
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::BTreeMap;

pub fn recorded_account_count(database: &Database, installation: &str) -> Option<usize> {
    database
        .read("Count playtime accounts", |connection| {
            connection.query_row(
                "SELECT COUNT(*) FROM playtime_account WHERE installation = ?1",
                [installation],
                |row| row.get::<_, i64>(0),
            )
        })
        .ok()
        .map(|count| count as usize)
}

/// Includes installations that are no longer detected.
pub fn stored_installations(database: &Database) -> Vec<String> {
    database
        .read("List playtime installations", |connection| {
            let mut statement = connection
                .prepare("SELECT installation FROM playtime_meta ORDER BY installation")?;
            let keys = statement
                .query_map([], |row| row.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            Ok(keys)
        })
        .unwrap_or_default()
}

/// `None` erases every installation.
pub fn clear_stored_history(database: &Database, installation: Option<&str>) -> Result<()> {
    database.transaction("Clear playtime history", |transaction| {
        for table in PLAYTIME_TABLES {
            match installation {
                Some(key) => transaction.execute(
                    &format!("DELETE FROM {table} WHERE installation = ?1"),
                    [key],
                )?,
                None => transaction.execute(&format!("DELETE FROM {table}"), [])?,
            };
        }
        Ok(())
    })
}

pub(super) const PLAYTIME_TABLES: [&str; 5] = [
    "playtime_source",
    "playtime_meta",
    "playtime_account",
    "playtime_game",
    "playtime_session",
];

pub(super) const SESSION_COLUMNS: &str =
    "slot, session_id, account_id, account_name, persona_name, app_id, \
    game_name, root_pid, process_create_time, launch_timestamp, detected_at, started_at, \
    tracked_started_at, last_seen_at, ended_at, duration_seconds, tracking_mode, end_reason, \
    exit_code, launch_command";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SaveScope {
    ThisInstallation,
    EveryInstallation,
}

/// Finished sessions are append-only. When the stored rows still match what the
/// tracker last saved, only the sessions recorded since then are inserted; the
/// small tables are rewritten as before. Any mismatch falls back to a full rewrite.
pub(super) fn save_state(
    database: &Database,
    installation: &str,
    state: &PlaytimeState,
    scope: SaveScope,
    stored_sessions: usize,
) -> Result<()> {
    database.transaction("Save playtime history", |transaction| {
        let appendable = scope == SaveScope::ThisInstallation
            && stored_sessions <= state.sessions.len()
            && {
                let stored: i64 = transaction.query_row(
                    "SELECT COUNT(*) FROM playtime_session WHERE installation = ?1 AND active = 0",
                    [installation],
                    |row| row.get(0),
                )?;
                stored as usize == stored_sessions
            };
        for table in PLAYTIME_TABLES {
            if appendable && table == "playtime_session" {
                transaction.execute(
                    "DELETE FROM playtime_session WHERE installation = ?1 AND active = 1",
                    [installation],
                )?;
                continue;
            }
            match scope {
                SaveScope::ThisInstallation => transaction.execute(
                    &format!("DELETE FROM {table} WHERE installation = ?1"),
                    [installation],
                )?,
                SaveScope::EveryInstallation => {
                    transaction.execute(&format!("DELETE FROM {table}"), [])?
                }
            };
        }
        transaction.execute(
            "INSERT INTO playtime_meta (installation, tracking_started_at, history_started_at)
             VALUES (?1, ?2, ?3)",
            params![installation, state.tracking_started_at, state.history_started_at],
        )?;
        let cursor = &state.source;
        if cursor.known {
            transaction.execute(
                "INSERT INTO playtime_source (installation, device, inode, modified, offset, head)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    installation,
                    cursor.device as i64,
                    cursor.inode as i64,
                    cursor.modified,
                    cursor.offset as i64,
                    cursor.head,
                ],
            )?;
        }
        for (steam_id, account) in &state.accounts {
            transaction.execute(
                "INSERT INTO playtime_account (installation, steam_id, account_name, persona_name, total_seconds)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    installation,
                    steam_id,
                    account.account_name,
                    account.persona_name,
                    account.total_seconds
                ],
            )?;
            for (app_id, game) in &account.games {
                transaction.execute(
                    "INSERT INTO playtime_game (installation, account_id, app_id, name, total_seconds, sessions, last_played_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        installation,
                        steam_id,
                        app_id,
                        game.name,
                        game.total_seconds,
                        game.sessions as i64,
                        game.last_played_at
                    ],
                )?;
            }
        }
        let new_sessions = if appendable {
            &state.sessions[stored_sessions..]
        } else {
            &state.sessions[..]
        };
        for session in new_sessions {
            insert_session(transaction, installation, false, None, session)?;
        }
        for (slot, session) in &state.active_sessions {
            insert_session(transaction, installation, true, Some(slot), session)?;
        }
        Ok(())
    })
}

pub(super) fn insert_session(
    transaction: &rusqlite::Transaction<'_>,
    installation: &str,
    active: bool,
    slot: Option<&str>,
    session: &SessionRecord,
) -> rusqlite::Result<()> {
    transaction.execute(
        "INSERT INTO playtime_session (
            installation, active, slot, session_id, account_id, account_name, persona_name,
            app_id, game_name, root_pid, process_create_time, launch_timestamp, detected_at,
            started_at, tracked_started_at, last_seen_at, ended_at, duration_seconds,
            tracking_mode, end_reason, exit_code, launch_command
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)",
        params![
            installation,
            active,
            slot,
            session.session_id,
            session.account_id,
            session.account_name,
            session.persona_name,
            session.app_id,
            session.game_name,
            session.root_pid.map(i64::from),
            session.process_create_time,
            session.launch_timestamp,
            session.detected_at,
            session.started_at,
            session.tracked_started_at,
            session.last_seen_at,
            session.ended_at,
            session.duration_seconds,
            session.tracking_mode,
            session.end_reason,
            session.exit_code.map(i64::from),
            session.launch_command,
        ],
    )?;
    Ok(())
}

pub(super) fn read_sessions(
    connection: &Connection,
    installation: &str,
    active: bool,
) -> rusqlite::Result<Vec<(Option<String>, SessionRecord)>> {
    let mut statement = connection.prepare(&format!(
        "SELECT {SESSION_COLUMNS} FROM playtime_session WHERE installation = ?1 AND active = ?2 ORDER BY id"
    ))?;
    let rows = statement.query_map(params![installation, active], |row| {
        Ok((
            row.get::<_, Option<String>>(0)?,
            SessionRecord {
                session_id: row.get(1)?,
                account_id: row.get(2)?,
                account_name: row.get(3)?,
                persona_name: row.get(4)?,
                app_id: row.get(5)?,
                game_name: row.get(6)?,
                root_pid: row.get::<_, Option<i64>>(7)?.map(|pid| pid as u32),
                process_create_time: row.get(8)?,
                launch_timestamp: row.get(9)?,
                detected_at: row.get(10)?,
                started_at: row.get(11)?,
                tracked_started_at: row.get(12)?,
                last_seen_at: row.get(13)?,
                ended_at: row.get(14)?,
                duration_seconds: row.get(15)?,
                tracking_mode: row.get(16)?,
                end_reason: row.get(17)?,
                exit_code: row.get::<_, Option<i64>>(18)?.map(|code| code as i32),
                launch_command: row.get(19)?,
            },
        ))
    })?;
    rows.collect()
}

pub(super) fn load_state(database: &Database, installation: &str) -> Result<Option<PlaytimeState>> {
    database.read("Read playtime history", |connection| {
        let meta = connection
            .query_row(
                "SELECT tracking_started_at, history_started_at FROM playtime_meta WHERE installation = ?1",
                [installation],
                |row| Ok((row.get::<_, f64>(0)?, row.get::<_, f64>(1)?)),
            )
            .optional()?;
        let Some((tracking_started_at, history_started_at)) = meta else {
            return Ok(None);
        };

        let source = connection
            .query_row(
                "SELECT device, inode, modified, offset, head FROM playtime_source WHERE installation = ?1",
                [installation],
                |row| {
                    Ok(LogCursor {
                        known: true,
                        device: row.get::<_, i64>(0)? as u64,
                        inode: row.get::<_, i64>(1)? as u64,
                        modified: row.get(2)?,
                        offset: row.get::<_, i64>(3)? as u64,
                        head: row.get::<_, Option<Vec<u8>>>(4)?.unwrap_or_default(),
                    })
                },
            )
            .optional()?
            .unwrap_or_default();

        let mut accounts = BTreeMap::new();
        let mut statement = connection.prepare(
            "SELECT steam_id, account_name, persona_name, total_seconds FROM playtime_account WHERE installation = ?1",
        )?;
        for row in statement.query_map([installation], |row| {
            Ok((
                row.get::<_, String>(0)?,
                AccountState {
                    account_name: row.get(1)?,
                    persona_name: row.get(2)?,
                    total_seconds: row.get(3)?,
                    games: BTreeMap::new(),
                },
            ))
        })? {
            let (steam_id, account) = row?;
            accounts.insert(steam_id, account);
        }
        drop(statement);

        let mut statement = connection.prepare(
            "SELECT account_id, app_id, name, total_seconds, sessions, last_played_at FROM playtime_game WHERE installation = ?1",
        )?;
        for row in statement.query_map([installation], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                GameState {
                    name: row.get(2)?,
                    total_seconds: row.get(3)?,
                    sessions: row.get::<_, i64>(4)? as u64,
                    last_played_at: row.get(5)?,
                },
            ))
        })? {
            let (account_id, app_id, game) = row?;
            accounts
                .entry(account_id)
                .or_default()
                .games
                .insert(app_id, game);
        }
        drop(statement);

        let sessions = read_sessions(connection, installation, false)?
            .into_iter()
            .map(|(_, session)| session)
            .collect();
        let active_sessions = read_sessions(connection, installation, true)?
            .into_iter()
            .map(|(slot, session)| (slot.unwrap_or_default(), session))
            .collect();

        Ok(Some(PlaytimeState {
            tracking_started_at,
            history_started_at,
            source,
            accounts,
            active_sessions,
            sessions,
        }))
    })
}
