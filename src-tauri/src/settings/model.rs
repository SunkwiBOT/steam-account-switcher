use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use steam_core::error_code::ErrorCode;

use super::{default_true, same_path, SearchScope, UpdateInterval, VaultLockInterval};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
}

/// Upper bound for one account note, markup included.
pub const MAX_NOTE_BYTES: usize = 64 * 1024;

/// Folder icons offered by the interface. Unknown values are never stored.
pub const FOLDER_ICONS: [&str; 20] = [
    "folder",
    "folder-open",
    "star",
    "heart",
    "flame",
    "zap",
    "trophy",
    "crown",
    "target",
    "crosshair",
    "gamepad-2",
    "swords",
    "shield",
    "rocket",
    "ghost",
    "gem",
    "book",
    "music",
    "palette",
    "globe",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub language: crate::locale::Language,
    #[serde(default)]
    pub show_account_names: bool,
    #[serde(default)]
    pub show_steam_ids: bool,
    #[serde(default = "default_true")]
    pub show_installation_selector: bool,
    #[serde(default = "default_true")]
    pub tracking_enabled: bool,
    #[serde(default = "default_true")]
    pub close_to_tray: bool,
    #[serde(default)]
    pub update_interval: UpdateInterval,
    #[serde(default)]
    pub search_scope: SearchScope,
    #[serde(default)]
    pub vault_lock_interval: VaultLockInterval,
    #[serde(default)]
    pub last_update_check_at: Option<u64>,
    #[serde(default)]
    pub hidden_installations: Vec<String>,
    #[serde(default)]
    pub custom_installations: Vec<String>,
    #[serde(default)]
    pub folders: Vec<Folder>,
    #[serde(default)]
    pub account_folders: BTreeMap<String, String>,
    #[serde(default)]
    pub account_colors: BTreeMap<String, String>,
    #[serde(default)]
    pub account_notes: BTreeMap<String, String>,
    /// Stored notes that could not be parsed and could not be exported to a
    /// recovery file yet, written back as they were found.
    #[serde(skip)]
    pub unreadable_account_notes: Vec<String>,
    #[serde(default)]
    pub guard_account_profiles: BTreeMap<String, crate::retained_accounts::SavedAccountProfile>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: crate::locale::Language::Auto,
            show_account_names: false,
            show_steam_ids: false,
            show_installation_selector: true,
            tracking_enabled: true,
            close_to_tray: true,
            update_interval: UpdateInterval::SixHours,
            search_scope: SearchScope::Both,
            vault_lock_interval: VaultLockInterval::Startup,
            last_update_check_at: None,
            hidden_installations: Vec::new(),
            custom_installations: Vec::new(),
            folders: Vec::new(),
            account_folders: BTreeMap::new(),
            account_colors: BTreeMap::new(),
            account_notes: BTreeMap::new(),
            unreadable_account_notes: Vec::new(),
            guard_account_profiles: BTreeMap::new(),
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
            color: None,
            icon: None,
        });
        id
    }

    pub fn set_folder_color(&mut self, id: &str, color: Option<&str>) -> bool {
        let value = color
            .filter(|value| is_safe_color(value))
            .map(str::to_string);
        match self.folders.iter_mut().find(|folder| folder.id == id) {
            Some(folder) => {
                folder.color = value;
                true
            }
            None => false,
        }
    }

    pub fn set_folder_icon(&mut self, id: &str, icon: Option<&str>) -> bool {
        let value = icon
            .filter(|value| FOLDER_ICONS.contains(value))
            .map(str::to_string);
        match self.folders.iter_mut().find(|folder| folder.id == id) {
            Some(folder) => {
                folder.icon = value;
                true
            }
            None => false,
        }
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

    /// An empty note removes the entry. Only markup in the form the
    /// interface's sanitiser writes is accepted.
    pub fn set_account_note(&mut self, steam_id: &str, note: &str) -> Result<(), ErrorCode> {
        if note.len() > MAX_NOTE_BYTES {
            return Err(ErrorCode::NoteTooLong);
        }
        if !is_note_markup(note) {
            return Err(ErrorCode::InvalidNote);
        }
        if note.trim().is_empty() {
            self.account_notes.remove(steam_id);
        } else {
            self.account_notes
                .insert(steam_id.to_string(), note.to_string());
        }
        Ok(())
    }

    pub fn account_note(&self, steam_id: &str) -> Option<&str> {
        self.account_notes.get(steam_id).map(String::as_str)
    }

    pub fn account_folder(&self, steam_id: &str) -> Option<&str> {
        self.account_folders.get(steam_id).map(String::as_str)
    }

    pub fn account_color(&self, steam_id: &str) -> Option<&str> {
        self.account_colors.get(steam_id).map(String::as_str)
    }
}

/// ! Notes reach `innerHTML`, so anything but the sanitiser's own output is
/// ! refused: its text is escaped, and its only tags are the ones below.
fn is_note_markup(note: &str) -> bool {
    let mut rest = note;
    while let Some(start) = rest.find(['<', '>']) {
        let tag = &rest[start..];
        let Some(end) = tag.find('>').filter(|_| tag.starts_with('<')) else {
            return false;
        };
        if !is_note_tag(&tag[1..end]) {
            return false;
        }
        rest = &tag[end + 1..];
    }
    true
}

fn is_note_tag(tag: &str) -> bool {
    const BLOCKS: [&str; 9] = ["b", "i", "u", "s", "div", "ul", "ol", "li", "span"];
    if tag == "br" || BLOCKS.contains(&tag.strip_prefix('/').unwrap_or("")) {
        return true;
    }
    if tag != "span" && BLOCKS.contains(&tag) {
        return true;
    }
    tag.strip_prefix("span style=\"color:#")
        .and_then(|rest| rest.strip_suffix('"'))
        .is_some_and(|hex| {
            hex.len() == 6
                && hex
                    .bytes()
                    .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
        })
}

pub(super) fn slug(name: &str) -> String {
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

/// ! Only plain hex colours are stored, so a value can never inject CSS.
pub(super) fn is_safe_color(value: &str) -> bool {
    let bytes = value.as_bytes();
    matches!(bytes.len(), 4 | 7 | 9)
        && bytes[0] == b'#'
        && bytes[1..].iter().all(u8::is_ascii_hexdigit)
}

#[cfg(test)]
mod tests {
    use super::is_note_markup;

    #[test]
    fn accepts_sanitised_markup() {
        assert!(is_note_markup(""));
        assert!(is_note_markup(
            "<div><b>a &lt;b&gt; &amp;</b></div><ul><li><span style=\"color:#f43f5e\">x</span></li></ul><br>"
        ));
    }

    #[test]
    fn refuses_other_markup() {
        assert!(!is_note_markup("<img src=x onerror=alert(1)>"));
        assert!(!is_note_markup("<script>alert(1)</script>"));
        assert!(!is_note_markup("<span style=\"color:red\">x</span>"));
        assert!(!is_note_markup("<span>x</span>"));
        assert!(!is_note_markup("<b onclick=x>"));
        assert!(!is_note_markup("a > b"));
        assert!(!is_note_markup("<b"));
    }
}
