// ! Steam's folders, processes and launch command are only known for Linux and
// ! Windows; any other target would build an application that detects nothing.
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
compile_error!("steam-core supports Linux and Windows only.");

pub mod accounts;
pub mod authenticator;
pub mod autologin;
pub mod error;
pub mod error_code;
pub mod games;
pub mod metadata;
pub mod model;
pub mod pathdetect;
pub mod playtime;
pub mod processes;
pub mod ranks;
pub mod storage;
pub mod util;
pub mod vdf;

#[cfg(windows)]
pub mod windows_registry;

pub use accounts::SteamManager;
pub use authenticator::Authenticator;
pub use error::{Result, SteamError};
pub use metadata::RankStore;
pub use model::{InstallKind, SteamAccount, SteamInstallation};
pub use pathdetect::detect_all;
pub use playtime::PlaytimeTracker;
pub use util::format_playtime;
