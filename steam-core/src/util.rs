//! Small shared helpers: clocks, atomic writes and formatting.

use std::fs;
use std::io;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_unix() -> f64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs_f64(),
        Err(_) => 0.0,
    }
}

/// Writes through a temporary file so a crash cannot truncate Steam's config.
pub fn write_atomic(path: &Path, contents: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "steam-account-switcher".to_string());
    let temporary = path.with_file_name(format!(".{file_name}.tmp"));

    fs::write(&temporary, contents)?;
    match fs::rename(&temporary, path) {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ = fs::remove_file(&temporary);
            Err(error)
        }
    }
}

/// `0m`, `<1m`, `45m`, `3h`, `3h 20m`.
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

/// Compact relative time used by the "Last Login" column.
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

/// Absolute local timestamp shown in tooltips.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_compact_durations() {
        assert_eq!(format_playtime(0.0), "0m");
        assert_eq!(format_playtime(59.0), "<1m");
        assert_eq!(format_playtime(60.0), "1m");
        assert_eq!(format_playtime(3600.0), "1h");
        assert_eq!(format_playtime(3600.0 + 20.0 * 60.0), "1h 20m");
    }

    #[test]
    fn writes_atomically() {
        let directory = std::env::temp_dir().join("steam-core-atomic-write");
        let path = directory.join("sample.json");
        write_atomic(&path, "{\"value\": 1}").expect("first write");
        write_atomic(&path, "{\"value\": 2}").expect("second write");

        let content = std::fs::read_to_string(&path).expect("file exists");
        assert_eq!(content, "{\"value\": 2}");
        assert!(!directory.join(".sample.json.tmp").exists());
        let _ = std::fs::remove_dir_all(&directory);
    }
}
