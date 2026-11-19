use std::path::PathBuf;

use steam_core::model::{home_dir, same_path};

fn default_true() -> bool {
    true
}

mod model;
mod preferences;
mod store;

pub use model::*;
pub use preferences::*;
pub use store::*;

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

pub fn database_path() -> PathBuf {
    config_dir().join("steam-account-switcher.db")
}
