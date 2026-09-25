//! Locates every Steam data root on the machine.
//!
//! Linux users can have several clients at once (distribution package,
//! Flatpak and Snap), so the detector returns them all and lets the UI decide.

use std::path::{Path, PathBuf};

#[cfg(not(windows))]
use crate::model::home_dir;
use crate::model::{resolve, InstallKind, SteamInstallation};

/// Returns the executable for `name` when it is available on `PATH`.
pub fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for directory in std::env::split_paths(&path) {
        for candidate in executable_names(name) {
            let full = directory.join(candidate);
            if full.is_file() {
                return Some(full);
            }
        }
    }
    None
}

#[cfg(windows)]
fn executable_names(name: &str) -> Vec<String> {
    if name.to_ascii_lowercase().ends_with(".exe") {
        return vec![name.to_string()];
    }
    vec![format!("{name}.exe"), name.to_string()]
}

#[cfg(not(windows))]
fn executable_names(name: &str) -> Vec<String> {
    vec![name.to_string()]
}

/// Detects every Steam installation, best candidate first.
#[cfg(windows)]
pub fn detect_all() -> Vec<SteamInstallation> {
    rank_candidates(windows_candidates())
}

/// Detects every Steam installation, best candidate first.
#[cfg(not(windows))]
pub fn detect_all() -> Vec<SteamInstallation> {
    let Some(home) = home_dir() else {
        return Vec::new();
    };
    rank_candidates(linux_candidates(&home))
}

/// Keeps the first occurrence of each installation and sorts by completeness.
fn rank_candidates(candidates: Vec<SteamInstallation>) -> Vec<SteamInstallation> {
    let mut unique: Vec<SteamInstallation> = Vec::new();
    for installation in candidates {
        if !looks_installed(&installation) {
            continue;
        }
        let duplicate = unique.iter().any(|existing| {
            existing.kind == installation.kind
                && resolve(&existing.root) == resolve(&installation.root)
                && resolve(&existing.registry_path) == resolve(&installation.registry_path)
        });
        if !duplicate {
            unique.push(installation);
        }
    }

    unique.sort_by_key(|installation| std::cmp::Reverse(installation_score(installation)));
    unique
}

/// Directories that may contain a Steam client below a user supplied path.
#[cfg(not(windows))]
const CUSTOM_SUBPATHS: [&str; 7] = [
    "",
    ".steam/steam",
    ".steam/root",
    ".local/share/Steam",
    ".var/app/com.valvesoftware.Steam/.local/share/Steam",
    "steam",
    "Steam",
];

#[cfg(windows)]
const CUSTOM_SUBPATHS: [&str; 2] = ["", "Steam"];

/// Builds an installation from a path typed by the user, either a Steam root or
/// a parent directory such as `~/.steam`.
pub fn from_custom_path(path: &Path) -> Option<SteamInstallation> {
    let base = crate::model::resolve(&crate::model::expand_home(path));
    if !base.is_dir() {
        return None;
    }

    let root = CUSTOM_SUBPATHS
        .iter()
        .map(|relative| {
            if relative.is_empty() {
                base.clone()
            } else {
                base.join(relative)
            }
        })
        .find(|candidate| looks_like_steam_root(candidate))?;

    #[cfg(windows)]
    {
        let launch = root.join("steam.exe");
        let command = if launch.is_file() {
            vec![launch.to_string_lossy().to_string()]
        } else {
            vec!["steam.exe".to_string()]
        };
        Some(build_installation_windows(&root, command))
    }

    #[cfg(not(windows))]
    {
        let kind = custom_kind(&root);
        let commands = match kind {
            InstallKind::Flatpak => {
                command_candidates(&[&["flatpak", "run", "com.valvesoftware.Steam"]])
            }
            InstallKind::Snap => command_candidates(&[&["snap", "run", "steam"], &["steam"]]),
            _ => command_candidates(&[&["steam"], &["steam-native"]]),
        };
        let registry = [base.join("registry.vdf"), root.join("registry.vdf")];
        let prefix = base.parent().map(Path::to_path_buf);
        Some(build_installation(
            kind, &root, &registry, &commands, prefix,
        ))
    }
}

/// A directory is a Steam root when it holds any of the client's markers.
pub fn looks_like_steam_root(path: &Path) -> bool {
    path.join("config/loginusers.vdf").is_file()
        || path.join("registry.vdf").is_file()
        || path.join("steamapps").is_dir()
        || path.join("userdata").is_dir()
        || path.join("steam.exe").is_file()
        || path.join("steam.sh").is_file()
}

/// Why a folder is, or is not, usable as a Steam root: Steam is often installed
/// but never signed in, which is not the same as "not found".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SteamFolder {
    /// Missing, or a file rather than a folder.
    NotADirectory,
    /// A real folder that has nothing to do with a Steam client.
    NotSteam,
    /// Steam is installed here but never signed in: no login history yet.
    NeverSignedIn,
    /// `config/loginusers.vdf` exists, accounts can be read.
    Usable,
}

impl SteamFolder {
    pub fn message(self) -> &'static str {
        match self {
            SteamFolder::NotADirectory => "This folder does not exist.",
            SteamFolder::NotSteam => "No Steam client found in this folder.",
            SteamFolder::NeverSignedIn => {
                "Steam found, sign in once so it creates its login history."
            }
            SteamFolder::Usable => "Steam installation ready.",
        }
    }
}

/// Classifies a single folder, used for the picker and for later reads so a
/// path can never be accepted by one and rejected by the other.
pub fn classify_steam_folder(path: &Path) -> SteamFolder {
    if !path.is_dir() {
        return SteamFolder::NotADirectory;
    }
    if path.join("config/loginusers.vdf").is_file() {
        return SteamFolder::Usable;
    }
    if looks_like_steam_root(path) {
        return SteamFolder::NeverSignedIn;
    }
    SteamFolder::NotSteam
}

/// Classification of a user supplied path, after expanding the known layouts.
pub fn classify_custom_path(path: &Path) -> SteamFolder {
    let base = crate::model::resolve(&crate::model::expand_home(path));
    let classification = classify_steam_folder(&base);
    if classification != SteamFolder::Usable && classification != SteamFolder::NotADirectory {
        for relative in CUSTOM_SUBPATHS.iter().filter(|entry| !entry.is_empty()) {
            match classify_steam_folder(&base.join(relative)) {
                SteamFolder::Usable => return SteamFolder::Usable,
                SteamFolder::NeverSignedIn => return SteamFolder::NeverSignedIn,
                _ => {}
            }
        }
    }
    classification
}

#[cfg(not(windows))]
fn custom_kind(root: &Path) -> InstallKind {
    let text = root.to_string_lossy().to_ascii_lowercase();
    if text.contains(".var/app/com.valvesoftware.steam") {
        InstallKind::Flatpak
    } else if text.contains("/snap/") || text.contains("snap/steam") {
        InstallKind::Snap
    } else {
        InstallKind::Native
    }
}

#[cfg(not(windows))]
fn linux_candidates(home: &Path) -> Vec<SteamInstallation> {
    let mut candidates = Vec::new();

    let mut native_roots = vec![
        home.join(".local/share/Steam"),
        home.join(".steam/steam"),
        home.join(".steam/root"),
        home.join(".steam/debian-installation"),
    ];
    // A non-default XDG_DATA_HOME moves the install somewhere else entirely.
    if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
        let steam = PathBuf::from(xdg).join("Steam");
        if !native_roots.contains(&steam) {
            native_roots.insert(0, steam);
        }
    }
    let native_registry = [home.join(".steam/registry.vdf")];
    let native_commands = command_candidates(&[&["steam"], &["steam-native"]]);
    for root in native_roots {
        candidates.push(build_installation(
            InstallKind::Native,
            &root,
            &native_registry,
            &native_commands,
            Some(home.join(".steam")),
        ));
    }

    let flatpak_home = home.join(".var/app/com.valvesoftware.Steam");
    let flatpak_roots = [
        flatpak_home.join(".local/share/Steam"),
        flatpak_home.join(".steam/steam"),
        flatpak_home.join("steam"),
    ];
    let flatpak_registry = [flatpak_home.join(".steam/registry.vdf")];
    let flatpak_commands = command_candidates(&[&["flatpak", "run", "com.valvesoftware.Steam"]]);
    for root in flatpak_roots {
        candidates.push(build_installation(
            InstallKind::Flatpak,
            &root,
            &flatpak_registry,
            &flatpak_commands,
            Some(flatpak_home.join(".steam")),
        ));
    }

    let snap_home = home.join("snap/steam");
    let snap_roots = [
        snap_home.join("common/.local/share/Steam"),
        snap_home.join("current/.local/share/Steam"),
        snap_home.join("common/.steam/steam"),
    ];
    let snap_registry = [
        snap_home.join("common/.steam/registry.vdf"),
        snap_home.join("current/.steam/registry.vdf"),
    ];
    let snap_commands = command_candidates(&[&["snap", "run", "steam"], &["steam"]]);
    for root in snap_roots {
        let prefix = root.parent().map(Path::to_path_buf);
        candidates.push(build_installation(
            InstallKind::Snap,
            &root,
            &snap_registry,
            &snap_commands,
            prefix,
        ));
    }

    candidates
}

#[cfg(windows)]
fn windows_candidates() -> Vec<SteamInstallation> {
    let mut roots: Vec<PathBuf> = Vec::new();

    if let Some(path) = crate::windows_registry::steam_path() {
        roots.push(path);
    }
    if let Some(path) = crate::windows_registry::steam_install_path() {
        roots.push(path);
    }
    for variable in ["ProgramFiles(x86)", "ProgramFiles", "ProgramW6432"] {
        if let Some(base) = std::env::var_os(variable) {
            roots.push(PathBuf::from(base).join("Steam"));
        }
    }

    let mut candidates = Vec::new();
    for root in roots {
        let launch = root.join("steam.exe");
        let command = if launch.is_file() {
            vec![launch.to_string_lossy().to_string()]
        } else if let Some(steam) = which("steam") {
            vec![steam.to_string_lossy().to_string()]
        } else {
            vec!["steam.exe".to_string()]
        };

        candidates.push(build_installation_windows(&root, command));
    }

    candidates
}

#[cfg(not(windows))]
fn command_candidates(commands: &[&[&str]]) -> Vec<Vec<String>> {
    commands
        .iter()
        .map(|command| command.iter().map(|part| (*part).to_string()).collect())
        .collect()
}

#[cfg(not(windows))]
fn build_installation(
    kind: InstallKind,
    root: &Path,
    registry_candidates: &[PathBuf],
    commands: &[Vec<String>],
    prefix: Option<PathBuf>,
) -> SteamInstallation {
    let root = resolve(root);
    let config_dir = root.join("config");

    let mut registry_options: Vec<PathBuf> = registry_candidates.to_vec();
    registry_options.push(root.join("registry.vdf"));
    if let Some(parent) = root.parent() {
        registry_options.push(parent.join("registry.vdf"));
    }
    let registry_path = registry_options
        .iter()
        .find(|candidate| candidate.is_file())
        .cloned()
        .unwrap_or_else(|| registry_options[0].clone());

    SteamInstallation {
        kind,
        config_dir: config_dir.clone(),
        loginusers_path: config_dir.join("loginusers.vdf"),
        registry_path: resolve(&registry_path),
        userdata_dir: root.join("userdata"),
        launch_command: choose_launch_command(commands),
        prefix: prefix.map(|path| resolve(&path)),
        root,
    }
}

#[cfg(windows)]
fn build_installation_windows(root: &Path, launch_command: Vec<String>) -> SteamInstallation {
    let root = resolve(root);
    let config_dir = root.join("config");

    SteamInstallation {
        kind: InstallKind::Windows,
        config_dir: config_dir.clone(),
        loginusers_path: config_dir.join("loginusers.vdf"),
        // The Windows client stores auto-login state in the real registry, so
        // this path only exists for diagnostics and legacy imports.
        registry_path: root.join("registry.vdf"),
        userdata_dir: root.join("userdata"),
        launch_command,
        prefix: None,
        root,
    }
}

#[cfg(not(windows))]
fn choose_launch_command(commands: &[Vec<String>]) -> Vec<String> {
    commands
        .iter()
        .find(|command| {
            command
                .first()
                .map(|binary| which(binary).is_some() || Path::new(binary).is_file())
                .unwrap_or(false)
        })
        .or_else(|| commands.first())
        .cloned()
        .unwrap_or_default()
}

fn looks_installed(installation: &SteamInstallation) -> bool {
    installation.root.is_dir()
        || installation.config_dir.is_dir()
        || installation.loginusers_path.is_file()
        || installation.registry_path.is_file()
}

fn installation_score(installation: &SteamInstallation) -> (u8, u8, u8) {
    (
        installation.loginusers_path.is_file() as u8,
        installation.registry_path.is_file() as u8,
        installation.config_dir.is_dir() as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installation_label_contains_kind_and_root() {
        let installation = SteamInstallation {
            kind: InstallKind::Flatpak,
            root: PathBuf::from("/tmp/steam"),
            config_dir: PathBuf::from("/tmp/steam/config"),
            loginusers_path: PathBuf::from("/tmp/steam/config/loginusers.vdf"),
            registry_path: PathBuf::from("/tmp/steam/registry.vdf"),
            userdata_dir: PathBuf::from("/tmp/steam/userdata"),
            launch_command: vec!["flatpak".to_string()],
            prefix: None,
        };
        assert_eq!(installation.label(), "flatpak: /tmp/steam");
    }

    #[test]
    fn detection_never_panics_and_is_sorted() {
        let installations = detect_all();
        let scores: Vec<_> = installations.iter().map(installation_score).collect();
        let mut sorted = scores.clone();
        sorted.sort_by(|left, right| right.cmp(left));
        assert_eq!(scores, sorted);
    }

    #[test]
    fn custom_paths_resolve_a_steam_root_or_parent_directory() {
        let directory =
            std::env::temp_dir().join(format!("steam-core-custom-path-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        let root = directory.join("Steam");
        std::fs::create_dir_all(root.join("config")).expect("config dir");
        std::fs::write(root.join("config/loginusers.vdf"), "\"users\"\n{\n}\n").expect("file");

        let direct = from_custom_path(&root).expect("root resolves");
        assert_eq!(direct.root, resolve(&root));

        let parent = from_custom_path(&directory).expect("parent resolves");
        assert_eq!(parent.root, resolve(&root));

        assert!(from_custom_path(&directory.join("missing")).is_none());

        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn unrelated_directories_are_rejected() {
        let directory =
            std::env::temp_dir().join(format!("steam-core-not-steam-{}", std::process::id()));
        std::fs::create_dir_all(&directory).expect("temp dir");
        assert!(from_custom_path(&directory).is_none());
        let _ = std::fs::remove_dir_all(&directory);
    }
}
