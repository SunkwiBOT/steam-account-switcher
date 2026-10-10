use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_unix() -> f64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs_f64(),
        Err(_) => 0.0,
    }
}

/// Writes a synced temporary file, then renames it over `path`: readers see the
/// old or the new contents, never a partial file. The rename is not fsynced, so
/// after a power loss the previous contents may still be the ones on disk.
pub fn write_atomic(path: &Path, contents: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let temporary = unique_temporary(path);
    let result = write_synced(&temporary, contents).and_then(|()| fs::rename(&temporary, path));
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn write_synced(path: &Path, contents: &str) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(contents.as_bytes())?;
    file.sync_all()
}

fn unique_temporary(path: &Path) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "steam-account-switcher".to_string());
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    path.with_file_name(format!(".{file_name}.{}.{count}.tmp", std::process::id()))
}

pub fn format_playtime(seconds: f64) -> String {
    let total_seconds = seconds.max(0.0) as i64;
    if total_seconds < 60 {
        return if total_seconds == 0 {
            "0m".to_string()
        } else {
            "<1m".to_string()
        };
    }

    let total_minutes = total_seconds / 60;
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;

    if hours == 0 {
        format!("{minutes}m")
    } else if minutes == 0 {
        format!("{hours}h")
    } else {
        format!("{hours}h {minutes}m")
    }
}

pub fn relative_time(timestamp: Option<i64>) -> String {
    let Some(timestamp) = timestamp else {
        return String::new();
    };

    let seconds = (now_unix() as i64 - timestamp).max(0) as u64;
    if seconds < 1 {
        return "just now".to_string();
    }

    const UNITS: [(&str, u64); 6] = [
        ("year", 365 * 24 * 60 * 60),
        ("month", 30 * 24 * 60 * 60),
        ("day", 24 * 60 * 60),
        ("hour", 60 * 60),
        ("minute", 60),
        ("second", 1),
    ];

    let mut remaining = seconds;
    let mut parts: Vec<String> = Vec::new();
    for (name, unit_seconds) in UNITS {
        let value = remaining / unit_seconds;
        remaining %= unit_seconds;
        if value > 0 {
            let suffix = if value == 1 { "" } else { "s" };
            parts.push(format!("{value} {name}{suffix}"));
            if parts.len() == 2 {
                break;
            }
        }
    }

    if parts.is_empty() {
        "just now".to_string()
    } else {
        format!("{} ago", parts.join(", "))
    }
}

pub fn absolute_time(timestamp: Option<i64>) -> String {
    let Some(timestamp) = timestamp else {
        return String::new();
    };

    match time::OffsetDateTime::from_unix_timestamp(timestamp) {
        Ok(instant) => {
            let local = instant.to_offset(local_offset());
            let format = time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]");
            local
                .format(&format)
                .unwrap_or_else(|_| timestamp.to_string())
        }
        Err(_) => timestamp.to_string(),
    }
}

/// Local UTC offset, resolved once because the lookup is not thread safe on
/// every platform.
pub fn local_offset() -> time::UtcOffset {
    use std::sync::OnceLock;
    static OFFSET: OnceLock<time::UtcOffset> = OnceLock::new();
    *OFFSET.get_or_init(|| time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC))
}
