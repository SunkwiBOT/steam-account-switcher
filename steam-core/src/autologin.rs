use crate::error::{Result, SteamError};
#[cfg(any(not(windows), test))]
use crate::error_code::ErrorCode;
use crate::model::SteamInstallation;
#[cfg(any(not(windows), test))]
use crate::vdf;

#[cfg(any(not(windows), test))]
const REGISTRY_PATH: [&str; 5] = ["Registry", "HKCU", "Software", "Valve", "Steam"];

pub fn read_auto_login(installation: &SteamInstallation) -> Result<String> {
    #[cfg(all(windows, not(test)))]
    {
        let _ = installation;
        Ok(crate::windows_registry::read_auto_login().unwrap_or_default())
    }

    #[cfg(any(not(windows), test))]
    {
        if !installation.registry_path.is_file() {
            return Ok(String::new());
        }

        let root = read_registry(installation)?;
        let value = root
            .path(&REGISTRY_PATH)
            .and_then(|steam| steam.get_str("AutoLoginUser"))
            .unwrap_or_default()
            .to_string();
        Ok(value)
    }
}

/// Points Steam at `account_name`. An empty name clears the selection.
pub fn write_auto_login(installation: &SteamInstallation, account_name: &str) -> Result<()> {
    #[cfg(all(windows, not(test)))]
    {
        let _ = installation;
        crate::windows_registry::write_auto_login(account_name)
            .map_err(|error| SteamError::io("Unable to update the Steam registry key", error))
    }

    #[cfg(any(not(windows), test))]
    {
        if !installation.registry_path.is_file() {
            return Err(SteamError::NotFound(format!(
                "Steam registry not found: {}",
                installation.registry_path.display()
            )));
        }

        let mut root = read_registry(installation)?;
        let steam = root
            .path_mut(&REGISTRY_PATH, true)
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
}

#[cfg(any(not(windows), test))]
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

#[cfg(any(not(windows), test))]
fn write_registry(installation: &SteamInstallation, root: &vdf::VdfObject) -> Result<()> {
    crate::util::write_atomic(&installation.registry_path, &vdf::dump(root)).map_err(|error| {
        SteamError::io(
            format!("Unable to write {}", installation.registry_path.display()),
            error,
        )
    })
}
