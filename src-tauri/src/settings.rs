//! User preferences that are not tied to a single Steam installation.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use steam_core::model::{home_dir, resolve};
use steam_core::util::write_atomic;

fn default_true() -> bool {
    true
}

/// A user created group of accounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Show the "Installation" picker in the header.
    #[serde(default = "default_true")]
    pub show_installation_selector: bool,
    /// Show the machine total in the footer.
    #[serde(default = "default_true")]
    pub show_machine_playtime: bool,
    /// Roots of the automatically detected Steam installations to ignore.
    #[serde(default)]
    pub hidden_installations: Vec<String>,
    /// Extra Steam roots added by hand.
    #[serde(default)]
    pub custom_installations: Vec<String>,
    /// User created folders shown above the account cards.
    #[serde(default)]
    pub folders: Vec<Folder>,
    /// `steam_id` -> folder id.
    #[serde(default)]
    pub account_folders: BTreeMap<String, String>,
    /// `steam_id` -> accent colour of the card.
    #[serde(default)]
    pub account_colors: BTreeMap<String, String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_installation_selector: true,
            show_machine_playtime: true,
            hidden_installations: Vec::new(),
            custom_installations: Vec::new(),
            folders: Vec::new(),
            account_folders: BTreeMap::new(),
            account_colors: BTreeMap::new(),
        }
    }
}

impl Settings {
    pub fn is_hidden(&self, root: &str) -> bool {
        self.hidden_installations
            .iter()
            .any(|candidate| same_path(candidate, root))
    }

    pub fn is_custom(&self, root: &str) -> bool {
        self.custom_installations
            .iter()
            .any(|candidate| same_path(candidate, root))
    }

    pub fn hide(&mut self, root: &str) {
        if !self.is_hidden(root) {
            self.hidden_installations.push(root.to_string());
        }
    }

    pub fn show(&mut self, root: &str) {
        self.hidden_installations
            .retain(|candidate| !same_path(candidate, root));
    }

    pub fn add_custom(&mut self, root: &str) {
        if !self.is_custom(root) {
            self.custom_installations.push(root.to_string());
        }
        self.show(root);
    }

    pub fn remove_custom(&mut self, root: &str) {
        self.custom_installations
            .retain(|candidate| !same_path(candidate, root));
        self.show(root);
    }

    /// Creates a folder and returns its id, keeping names unique.
    pub fn create_folder(&mut self, name: &str) -> String {
        let base = slug(name);
        let mut id = base.clone();
        let mut counter = 2;
        while self.folders.iter().any(|folder| folder.id == id) {
            id = format!("{base}-{counter}");
            counter += 1;
        }
        self.folders.push(Folder {
            id: id.clone(),
            name: name.trim().to_string(),
        });
        id
    }

    pub fn rename_folder(&mut self, id: &str, name: &str) -> bool {
        match self.folders.iter_mut().find(|folder| folder.id == id) {
            Some(folder) => {
                folder.name = name.trim().to_string();
                true
            }
            None => false,
        }
    }

    /// Deletes a folder and moves its accounts back to the root.
    pub fn delete_folder(&mut self, id: &str) -> bool {
        let before = self.folders.len();
        self.folders.retain(|folder| folder.id != id);
        self.account_folders.retain(|_, folder| folder != id);
        self.folders.len() != before
    }

    /// Moves an account into a folder, or back to the root when `folder` is
    /// `None` or points at a folder that does not exist.
    pub fn set_account_folder(&mut self, steam_id: &str, folder: Option<&str>) {
        let valid = folder
            .map(|id| self.folders.iter().any(|entry| entry.id == id))
            .unwrap_or(false);
        match (valid, folder) {
            (true, Some(id)) => {
                self.account_folders
                    .insert(steam_id.to_string(), id.to_string());
            }
            _ => {
                self.account_folders.remove(steam_id);
            }
        }
    }

    /// Sets (or clears) the accent colour of an account card.
    pub fn set_account_color(&mut self, steam_id: &str, color: Option<&str>) {
        match color.filter(|value| is_safe_color(value)) {
            Some(color) => {
                self.account_colors
                    .insert(steam_id.to_string(), color.to_string());
            }
            None => {
                self.account_colors.remove(steam_id);
            }
        }
    }

    pub fn account_folder(&self, steam_id: &str) -> Option<&str> {
        self.account_folders.get(steam_id).map(String::as_str)
    }

    pub fn account_color(&self, steam_id: &str) -> Option<&str> {
        self.account_colors.get(steam_id).map(String::as_str)
    }
}

/// Folder ids are derived from the name so they stay readable in the file.
fn slug(name: &str) -> String {
    let mut slug: String = name
        .trim()
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect();
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }
    let trimmed = slug.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "folder".to_string()
    } else {
        trimmed
    }
}

/// Only plain hex colours are stored, so a value can never inject CSS.
fn is_safe_color(value: &str) -> bool {
    let bytes = value.as_bytes();
    matches!(bytes.len(), 4 | 7 | 9)
        && bytes[0] == b'#'
        && bytes[1..].iter().all(u8::is_ascii_hexdigit)
}

/// Settings file kept beside the other user configuration.
pub struct SettingsStore {
    path: PathBuf,
    data: Settings,
}

impl SettingsStore {
    pub fn load() -> Self {
        let path = default_path();
        let data = std::fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<Settings>(&text).ok())
            .unwrap_or_default();
        Self { path, data }
    }

    pub fn data(&self) -> &Settings {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Settings {
        &mut self.data
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.data)
            .map_err(|error| format!("Unable to encode settings: {error}"))?;
        write_atomic(&self.path, &json)
            .map_err(|error| format!("Unable to write {}: {error}", self.path.display()))
    }
}

/// `%APPDATA%\steam-account-switcher` on Windows,
/// `$XDG_CONFIG_HOME/steam-account-switcher` (or `~/.config`) elsewhere.
pub fn config_dir() -> PathBuf {
    #[cfg(windows)]
    {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            return PathBuf::from(appdata).join("steam-account-switcher");
        }
    }

    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| home_dir().map(|home| home.join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("steam-account-switcher")
}

fn default_path() -> PathBuf {
    config_dir().join("settings.json")
}

/// Compares paths the way the settings file stores them.
fn same_path(left: &str, right: &str) -> bool {
    resolve(std::path::Path::new(left)) == resolve(std::path::Path::new(right))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hiding_and_showing_installations_is_idempotent() {
        let mut settings = Settings::default();
        settings.hide("/tmp/steam");
        settings.hide("/tmp/steam");
        assert_eq!(settings.hidden_installations.len(), 1);
        assert!(settings.is_hidden("/tmp/steam"));

        settings.show("/tmp/steam");
        assert!(!settings.is_hidden("/tmp/steam"));
    }

    #[test]
    fn custom_installations_are_never_hidden() {
        let mut settings = Settings::default();
        settings.add_custom("/tmp/custom-steam");
        assert!(settings.is_custom("/tmp/custom-steam"));
        assert!(!settings.is_hidden("/tmp/custom-steam"));

        settings.hide("/tmp/custom-steam");
        settings.add_custom("/tmp/custom-steam");
        assert!(!settings.is_hidden("/tmp/custom-steam"));

        settings.remove_custom("/tmp/custom-steam");
        assert!(!settings.is_custom("/tmp/custom-steam"));
    }

    #[test]
    fn defaults_show_everything() {
        let settings = Settings::default();
        assert!(settings.show_installation_selector);
        assert!(settings.show_machine_playtime);
        assert!(settings.hidden_installations.is_empty());
    }

    #[test]
    fn folders_are_created_with_unique_ids() {
        let mut settings = Settings::default();
        let first = settings.create_folder("Main accounts");
        let second = settings.create_folder("Main accounts");
        assert_eq!(first, "main-accounts");
        assert_eq!(second, "main-accounts-2");
        assert_eq!(settings.folders.len(), 2);
    }

    #[test]
    fn deleting_a_folder_returns_its_accounts_to_the_root() {
        let mut settings = Settings::default();
        let folder = settings.create_folder("Smurfs");
        settings.set_account_folder("76561198000000001", Some(&folder));
        assert_eq!(settings.account_folder("76561198000000001"), Some("smurfs"));

        settings.delete_folder(&folder);
        assert!(settings.folders.is_empty());
        assert!(settings.account_folder("76561198000000001").is_none());
    }

    #[test]
    fn unknown_folders_and_unsafe_colours_are_rejected() {
        let mut settings = Settings::default();
        settings.set_account_folder("76561198000000001", Some("missing"));
        assert!(settings.account_folder("76561198000000001").is_none());

        settings.set_account_color("76561198000000001", Some("#8b5cf6"));
        assert_eq!(settings.account_color("76561198000000001"), Some("#8b5cf6"));

        settings.set_account_color("76561198000000001", Some("red; background:url(x)"));
        assert!(settings.account_color("76561198000000001").is_none());
    }

    #[test]
    fn renaming_keeps_the_folder_id() {
        let mut settings = Settings::default();
        let folder = settings.create_folder("Smurfs");
        assert!(settings.rename_folder(&folder, "Secondaries"));
        assert_eq!(settings.folders[0].name, "Secondaries");
        assert_eq!(settings.folders[0].id, folder);
        assert!(!settings.rename_folder("missing", "nope"));
    }
}
