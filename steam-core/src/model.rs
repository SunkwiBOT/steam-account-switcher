//! Shared data structures describing a local Steam installation.

use std::path::{Path, PathBuf};

/// How Steam was installed. This drives path layout and how Steam is launched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InstallKind {
    /// Linux distribution package or upstream tarball (`~/.steam`, `~/.local/share/Steam`).
    Native,
    /// Flatpak build of the Steam client.
    Flatpak,
    /// Snap build of the Steam client.
    Snap,
    /// Windows installation (registry + `steam.exe`).
    Windows,
}

impl InstallKind {
    pub fn as_str(self) -> &'static str {
        match self {
            InstallKind::Native => "native",
            InstallKind::Flatpak => "flatpak",
            InstallKind::Snap => "snap",
            InstallKind::Windows => "windows",
        }
    }
}

/// A Steam data root with everything the app needs to read or write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamInstallation {
    pub kind: InstallKind,
    pub root: PathBuf,
    pub config_dir: PathBuf,
    pub loginusers_path: PathBuf,
    pub registry_path: PathBuf,
    pub userdata_dir: PathBuf,
    pub launch_command: Vec<String>,
    pub prefix: Option<PathBuf>,
}

impl SteamInstallation {
    /// Human readable label used by the installation picker.
    pub fn label(&self) -> String {
        format!("{}: {}", self.kind.as_str(), self.root.display())
    }
}

/// One account remembered by the local Steam client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamAccount {
    pub steam_id: String,
    pub account_name: String,
    pub persona_name: String,
    pub avatar_path: Option<PathBuf>,
    pub most_recent: bool,
    pub remember_password: bool,
    pub allow_auto_login: bool,
    pub wants_offline_mode: bool,
    pub skip_offline_mode_warning: bool,
    pub timestamp: Option<i64>,
}

impl SteamAccount {
    /// Persona name when Steam has one, otherwise the login name.
    pub fn display_name(&self) -> &str {
        if !self.persona_name.is_empty() {
            &self.persona_name
        } else if !self.account_name.is_empty() {
            &self.account_name
        } else {
            &self.steam_id
        }
    }

    /// Single uppercase letter used by the generated avatar.
    pub fn avatar_initial(&self) -> String {
        let label = self.display_name().trim();
        match label.chars().next() {
            Some(character) => character.to_uppercase().to_string(),
            None => "?".to_string(),
        }
    }
}

/// Steam keeps `Timestamp` as a unix epoch written in the client's local time.
pub fn parse_timestamp(value: &str) -> Option<i64> {
    value.trim().parse::<i64>().ok().filter(|value| *value > 0)
}

/// Steam writes `"1"`/`"0"`; older clients used `true`/`false`.
pub fn parse_vdf_bool(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes"
    )
}

/// Resolves `~` and canonicalizes without requiring the path to exist.
pub fn resolve(path: &Path) -> PathBuf {
    let expanded = expand_home(path);
    expanded.canonicalize().unwrap_or(expanded)
}

pub fn expand_home(path: &Path) -> PathBuf {
    let raw = path.to_string_lossy();
    if let Some(rest) = raw.strip_prefix("~/") {
        if let Some(home) = home_dir() {
            return home.join(rest);
        }
    }
    path.to_path_buf()
}

pub fn home_dir() -> Option<PathBuf> {
    // `HOME` on unix, `USERPROFILE` on Windows; falling back to the platform
    // specific variables keeps this dependency free.
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE")
            .or_else(|| {
                std::env::var_os("HOMEDRIVE")
                    .zip(std::env::var_os("HOMEPATH"))
                    .map(|(drive, path)| {
                        let mut joined = drive;
                        joined.push(path);
                        joined
                    })
            })
            .map(PathBuf::from)
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}
