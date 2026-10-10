use super::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GameState {
    pub name: String,
    pub total_seconds: f64,
    pub sessions: u64,
    pub last_played_at: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AccountState {
    pub account_name: String,
    pub persona_name: String,
    pub total_seconds: f64,
    pub games: BTreeMap<String, GameState>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SessionRecord {
    pub session_id: String,
    pub account_id: String,
    pub account_name: String,
    pub persona_name: String,
    pub app_id: String,
    pub game_name: String,
    pub root_pid: Option<u32>,
    pub process_create_time: Option<f64>,
    pub launch_timestamp: f64,
    pub detected_at: f64,
    pub started_at: f64,
    pub tracked_started_at: Option<f64>,
    pub last_seen_at: Option<f64>,
    pub ended_at: Option<f64>,
    pub duration_seconds: Option<f64>,
    pub tracking_mode: String,
    pub end_reason: String,
    pub exit_code: Option<i32>,
    pub launch_command: String,
}

/// Where the log reader stopped. `known` is false until a log has been seen.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct LogCursor {
    pub known: bool,
    pub device: u64,
    pub inode: u64,
    pub modified: f64,
    pub offset: u64,
    pub head: Vec<u8>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlaytimeState {
    pub tracking_started_at: f64,
    pub history_started_at: f64,
    pub source: LogCursor,
    pub accounts: BTreeMap<String, AccountState>,
    pub active_sessions: BTreeMap<String, SessionRecord>,
    pub sessions: Vec<SessionRecord>,
}

impl PlaytimeState {
    pub(super) fn fresh(now: f64) -> Self {
        Self {
            tracking_started_at: now,
            history_started_at: now,
            source: LogCursor::default(),
            accounts: BTreeMap::new(),
            active_sessions: BTreeMap::new(),
            sessions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActiveGame {
    pub app_id: String,
    pub name: String,
    pub account_id: String,
    pub elapsed_seconds: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GamePlaytime {
    pub app_id: String,
    pub name: String,
    pub seconds: f64,
    pub sessions: u64,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccountPlaytime {
    pub account_id: String,
    pub account_name: String,
    pub persona_name: String,
    pub seconds: f64,
    pub game_count: usize,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MachineGamePlaytime {
    pub app_id: String,
    pub name: String,
    pub seconds: f64,
    pub sessions: u64,
    pub account_count: usize,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionPlaytime {
    pub session_id: String,
    pub account_id: String,
    pub account_name: String,
    pub persona_name: String,
    pub app_id: String,
    pub game_name: String,
    pub launch_timestamp: f64,
    pub started_at: f64,
    pub ended_at: Option<f64>,
    pub duration_seconds: f64,
    pub active: bool,
    pub tracking_mode: String,
    pub end_reason: String,
    pub exit_code: Option<i32>,
    pub root_pid: Option<u32>,
    pub process_create_time: Option<f64>,
}

impl SessionPlaytime {
    pub fn display_name(&self) -> &str {
        if !self.persona_name.is_empty() {
            &self.persona_name
        } else if !self.account_name.is_empty() {
            &self.account_name
        } else {
            &self.account_id
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DailyPlaytime {
    pub day: Date,
    pub seconds: f64,
    pub sessions: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaytimeStatistics {
    pub total_seconds: f64,
    pub detailed_seconds: f64,
    pub session_count: usize,
    pub average_session_seconds: f64,
    pub longest_session_seconds: f64,
    pub first_session_at: Option<f64>,
    pub history_started_at: f64,
}
