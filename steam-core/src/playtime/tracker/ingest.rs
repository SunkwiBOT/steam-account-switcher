//! Reading Steam's content log: locating the cursor, detecting rotation and
//! games already running when tracking starts, and pulling in new lines.

use super::{log_cursor, PlaytimeTracker};
use crate::playtime::log::{
    file_identity, read_head, recent_game_events, EventKind, GameProcessEvent,
};
use crate::playtime::READ_CHUNK_SIZE;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

impl PlaytimeTracker {
    pub(super) fn has_valid_source(&self) -> bool {
        self.state.source.known
    }

    pub(super) fn initialize_at_log_end(&mut self) {
        match std::fs::metadata(&self.log_path) {
            Ok(metadata) => {
                let (device, inode, modified) = file_identity(&metadata);
                let head = File::open(&self.log_path)
                    .and_then(|mut handle| read_head(&mut handle))
                    .unwrap_or_default();
                self.state.source = log_cursor(device, inode, metadata.len(), modified, &head);
            }
            Err(_) => {
                self.state.source = log_cursor(0, 0, 0, 0.0, &[]);
            }
        }
    }

    /// Scans the tail of the log for games that are already running so a
    /// launch that happened before the app started is still counted.
    pub(super) fn discover_games_already_running(&mut self) {
        let Ok(events) = recent_game_events(&self.log_path) else {
            return;
        };
        let mut candidates: HashMap<String, GameProcessEvent> = HashMap::new();
        for event in events {
            match event.kind {
                EventKind::Start => {
                    candidates.insert(event.app_id.clone(), event);
                }
                EventKind::End => {
                    if candidates
                        .get(&event.app_id)
                        .is_some_and(|candidate| candidate.pid == event.pid)
                    {
                        candidates.remove(&event.app_id);
                    }
                }
                EventKind::Remove => {
                    candidates.remove(&event.app_id);
                }
            }
        }

        let now = (self.clock)();
        for event in candidates.into_values() {
            if self.probe_event_process(&event).running {
                self.start_session(&event, now, now, "partial");
            }
        }
    }

    pub(super) fn read_new_lines(&mut self) -> (Vec<String>, bool) {
        let Ok(mut handle) = File::open(&self.log_path) else {
            return (Vec::new(), false);
        };
        let Ok(metadata) = handle.metadata() else {
            return (Vec::new(), false);
        };

        let Ok(head) = read_head(&mut handle) else {
            return (Vec::new(), false);
        };
        let (device, inode, modified) = file_identity(&metadata);
        // ! A rotated log can reuse the device and inode (Windows always reports
        // ! 0), so the head fingerprint is the only reliable rotation check.
        let head_matches = head.starts_with(&self.state.source.head);
        let same_file = self.state.source.known
            && head_matches
            && self.state.source.device == device
            && self.state.source.inode == inode;

        let mut offset = if same_file {
            self.state.source.offset
        } else {
            0
        };
        if offset > metadata.len() {
            offset = 0;
        }

        if handle.seek(SeekFrom::Start(offset)).is_err() {
            return (Vec::new(), false);
        }

        let mut buffer = vec![0_u8; READ_CHUNK_SIZE as usize];
        let read = match handle.read(&mut buffer) {
            Ok(read) => read,
            Err(_) => return (Vec::new(), false),
        };
        let bytes = &buffer[..read];

        let mut source_changed = !same_file;
        let complete = match bytes.iter().rposition(|byte| *byte == b'\n') {
            Some(index) => {
                let consumed = &bytes[..=index];
                let new_offset = offset + consumed.len() as u64;
                source_changed |=
                    !self.state.source.known || self.state.source.offset != new_offset;
                self.state.source = log_cursor(device, inode, new_offset, modified, &head);
                String::from_utf8_lossy(consumed).to_string()
            }
            None => {
                if source_changed {
                    self.state.source = log_cursor(device, inode, offset, modified, &head);
                }
                return (Vec::new(), source_changed);
            }
        };

        let mut text = std::mem::take(&mut self.pending);
        text.push_str(&complete);
        let mut lines: Vec<String> = text.split('\n').map(str::to_string).collect();
        self.pending = lines.pop().unwrap_or_default();
        lines.retain(|line| !line.is_empty());

        (lines, source_changed)
    }
}
