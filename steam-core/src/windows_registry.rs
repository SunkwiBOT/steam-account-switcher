//! Windows registry access.
//!
//! Unlike Linux, the Windows Steam client does not write a `registry.vdf`
//! mirror: the auto-login account lives in `HKCU\Software\Valve\Steam`.

use std::io;
use std::path::PathBuf;

use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WRITE};
use winreg::RegKey;

const STEAM_KEY: &str = r"Software\Valve\Steam";

fn current_user_key() -> io::Result<RegKey> {
    RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(STEAM_KEY, KEY_READ)
}

/// `HKCU\Software\Valve\Steam\SteamPath`, written by the Steam installer.
pub fn steam_path() -> Option<PathBuf> {
    let key = current_user_key().ok()?;
    let value: String = key.get_value("SteamPath").ok()?;
    let path = PathBuf::from(value.replace('/', "\\"));
    path.is_dir().then_some(path)
}

/// `HKLM\SOFTWARE\WOW6432Node\Valve\Steam\InstallPath`, used as a fallback when
/// the per-user key is missing (fresh machine, different OS user).
pub fn steam_install_path() -> Option<PathBuf> {
    let key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(
            r"SOFTWARE\WOW6432Node\Valve\Steam",
            KEY_READ | KEY_WOW64_32KEY,
        )
        .or_else(|_| {
            RegKey::predef(HKEY_LOCAL_MACHINE)
                .open_subkey_with_flags(r"SOFTWARE\Valve\Steam", KEY_READ)
        })
        .ok()?;
    let value: String = key.get_value("InstallPath").ok()?;
    let path = PathBuf::from(value.replace('/', "\\"));
    path.is_dir().then_some(path)
}

/// Currently remembered auto-login account name.
pub fn read_auto_login() -> Option<String> {
    let key = current_user_key().ok()?;
    key.get_value::<String, _>("AutoLoginUser").ok()
}

/// Writes the account Steam should log into on its next start.
pub fn write_auto_login(account_name: &str) -> io::Result<()> {
    let key = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(STEAM_KEY, KEY_WRITE)?;
    key.set_value("AutoLoginUser", &account_name.to_string())?;
    key.set_value(
        "RememberPassword",
        &if account_name.is_empty() { 0u32 } else { 1u32 },
    )?;
    Ok(())
}
