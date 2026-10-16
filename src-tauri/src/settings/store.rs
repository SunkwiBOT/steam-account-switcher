use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, Transaction};
use serde::de::DeserializeOwned;
use serde::Serialize;
use steam_core::error_code::ErrorCode;
use steam_core::storage::Database;

use super::{Folder, Settings};

pub struct SettingsStore {
    database: Database,
    pub(super) data: Settings,
}

impl SettingsStore {
    /// Reads the settings from the database. A database that cannot be read is kept
    /// aside by `Database::open`, and the defaults are used for this run.
    pub fn load(database: Database) -> Self {
        let data = database
            .read("Read settings", read_settings)
            .unwrap_or_default();
        let mut store = Self { database, data };
        if !store.data.unreadable_account_notes.is_empty() {
            let notes = std::mem::take(&mut store.data.unreadable_account_notes);
            let exported = notes.len();
            store.data.unreadable_account_notes =
                export_unreadable_notes(store.database.path(), notes);
            // Only the copies now safe in a file leave the database.
            if store.data.unreadable_account_notes.len() < exported {
                let _ = store.save();
            }
        }
        store
    }

    pub fn data(&self) -> &Settings {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Settings {
        &mut self.data
    }

    pub fn save(&self) -> Result<(), String> {
        self.database
            .transaction("Save settings", |transaction| {
                write_settings(transaction, &self.data)
            })
            .map_err(|_| ErrorCode::SettingsSaveFailed.to_string())
    }
}

fn read_text(connection: &Connection, key: &str) -> Option<String> {
    connection
        .query_row("SELECT value FROM setting WHERE key = ?1", [key], |row| {
            row.get(0)
        })
        .ok()
}

pub(super) fn read_value<T: DeserializeOwned>(connection: &Connection, key: &str) -> Option<T> {
    serde_json::from_str(&read_text(connection, key)?).ok()
}

const NOTES_KEY: &str = "account_notes";
const UNREADABLE_NOTES_KEY: &str = "account_notes_unreadable";

/// ! Notes cannot be rebuilt from Steam, so a value that does not parse is kept
/// ! verbatim instead of being replaced by an empty map. Every copy kept by an
/// ! earlier run is read back too, so a second failure never drops the first.
fn read_notes(connection: &Connection) -> (BTreeMap<String, String>, Vec<String>) {
    let mut kept: Vec<String> = connection
        .prepare("SELECT value FROM setting WHERE key LIKE ?1 ORDER BY key")
        .and_then(|mut statement| {
            statement
                .query_map([format!("{UNREADABLE_NOTES_KEY}%")], |row| row.get(0))?
                .collect()
        })
        .unwrap_or_default();
    let Some(text) = read_text(connection, NOTES_KEY) else {
        return (BTreeMap::new(), kept);
    };
    match serde_json::from_str(&text) {
        Ok(notes) => (notes, kept),
        Err(error) => {
            crate::logging::write(&format!(
                "action=read_account_notes status=failed result=kept_aside error={error}"
            ));
            kept.push(text);
            (BTreeMap::new(), kept)
        }
    }
}

/// Writes unreadable notes next to the database, readable by the owner only,
/// so they can be recovered by hand. Returns the copies that could not be written.
fn export_unreadable_notes(database_path: &Path, notes: Vec<String>) -> Vec<String> {
    let Some(directory) = database_path.parent().filter(|path| path.is_dir()) else {
        return notes;
    };
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    notes
        .into_iter()
        .enumerate()
        .filter_map(|(index, text)| {
            let path = directory.join(format!("account-notes-recovered-{stamp}-{index}.json"));
            match write_private(&path, &text) {
                Ok(()) => {
                    crate::logging::write(&format!(
                        "action=export_account_notes status=ok path={}",
                        path.display()
                    ));
                    None
                }
                Err(error) => {
                    crate::logging::write(&format!(
                        "action=export_account_notes status=failed error={error}"
                    ));
                    Some(text)
                }
            }
        })
        .collect()
}

fn write_private(path: &Path, text: &str) -> std::io::Result<()> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()
}

pub(super) fn read_settings(connection: &Connection) -> rusqlite::Result<Settings> {
    let defaults = Settings::default();
    let mut folders_statement =
        connection.prepare("SELECT id, name, color, icon FROM folder ORDER BY position")?;
    let folders = folders_statement
        .query_map([], |row| {
            Ok(Folder {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                icon: row.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let pairs = |sql: &str| -> rusqlite::Result<BTreeMap<String, String>> {
        connection
            .prepare(sql)?
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect()
    };
    let names = |sql: &str| -> rusqlite::Result<Vec<String>> {
        connection
            .prepare(sql)?
            .query_map([], |row| row.get(0))?
            .collect()
    };

    let guard_account_profiles = connection
        .prepare(
            "SELECT steam_id, account_name, persona_name, avatar_path, timestamp FROM retained_account",
        )?
        .query_map([], |row| {
            let avatar: Option<String> = row.get(3)?;
            Ok((
                row.get::<_, String>(0)?,
                crate::retained_accounts::SavedAccountProfile {
                    account_name: row.get(1)?,
                    persona_name: row.get(2)?,
                    avatar_path: avatar.map(PathBuf::from),
                    timestamp: row.get(4)?,
                },
            ))
        })?
        .collect::<rusqlite::Result<BTreeMap<_, _>>>()?;

    let (account_notes, unreadable_account_notes) = read_notes(connection);

    Ok(Settings {
        language: read_value(connection, "language").unwrap_or(defaults.language),
        show_account_names: read_value(connection, "show_account_names")
            .unwrap_or(defaults.show_account_names),
        show_steam_ids: read_value(connection, "show_steam_ids").unwrap_or(defaults.show_steam_ids),
        show_installation_selector: read_value(connection, "show_installation_selector")
            .unwrap_or(defaults.show_installation_selector),
        tracking_enabled: read_value(connection, "tracking_enabled")
            .unwrap_or(defaults.tracking_enabled),
        close_to_tray: read_value(connection, "close_to_tray").unwrap_or(defaults.close_to_tray),
        update_interval: read_value(connection, "update_interval")
            .unwrap_or(defaults.update_interval),
        search_scope: read_value(connection, "search_scope").unwrap_or(defaults.search_scope),
        vault_lock_interval: read_value(connection, "vault_lock_interval")
            .unwrap_or(defaults.vault_lock_interval),
        last_update_check_at: read_value::<Option<u64>>(connection, "last_update_check_at")
            .flatten(),
        hidden_installations: names(
            "SELECT root FROM installation WHERE hidden = 1 ORDER BY root",
        )?,
        custom_installations: names(
            "SELECT root FROM installation WHERE custom = 1 ORDER BY root",
        )?,
        folders,
        account_folders: pairs("SELECT steam_id, folder_id FROM account_folder")?,
        account_colors: pairs("SELECT steam_id, color FROM account_color")?,
        // Notes live in the key/value table so the schema version stays the same.
        account_notes,
        unreadable_account_notes,
        guard_account_profiles,
    })
}

pub(super) fn write_settings(
    transaction: &Transaction<'_>,
    settings: &Settings,
) -> rusqlite::Result<()> {
    transaction.execute_batch(
        "DELETE FROM setting;
         DELETE FROM folder;
         DELETE FROM account_folder;
         DELETE FROM account_color;
         DELETE FROM installation;
         DELETE FROM retained_account;",
    )?;

    put_setting(transaction, "language", &settings.language)?;
    put_setting(
        transaction,
        "show_account_names",
        &settings.show_account_names,
    )?;
    put_setting(transaction, "show_steam_ids", &settings.show_steam_ids)?;
    put_setting(
        transaction,
        "show_installation_selector",
        &settings.show_installation_selector,
    )?;
    put_setting(transaction, "tracking_enabled", &settings.tracking_enabled)?;
    put_setting(transaction, "close_to_tray", &settings.close_to_tray)?;
    put_setting(transaction, "update_interval", &settings.update_interval)?;
    put_setting(transaction, "search_scope", &settings.search_scope)?;
    put_setting(
        transaction,
        "vault_lock_interval",
        &settings.vault_lock_interval,
    )?;
    put_setting(
        transaction,
        "last_update_check_at",
        &settings.last_update_check_at,
    )?;
    put_setting(transaction, NOTES_KEY, &settings.account_notes)?;
    for (index, text) in settings.unreadable_account_notes.iter().enumerate() {
        transaction.execute(
            "INSERT INTO setting (key, value) VALUES (?1, ?2)",
            (format!("{UNREADABLE_NOTES_KEY}.{index:04}"), text),
        )?;
    }

    for (position, folder) in settings.folders.iter().enumerate() {
        transaction.execute(
            "INSERT INTO folder (id, position, name, color, icon) VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                &folder.id,
                position as i64,
                &folder.name,
                &folder.color,
                &folder.icon,
            ),
        )?;
    }
    for (steam_id, folder_id) in &settings.account_folders {
        transaction.execute(
            "INSERT INTO account_folder (steam_id, folder_id) VALUES (?1, ?2)",
            (steam_id, folder_id),
        )?;
    }
    for (steam_id, color) in &settings.account_colors {
        transaction.execute(
            "INSERT INTO account_color (steam_id, color) VALUES (?1, ?2)",
            (steam_id, color),
        )?;
    }

    let mut installations: BTreeMap<&str, (bool, bool)> = BTreeMap::new();
    for root in &settings.hidden_installations {
        installations.entry(root).or_default().0 = true;
    }
    for root in &settings.custom_installations {
        installations.entry(root).or_default().1 = true;
    }
    for (root, (hidden, custom)) in installations {
        transaction.execute(
            "INSERT INTO installation (root, hidden, custom) VALUES (?1, ?2, ?3)",
            (root, hidden, custom),
        )?;
    }

    for (steam_id, profile) in &settings.guard_account_profiles {
        transaction.execute(
            "INSERT INTO retained_account (steam_id, account_name, persona_name, avatar_path, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                steam_id,
                &profile.account_name,
                &profile.persona_name,
                profile
                    .avatar_path
                    .as_ref()
                    .map(|path| path.to_string_lossy().to_string()),
                profile.timestamp,
            ),
        )?;
    }
    Ok(())
}

/// Each preference is stored as its JSON text, so the enums keep their serde names.
pub(super) fn put_setting<T: Serialize>(
    transaction: &Transaction<'_>,
    key: &str,
    value: &T,
) -> rusqlite::Result<()> {
    let text = serde_json::to_string(value).unwrap_or_default();
    transaction.execute(
        "INSERT INTO setting (key, value) VALUES (?1, ?2)",
        (key, text),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn corrupt_notes(database: &Database, text: &str) {
        database
            .transaction("seed", |transaction| {
                transaction.execute(
                    "INSERT OR REPLACE INTO setting (key, value) VALUES (?1, ?2)",
                    (NOTES_KEY, text),
                )?;
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn unreadable_notes_survive_saves_and_later_failures() {
        let database = Database::in_memory().unwrap();
        corrupt_notes(&database, "{\"first\":");
        let mut store = SettingsStore::load(database.clone());
        assert!(store.data().account_notes.is_empty());
        store
            .data_mut()
            .set_account_note("1", "<b>new</b>")
            .unwrap();
        store.save().unwrap();

        corrupt_notes(&database, "{\"second\":");
        let store = SettingsStore::load(database.clone());
        store.save().unwrap();

        let reloaded = SettingsStore::load(database);
        assert_eq!(
            reloaded.data().unreadable_account_notes,
            vec!["{\"first\":".to_string(), "{\"second\":".to_string()]
        );
    }

    #[test]
    fn unreadable_notes_are_exported_next_to_the_database() {
        let directory = std::env::temp_dir().join(format!(
            "notes-export-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("settings.db");
        let database = Database::open(&path).unwrap().database;
        corrupt_notes(&database, "{\"lost\":");

        let store = SettingsStore::load(database.clone());
        assert!(store.data().unreadable_account_notes.is_empty());
        let exported: Vec<_> = std::fs::read_dir(&directory)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("account-notes-recovered-")
            })
            .collect();
        assert_eq!(exported.len(), 1);
        assert_eq!(
            std::fs::read_to_string(exported[0].path()).unwrap(),
            "{\"lost\":"
        );
        assert!(SettingsStore::load(database)
            .data()
            .unreadable_account_notes
            .is_empty());
        let _ = std::fs::remove_dir_all(&directory);
    }
}
