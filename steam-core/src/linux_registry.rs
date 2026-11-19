use crate::error::{Result, SteamError};
use crate::error_code::ErrorCode;
use crate::model::SteamInstallation;
use crate::util::{self, WriteMode};
use crate::vdf;

const STEAM_PATH: [&str; 5] = ["Registry", "HKCU", "Software", "Valve", "Steam"];
const HKLM_STEAM_PATH: [&str; 5] = ["Registry", "HKLM", "Software", "Valve", "Steam"];

pub fn read_auto_login(installation: &SteamInstallation) -> Result<String> {
    if !installation.registry_path.is_file() {
        return Ok(String::new());
    }

    let root = read_registry(installation)?;
    let value = root
        .path(&STEAM_PATH)
        .and_then(|steam| steam.get_str("AutoLoginUser"))
        .unwrap_or_default()
        .to_string();
    Ok(value)
}

// ! An empty name clears the selection.
pub fn write_auto_login(installation: &SteamInstallation, account_name: &str) -> Result<()> {
    if !installation.registry_path.is_file() {
        return Err(SteamError::NotFound(format!(
            "Steam registry not found: {}",
            installation.registry_path.display()
        )));
    }

    let mut root = read_registry(installation)?;
    let steam = root
        .path_mut(&STEAM_PATH, true)
        .ok_or_else(|| SteamError::config(ErrorCode::SteamRegistryFailed))?;

    steam.set_string("AutoLoginUser", account_name);
    steam.set_string(
        "RememberPassword",
        if account_name.is_empty() { "0" } else { "1" },
    );
    steam.set_string("AlreadyLoggedIn", "0");

    write_registry(installation, &root)
        .map_err(|_| SteamError::config(ErrorCode::SteamRegistryFailed))
}

// ! `ActiveProcess\pid` is Windows-only; Steam for Linux mirrors its own PID under `HKLM\...\Steam\SteamPID` in the same file, and also drops a plain `steam.pid` file next to the prefix.
pub fn read_active_process_pid(installation: &SteamInstallation) -> Result<Option<u32>> {
    if let Some(pid) = read_steam_pid_file(installation) {
        return Ok(Some(pid));
    }

    if !installation.registry_path.is_file() {
        return Ok(None);
    }

    let root = read_registry(installation)?;
    Ok(root
        .path(&HKLM_STEAM_PATH)
        .and_then(|steam| steam.get_str("SteamPID"))
        .and_then(|value| value.trim().parse::<u32>().ok()))
}

fn read_steam_pid_file(installation: &SteamInstallation) -> Option<u32> {
    let prefix = installation.prefix.as_ref()?;
    std::fs::read_to_string(prefix.join("steam.pid"))
        .ok()?
        .trim()
        .parse::<u32>()
        .ok()
}

fn read_registry(installation: &SteamInstallation) -> Result<vdf::VdfObject> {
    let text = std::fs::read_to_string(&installation.registry_path).map_err(|error| {
        SteamError::io(
            format!("Unable to read {}", installation.registry_path.display()),
            error,
        )
    })?;

    vdf::parse(&text).map_err(|error| SteamError::Vdf {
        path: installation.registry_path.display().to_string(),
        message: error.message,
    })
}

fn write_registry(installation: &SteamInstallation, root: &vdf::VdfObject) -> Result<()> {
    let text = vdf::dump_verified(root).map_err(|error| SteamError::Vdf {
        path: installation.registry_path.display().to_string(),
        message: error.message,
    })?;
    util::write_atomic(
        &installation.registry_path,
        &text,
        WriteMode::PreserveExisting,
    )
    .map_err(|error| {
        SteamError::io(
            format!("Unable to write {}", installation.registry_path.display()),
            error,
        )
    })
}
