use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use time::Date;

use ts_rs::TS;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrackingMode {
    // ! The game was already running when tracking started: no launch event seen, `started_at` is an estimate.
    #[default]
    Partial,
    Full,
}

impl TrackingMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Partial => "PARTIAL",
            Self::Full => "FULL",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "PARTIAL" => Some(Self::Partial),
            "FULL" => Some(Self::Full),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EndReason {
    ReplacedByNewLaunch,
    RootProcessExitLog,
    ProcessMissingAfterRestart,
    ProcessExit,
    TrackingDisabled,
    SteamRunningList,
    // ! A finished session loaded with no reason recorded.
    Unknown,
}

impl EndReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReplacedByNewLaunch => "REPLACED_BY_NEW_LAUNCH",
            Self::RootProcessExitLog => "ROOT_PROCESS_EXIT_LOG",
            Self::ProcessMissingAfterRestart => "PROCESS_MISSING_AFTER_RESTART",
            Self::ProcessExit => "PROCESS_EXIT",
            Self::TrackingDisabled => "TRACKING_DISABLED",
            Self::SteamRunningList => "STEAM_RUNNING_LIST",
            Self::Unknown => "UNKNOWN",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "REPLACED_BY_NEW_LAUNCH" => Some(Self::ReplacedByNewLaunch),
            "ROOT_PROCESS_EXIT_LOG" => Some(Self::RootProcessExitLog),
            "PROCESS_MISSING_AFTER_RESTART" => Some(Self::ProcessMissingAfterRestart),
            "PROCESS_EXIT" => Some(Self::ProcessExit),
            "TRACKING_DISABLED" => Some(Self::TrackingDisabled),
            "STEAM_RUNNING_LIST" => Some(Self::SteamRunningList),
            "UNKNOWN" => Some(Self::Unknown),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlaytimeStatus {
    Running,
    Finished,
}

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
    pub tracking_mode: TrackingMode,
    pub end_reason: Option<EndReason>,
    pub exit_code: Option<i32>,
    pub launch_command: String,
}

// ! Where the log reader stopped. `known` is false until a log has been seen.
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
    pub tracking_mode: TrackingMode,
    pub end_reason: Option<EndReason>,
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
