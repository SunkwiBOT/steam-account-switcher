//! Reads and writes the account Steam will log into on its next start.
//!
//! Unit tests use fixture files on every OS and never write the real Windows registry.

use crate::error::{Result, SteamError};
use crate::model::SteamInstallation;
#[cfg(any(not(windows), test))]
use crate::vdf;

#[cfg(any(not(windows), test))]
const REGISTRY_PATH: [&str; 5] = ["Registry", "HKCU", "Software", "Valve", "Steam"];

/// Auto-login account name, empty when Steam has none remembered.
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
        let steam = root.path_mut(&REGISTRY_PATH, true).ok_or_else(|| {
            SteamError::config("Unable to locate or create the Steam registry section.")
        })?;

        steam.set_string("AutoLoginUser", account_name);
        steam.set_string(
            "RememberPassword",
            if account_name.is_empty() { "0" } else { "1" },
        );
        steam.set_string("AlreadyLoggedIn", "0");

        write_registry(installation, &root).map_err(|error| {
            SteamError::config(explain_write_failure(&installation.registry_path, &error))
        })
    }
}

/// Some guides tell users to mark `registry.vdf` immutable (`chattr +i`) to
/// stop Steam from resetting auto-login. The atomic rename then fails with a
/// bare permission error, which reads like a bug in this application, so name
/// the likely cause instead.
#[cfg(any(not(windows), test))]
fn explain_write_failure(path: &std::path::Path, error: &SteamError) -> String {
    let text = error.to_string();
    let lowered = text.to_ascii_lowercase();
    if lowered.contains("permission denied") || lowered.contains("operation not permitted") {
        return format!(
            "{text}. {} may be write-protected: `chattr +i` on Linux (or a read-only \
Steam folder) stops Steam from resetting auto-login. Remove the protection with \
`sudo chattr -i {}` and retry.",
            path.display(),
            path.display()
        );
    }
    text
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::InstallKind;
    use std::path::PathBuf;

    fn installation(registry_path: PathBuf) -> SteamInstallation {
        SteamInstallation {
            kind: InstallKind::Native,
            root: registry_path.parent().unwrap().to_path_buf(),
            config_dir: registry_path.parent().unwrap().join("config"),
            loginusers_path: registry_path
                .parent()
                .unwrap()
                .join("config/loginusers.vdf"),
            registry_path: registry_path.clone(),
            userdata_dir: registry_path.parent().unwrap().join("userdata"),
            launch_command: vec!["steam".to_string()],
            prefix: None,
        }
    }

    #[test]
    fn writes_then_reads_the_auto_login_user() {
        let directory = std::env::temp_dir().join("steam-core-autologin");
        std::fs::create_dir_all(&directory).expect("temp dir");
        let registry_path = directory.join("registry.vdf");
        std::fs::write(
            &registry_path,
            "\"Registry\"\n{\n\t\"HKCU\"\n\t{\n\t\t\"Software\"\n\t\t{\n\t\t\t\"Valve\"\n\t\t\t{\n\t\t\t\t\"Steam\"\n\t\t\t\t{\n\t\t\t\t\t\"AutoLoginUser\"\t\t\"old\"\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t}\n}\n",
        )
        .expect("seed registry");

        let installation = installation(registry_path.clone());
        assert_eq!(read_auto_login(&installation).unwrap(), "old");

        write_auto_login(&installation, "new-account").expect("write");
        assert_eq!(read_auto_login(&installation).unwrap(), "new-account");

        let text = std::fs::read_to_string(&registry_path).expect("read back");
        assert!(text.contains("\"RememberPassword\"\t\t\"1\""));
        assert!(text.contains("\"AlreadyLoggedIn\"\t\t\"0\""));

        write_auto_login(&installation, "").expect("clear");
        assert_eq!(read_auto_login(&installation).unwrap(), "");

        let _ = std::fs::remove_dir_all(&directory);
    }
}
