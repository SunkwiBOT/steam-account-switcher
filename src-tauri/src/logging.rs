//! Append-only log of the actions performed on Steam.
//!
//! Switching drives another application, so when something goes wrong the
//! timeline has to be readable afterwards without running a debug build.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use steam_core::util::absolute_time;

const MAX_BYTES: u64 = 256 * 1024;

static TARGET: Mutex<Option<PathBuf>> = Mutex::new(None);

pub fn init(directory: PathBuf) {
    if fs::create_dir_all(&directory).is_err() {
        return;
    }
    if let Ok(mut slot) = TARGET.lock() {
        *slot = Some(directory.join("steam-account-switcher.log"));
    }
}

pub fn write(message: &str) {
    let Ok(slot) = TARGET.lock() else {
        return;
    };
    let Some(path) = slot.clone() else {
        return;
    };
    drop(slot);

    rotate(&path);
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(file, "[{}] {message}", now_text());
    }
}

fn rotate(path: &Path) {
    let too_big = fs::metadata(path)
        .map(|meta| meta.len() > MAX_BYTES)
        .unwrap_or(false);
    if too_big {
        let _ = fs::rename(path, path.with_extension("log.old"));
    }
}

fn now_text() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or_default();
    absolute_time(Some(seconds))
}
