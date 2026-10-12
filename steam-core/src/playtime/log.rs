use super::{ProcessFacts, DISCOVERY_READ_SIZE, LOG_HEAD_SIZE};
use crate::error::{Result, SteamError};
use crate::util::local_wall_clock_to_unix;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use time::PrimitiveDateTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    Start,
    End,
    Remove,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameProcessEvent {
    pub kind: EventKind,
    pub timestamp: f64,
    pub app_id: String,
    pub pid: Option<u32>,
    pub command: String,
    pub exit_code: Option<i32>,
}

pub fn parse_gameprocess_line(line: &str) -> Option<GameProcessEvent> {
    let line = line.trim_end_matches(['\r', '\n']).trim();
    let rest = line.strip_prefix('[')?;
    let (timestamp, body) = rest.split_once(']')?;
    let timestamp = parse_log_timestamp(timestamp.trim())?;
    let body = body.trim_start();

    if let Some(after) = body.strip_prefix("AppID ") {
        let (app_id, after) = take_number(after)?;
        if let Some(after) = after.strip_prefix(" adding PID ") {
            let (pid, after) = take_number(after)?;
            let command = after
                .strip_prefix(" as a tracked process ")
                .unwrap_or_default();
            return Some(GameProcessEvent {
                kind: EventKind::Start,
                timestamp,
                app_id: app_id.to_string(),
                pid: pid.parse().ok(),
                command: command.to_string(),
                exit_code: None,
            });
        }
        if let Some(after) = after.strip_prefix(" no longer tracking PID ") {
            let (pid, after) = take_number(after)?;
            return Some(GameProcessEvent {
                kind: EventKind::End,
                timestamp,
                app_id: app_id.to_string(),
                pid: pid.parse().ok(),
                command: String::new(),
                exit_code: parse_exit_code(after),
            });
        }
    }

    if let Some(after) = body.strip_prefix("Remove ") {
        let (app_id, after) = take_number(after)?;
        if after.trim_start().starts_with("from running list") {
            return Some(GameProcessEvent {
                kind: EventKind::Remove,
                timestamp,
                app_id: app_id.to_string(),
                pid: None,
                command: String::new(),
                exit_code: None,
            });
        }
    }

    None
}

pub(super) fn probe_event_process(
    event: &GameProcessEvent,
    probe: &dyn Fn(u32) -> ProcessFacts,
) -> ProcessFacts {
    let Some(pid) = event.pid else {
        return ProcessFacts {
            running: false,
            start_time: None,
        };
    };

    let facts = probe(pid);
    if !facts.running {
        return facts;
    }

    // A live process whose start time is far from the launch event is a reused
    // pid, not this session's game.
    match facts.start_time {
        Some(created_at) if (created_at - event.timestamp).abs() > 180.0 => ProcessFacts {
            running: false,
            start_time: facts.start_time,
        },
        _ => facts,
    }
}

pub(super) fn recent_game_events(log_path: &Path) -> Result<Vec<GameProcessEvent>> {
    let mut handle = match File::open(log_path) {
        Ok(handle) => handle,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(SteamError::io("Read running Steam games", error)),
    };
    let metadata = handle
        .metadata()
        .map_err(|error| SteamError::io("Read running Steam games", error))?;
    let offset = metadata.len().saturating_sub(DISCOVERY_READ_SIZE);
    handle
        .seek(SeekFrom::Start(offset))
        .map_err(|error| SteamError::io("Read running Steam games", error))?;
    let mut data = Vec::new();
    handle
        .take(DISCOVERY_READ_SIZE)
        .read_to_end(&mut data)
        .map_err(|error| SteamError::io("Read running Steam games", error))?;
    // The window can start inside a multi-byte character; that partial line is
    // skipped below, so lossy decoding never costs the whole file.
    let text = String::from_utf8_lossy(&data);
    let mut events = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if index == 0 && offset > 0 {
            continue;
        }
        if let Some(event) = parse_gameprocess_line(line) {
            events.push(event);
        } else if line.contains("Client version:") {
            events.clear();
        }
    }
    Ok(events)
}

/// Read-only check independent of playtime tracking.
pub fn running_game_events(
    log_path: &Path,
    probe: &dyn Fn(u32) -> ProcessFacts,
) -> Result<Vec<GameProcessEvent>> {
    let mut candidates = BTreeMap::new();
    for event in recent_game_events(log_path)? {
        match event.kind {
            EventKind::Start => {
                candidates.insert((event.app_id.clone(), event.pid), event);
            }
            EventKind::End => {
                candidates.remove(&(event.app_id, event.pid));
            }
            EventKind::Remove => {
                candidates.retain(|(app_id, _), _| app_id != &event.app_id);
            }
        }
    }
    Ok(candidates
        .into_values()
        .filter(|event| probe_event_process(event, probe).running)
        .collect())
}

/// Steam writes `..., exit code 0` on most platforms.
pub(super) fn parse_exit_code(rest: &str) -> Option<i32> {
    let marker = rest.find("exit code ")?;
    let value = &rest[marker + "exit code ".len()..];
    let end = value
        .find(|character: char| !character.is_ascii_digit() && character != '-')
        .unwrap_or(value.len());
    value[..end].parse::<i32>().ok()
}

pub(super) fn take_number(input: &str) -> Option<(&str, &str)> {
    let end = input
        .find(|character: char| !character.is_ascii_digit())
        .unwrap_or(input.len());
    if end == 0 {
        return None;
    }
    Some((&input[..end], &input[end..]))
}

/// Steam logs local time without a zone: `2026-01-10 12:00:00`.
pub(super) fn parse_log_timestamp(value: &str) -> Option<f64> {
    let format = time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
    let primitive = PrimitiveDateTime::parse(value, &format).ok()?;
    Some(local_wall_clock_to_unix(primitive) as f64)
}

pub(super) fn read_head(handle: &mut File) -> std::io::Result<Vec<u8>> {
    handle.seek(SeekFrom::Start(0))?;
    let mut head = Vec::new();
    handle.take(LOG_HEAD_SIZE).read_to_end(&mut head)?;
    Ok(head)
}

#[cfg(unix)]
pub(super) fn file_identity(metadata: &std::fs::Metadata) -> (u64, u64, f64) {
    use std::os::unix::fs::MetadataExt;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs_f64())
        .unwrap_or(0.0);
    (metadata.dev(), metadata.ino(), modified)
}

// std exposes no file index on Windows; rotation is caught by the head fingerprint.
#[cfg(not(unix))]
pub(super) fn file_identity(metadata: &std::fs::Metadata) -> (u64, u64, f64) {
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs_f64())
        .unwrap_or(0.0);
    (0, 0, modified)
}
