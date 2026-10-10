use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use time::{Date, Duration, OffsetDateTime, PrimitiveDateTime};

use crate::error::{Result, SteamError};
use crate::games::SteamGameCatalog;
use crate::model::{SteamAccount, SteamInstallation};
use crate::processes::ProcessFacts;
use crate::storage::Database;
use crate::util::{local_offset, now_unix};
use rusqlite::{params, Connection, OptionalExtension};

pub const CHECKPOINT_INTERVAL: f64 = 30.0;

/// Leading bytes of the log that identify it. Rotation is detected when they
/// change, which works on every platform (Windows has no inode in std).
const LOG_HEAD_SIZE: u64 = 256;

pub fn installation_key(installation: &SteamInstallation) -> String {
    use sha1::{Digest, Sha1};
    let key = crate::model::resolve(&installation.config_dir);
    format!("{:x}", Sha1::digest(key.as_os_str().as_encoded_bytes()))
}

/// Events older than the app start are only trusted when the process is alive.
pub const FRESH_EVENT_TOLERANCE: f64 = 5.0;
const READ_CHUNK_SIZE: u64 = 4 * 1024 * 1024;
const DISCOVERY_READ_SIZE: u64 = 16 * 1024 * 1024;

pub type ClockRef = Arc<dyn Fn() -> f64 + Send + Sync>;
pub type ProbeRef = Arc<dyn Fn(u32) -> ProcessFacts + Send + Sync>;
pub type AccountProvider = Arc<dyn Fn() -> Option<SteamAccount> + Send + Sync>;

mod log;
mod model;
mod store;
mod tracker;

pub use log::{parse_gameprocess_line, running_game_events, EventKind, GameProcessEvent};
pub use model::*;
pub use store::{clear_stored_history, recorded_account_count, stored_installations};
pub use tracker::{system_clock, PlaytimeTracker};

use log::*;
use store::*;
