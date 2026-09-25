//! Steam logic for the account switcher, with no GUI dependency so it can be
//! tested without a windowing environment.

pub mod accounts;
pub mod autologin;
pub mod error;
pub mod games;
pub mod metadata;
pub mod model;
pub mod pathdetect;
pub mod playtime;
pub mod processes;
pub mod ranks;
pub mod util;
pub mod vdf;

#[cfg(windows)]
pub mod windows_registry;

pub use accounts::SteamManager;
pub use error::{Result, SteamError};
pub use metadata::RankStore;
pub use model::{InstallKind, SteamAccount, SteamInstallation};
pub use pathdetect::detect_all;
pub use playtime::PlaytimeTracker;
pub use util::format_playtime;
