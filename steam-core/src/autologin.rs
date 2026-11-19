use crate::error::Result;
use crate::model::SteamInstallation;

pub fn read_auto_login(installation: &SteamInstallation) -> Result<String> {
    #[cfg(windows)]
    {
        let _ = installation;
        Ok(crate::windows_registry::read_auto_login().unwrap_or_default())
    }

    #[cfg(not(windows))]
    crate::linux_registry::read_auto_login(installation)
}

// ! An empty name clears the selection.
pub fn write_auto_login(installation: &SteamInstallation, account_name: &str) -> Result<()> {
    #[cfg(windows)]
    {
        let _ = installation;
        crate::windows_registry::write_auto_login(account_name).map_err(|error| {
            crate::error::SteamError::io("Unable to update the Steam registry key", error)
        })
    }

    #[cfg(not(windows))]
    crate::linux_registry::write_auto_login(installation, account_name)
}

// ! Steam's own self-report of the process it believes is running, used to verify a launch instead of trusting a process named "steam" to be the real client (see `processes::launch_steam_verified`).
pub fn read_active_process_pid(installation: &SteamInstallation) -> Result<Option<u32>> {
    #[cfg(windows)]
    {
        let _ = installation;
        Ok(crate::windows_registry::active_process_pid())
    }

    #[cfg(not(windows))]
    crate::linux_registry::read_active_process_pid(installation)
}
