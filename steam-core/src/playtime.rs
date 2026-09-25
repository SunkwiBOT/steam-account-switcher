//! Playtime tracking from Steam's `logs/gameprocess_log.txt`: sessions,
//! per-account and per-game totals, daily history. Steam profile hours are
//! never imported.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use time::{Date, Duration, OffsetDateTime, PrimitiveDateTime};

use crate::error::{Result, SteamError};
use crate::games::SteamGameCatalog;
use crate::model::{SteamAccount, SteamInstallation};
use crate::processes::ProcessFacts;
use crate::util::{local_offset, now_unix, write_atomic};

/// How often active sessions are written to disk.
pub const CHECKPOINT_INTERVAL: f64 = 30.0;
/// Events older than the app start are only trusted when the process is alive.
pub const FRESH_EVENT_TOLERANCE: f64 = 5.0;
const READ_CHUNK_SIZE: u64 = 4 * 1024 * 1024;
const DISCOVERY_READ_SIZE: u64 = 16 * 1024 * 1024;
const CURRENT_STATE_VERSION: u32 = 2;

pub type ClockRef = Arc<dyn Fn() -> f64 + Send + Sync>;
pub type ProbeRef = Arc<dyn Fn(u32) -> ProcessFacts + Send + Sync>;
pub type AccountProvider = Arc<dyn Fn() -> Option<SteamAccount> + Send + Sync>;

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
    /// Exit status Steam reported when it stopped tracking the process.
    pub exit_code: Option<i32>,
}

/// Parses one `gameprocess_log.txt` line.
///
/// Only the three event kinds needed for local tracking are recognised; every
/// other line returns `None`.
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

/// Steam writes `..., exit code 0` on most platforms. The value is kept so the
/// session history can explain *how* a game ended.
fn parse_exit_code(rest: &str) -> Option<i32> {
    let marker = rest.find("exit code ")?;
    let value = &rest[marker + "exit code ".len()..];
    let end = value
        .find(|character: char| !character.is_ascii_digit() && character != '-')
        .unwrap_or(value.len());
    value[..end].parse::<i32>().ok()
}

fn take_number(input: &str) -> Option<(&str, &str)> {
    let end = input
        .find(|character: char| !character.is_ascii_digit())
        .unwrap_or(input.len());
    if end == 0 {
        return None;
    }
    Some((&input[..end], &input[end..]))
}

/// Steam logs local time without a zone: `2026-01-10 12:00:00`.
fn parse_log_timestamp(value: &str) -> Option<f64> {
    let format = time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
    let primitive = PrimitiveDateTime::parse(value, &format).ok()?;
    Some(primitive.assume_offset(local_offset()).unix_timestamp() as f64)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GameState {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub total_seconds: f64,
    #[serde(default)]
    pub sessions: u64,
    #[serde(default)]
    pub last_played_at: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AccountState {
    #[serde(default)]
    pub account_name: String,
    #[serde(default)]
    pub persona_name: String,
    #[serde(default)]
    pub total_seconds: f64,
    #[serde(default)]
    pub games: BTreeMap<String, GameState>,
}

/// One session, either still running (`active_sessions`) or archived (`sessions`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SessionRecord {
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub account_id: String,
    #[serde(default)]
    pub account_name: String,
    #[serde(default)]
    pub persona_name: String,
    #[serde(default)]
    pub app_id: String,
    #[serde(default)]
    pub game_name: String,
    #[serde(default)]
    pub root_pid: Option<u32>,
    #[serde(default)]
    pub process_create_time: Option<f64>,
    #[serde(default)]
    pub launch_timestamp: f64,
    #[serde(default)]
    pub detected_at: f64,
    #[serde(default)]
    pub started_at: f64,
    #[serde(default)]
    pub tracked_started_at: Option<f64>,
    #[serde(default)]
    pub last_seen_at: Option<f64>,
    #[serde(default)]
    pub ended_at: Option<f64>,
    #[serde(default)]
    pub duration_seconds: Option<f64>,
    #[serde(default)]
    pub tracking_mode: String,
    #[serde(default)]
    pub end_reason: String,
    /// Exit status reported by Steam for the tracked process, when it said one.
    #[serde(default)]
    pub exit_code: Option<i32>,
    /// Command line Steam logged for the launch, kept for diagnostics.
    #[serde(default)]
    pub launch_command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaytimeState {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub tracking_started_at: f64,
    #[serde(default)]
    pub history_started_at: f64,
    #[serde(default)]
    pub source: Map<String, Value>,
    #[serde(default)]
    pub accounts: BTreeMap<String, AccountState>,
    #[serde(default)]
    pub active_sessions: BTreeMap<String, SessionRecord>,
    #[serde(default)]
    pub sessions: Vec<SessionRecord>,
    /// Unknown keys written by other versions are preserved on save.
    #[serde(flatten, default)]
    pub extra: Map<String, Value>,
}

fn default_version() -> u32 {
    CURRENT_STATE_VERSION
}

impl PlaytimeState {
    fn fresh(now: f64) -> Self {
        Self {
            version: CURRENT_STATE_VERSION,
            tracking_started_at: now,
            history_started_at: now,
            source: Map::new(),
            accounts: BTreeMap::new(),
            active_sessions: BTreeMap::new(),
            sessions: Vec::new(),
            extra: Map::new(),
        }
    }
}

// ---------------------------------------------------------------- view types

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
    pub legacy_seconds: f64,
    pub session_count: usize,
    pub average_session_seconds: f64,
    pub longest_session_seconds: f64,
    pub first_session_at: Option<f64>,
    pub history_started_at: f64,
}

// ------------------------------------------------------------------- tracker

/// Tails Steam's game process log and keeps the local playtime database.
pub struct PlaytimeTracker {
    state_path: PathBuf,
    log_path: PathBuf,
    catalog: SteamGameCatalog,
    state: PlaytimeState,
    instance_started_at: f64,
    last_checkpoint: f64,
    loaded_active_app_ids: HashSet<String>,
    pending: String,
    clock: ClockRef,
    probe: ProbeRef,
    account_provider: AccountProvider,
}

impl PlaytimeTracker {
    pub fn new(
        installation: &SteamInstallation,
        clock: ClockRef,
        probe: ProbeRef,
        account_provider: AccountProvider,
    ) -> Self {
        let instance_started_at = clock();
        let state_path = installation
            .config_dir
            .join("steam-account-switcher-playtime.json");
        let log_path = installation.root.join("logs/gameprocess_log.txt");

        let (mut state, loaded) = load_state(&state_path, instance_started_at);
        let migrated = migrate_state(&mut state, instance_started_at);
        let loaded_active_app_ids = if loaded {
            state.active_sessions.keys().cloned().collect()
        } else {
            HashSet::new()
        };

        let mut tracker = Self {
            state_path,
            log_path,
            catalog: SteamGameCatalog::new(installation),
            state,
            instance_started_at,
            last_checkpoint: instance_started_at,
            loaded_active_app_ids,
            pending: String::new(),
            clock,
            probe,
            account_provider,
        };

        if !loaded || !tracker.has_valid_source() || migrated {
            if !loaded || !tracker.has_valid_source() {
                tracker.initialize_at_log_end();
                tracker.discover_games_already_running();
            }
            let _ = tracker.save();
        }

        tracker
    }

    /// Path of the JSON database backing this tracker.
    pub fn state_path(&self) -> &Path {
        &self.state_path
    }

    pub fn log_path(&self) -> &Path {
        &self.log_path
    }

    pub fn tracking_started_at(&self) -> f64 {
        self.state.tracking_started_at
    }

    pub fn history_started_at(&self) -> f64 {
        self.state.history_started_at
    }

    /// Reads new log lines, updates sessions and checkpoints active ones.
    pub fn poll(&mut self) -> bool {
        let now = (self.clock)();
        let (lines, cursor_changed) = self.read_new_lines();
        let mut changed = cursor_changed;

        for line in lines {
            let Some(event) = parse_gameprocess_line(&line) else {
                continue;
            };
            match event.kind {
                EventKind::Start => changed |= self.handle_start(&event, now),
                EventKind::End => changed |= self.handle_end(&event),
                EventKind::Remove => {
                    changed |= self.finish_session(
                        &event.app_id,
                        event.timestamp,
                        "steam_running_list",
                        None,
                    )
                }
            }
        }

        changed |= self.reconcile_processes(now);

        if !self.state.active_sessions.is_empty()
            && now - self.last_checkpoint >= CHECKPOINT_INTERVAL
        {
            for session in self.state.active_sessions.values_mut() {
                session.last_seen_at = Some(now);
            }
            self.last_checkpoint = now;
            changed = true;
        }

        if changed {
            let _ = self.save();
        }
        changed
    }

    /// Final poll performed while the application shuts down.
    pub fn close(&mut self) {
        self.poll();
        let now = (self.clock)();
        for session in self.state.active_sessions.values_mut() {
            session.last_seen_at = Some(now);
        }
        let _ = self.save();
    }

    // ------------------------------------------------------------- accounting

    pub fn account_seconds(&self, account_id: &str, include_active: bool) -> f64 {
        let stored = self
            .state
            .accounts
            .get(account_id)
            .map(|account| account.total_seconds)
            .unwrap_or(0.0);

        if !include_active {
            return stored;
        }

        let now = (self.clock)();
        stored
            + self
                .state
                .active_sessions
                .values()
                .filter(|session| session.account_id == account_id)
                .map(|session| (now - session.started_at).max(0.0))
                .sum::<f64>()
    }

    pub fn global_seconds(&self, include_active: bool) -> f64 {
        let stored: f64 = self
            .state
            .accounts
            .values()
            .map(|account| account.total_seconds)
            .sum();

        if !include_active {
            return stored;
        }

        let now = (self.clock)();
        stored
            + self
                .state
                .active_sessions
                .values()
                .map(|session| (now - session.started_at).max(0.0))
                .sum::<f64>()
    }

    pub fn active_games(&self) -> Vec<ActiveGame> {
        let now = (self.clock)();
        let mut games: Vec<ActiveGame> = self
            .state
            .active_sessions
            .values()
            .map(|session| ActiveGame {
                app_id: session.app_id.clone(),
                name: self.game_name(session),
                account_id: session.account_id.clone(),
                elapsed_seconds: (now - session.started_at).max(0.0),
            })
            .collect();

        games.sort_by_key(|game| game.name.to_lowercase());
        games
    }

    pub fn account_games(&self, account_id: &str) -> Vec<GamePlaytime> {
        let mut games: HashMap<String, GamePlaytime> = HashMap::new();

        if let Some(account) = self.state.accounts.get(account_id) {
            for (app_id, game) in &account.games {
                games.insert(
                    app_id.clone(),
                    GamePlaytime {
                        app_id: app_id.clone(),
                        name: if game.name.is_empty() {
                            self.catalog.display_name_for(app_id)
                        } else {
                            game.name.clone()
                        },
                        seconds: game.total_seconds,
                        sessions: game.sessions,
                        active: false,
                    },
                );
            }
        }

        let now = (self.clock)();
        for session in self.state.active_sessions.values() {
            if session.account_id != account_id {
                continue;
            }

            let live = (now - session.started_at).max(0.0);
            let entry = games.entry(session.app_id.clone()).or_insert(GamePlaytime {
                app_id: session.app_id.clone(),
                name: self.game_name(session),
                seconds: 0.0,
                sessions: 0,
                active: true,
            });
            entry.seconds += live;
            entry.active = true;
        }

        let mut games: Vec<GamePlaytime> = games.into_values().collect();
        games.sort_by(|left, right| {
            right
                .seconds
                .partial_cmp(&left.seconds)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
        });
        games
    }

    pub fn account_playtimes(&self) -> Vec<AccountPlaytime> {
        let active_accounts: HashSet<&str> = self
            .state
            .active_sessions
            .values()
            .map(|session| session.account_id.as_str())
            .collect();

        let mut summaries: Vec<AccountPlaytime> = self
            .state
            .accounts
            .iter()
            .map(|(account_id, account)| {
                let games = self.account_games(account_id);
                AccountPlaytime {
                    account_id: account_id.clone(),
                    account_name: account.account_name.clone(),
                    persona_name: account.persona_name.clone(),
                    seconds: self.account_seconds(account_id, true),
                    game_count: games.len(),
                    active: active_accounts.contains(account_id.as_str()),
                }
            })
            .collect();

        summaries.sort_by(|left, right| {
            right
                .seconds
                .partial_cmp(&left.seconds)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    let left_label = if left.persona_name.is_empty() {
                        &left.account_name
                    } else {
                        &left.persona_name
                    };
                    let right_label = if right.persona_name.is_empty() {
                        &right.account_name
                    } else {
                        &right.persona_name
                    };
                    left_label.to_lowercase().cmp(&right_label.to_lowercase())
                })
        });

        summaries
    }

    pub fn machine_games(&self) -> Vec<MachineGamePlaytime> {
        struct Aggregate {
            name: String,
            seconds: f64,
            sessions: u64,
            accounts: HashSet<String>,
            active: bool,
        }

        let mut aggregate: HashMap<String, Aggregate> = HashMap::new();
        for account in self.account_playtimes() {
            for game in self.account_games(&account.account_id) {
                let entry = aggregate.entry(game.app_id.clone()).or_insert(Aggregate {
                    name: game.name.clone(),
                    seconds: 0.0,
                    sessions: 0,
                    accounts: HashSet::new(),
                    active: false,
                });
                entry.seconds += game.seconds;
                entry.sessions += game.sessions;
                entry.accounts.insert(account.account_id.clone());
                entry.active |= game.active;
            }
        }

        let mut games: Vec<MachineGamePlaytime> = aggregate
            .into_iter()
            .map(|(app_id, entry)| MachineGamePlaytime {
                app_id,
                name: entry.name,
                seconds: entry.seconds,
                sessions: entry.sessions,
                account_count: entry.accounts.len(),
                active: entry.active,
            })
            .collect();

        games.sort_by(|left, right| {
            right
                .seconds
                .partial_cmp(&left.seconds)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
        });
        games
    }

    pub fn session_history(
        &self,
        account_id: Option<&str>,
        include_active: bool,
    ) -> Vec<SessionPlaytime> {
        let now = (self.clock)();
        let mut sessions = Vec::new();

        for record in &self.state.sessions {
            if let Some(session) = self.session_view(record, now, false) {
                if account_id.is_none_or(|wanted| session.account_id == wanted) {
                    sessions.push(session);
                }
            }
        }

        if include_active {
            for record in self.state.active_sessions.values() {
                if let Some(session) = self.session_view(record, now, true) {
                    if account_id.is_none_or(|wanted| session.account_id == wanted) {
                        sessions.push(session);
                    }
                }
            }
        }

        sessions.sort_by(|left, right| {
            right
                .started_at
                .partial_cmp(&left.started_at)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| right.session_id.cmp(&left.session_id))
        });
        sessions
    }

    /// Daily totals for the last `days` days, split at local midnight.
    pub fn daily_playtime(&self, days: i64, account_id: Option<&str>) -> Vec<DailyPlaytime> {
        let days = days.clamp(1, 366);
        let now = (self.clock)();
        let offset = local_offset();
        let today = OffsetDateTime::from_unix_timestamp(now as i64)
            .unwrap_or(OffsetDateTime::UNIX_EPOCH)
            .to_offset(offset)
            .date();
        let first_day = today - Duration::days(days - 1);
        let range_start = midnight_timestamp(first_day, offset);

        let mut buckets: BTreeMap<Date, (f64, u64)> = BTreeMap::new();
        let mut day = first_day;
        while day <= today {
            buckets.insert(day, (0.0, 0));
            day += Duration::days(1);
        }

        for session in self.session_history(account_id, true) {
            let session_end = session.ended_at.unwrap_or(now).min(now);
            let mut cursor = session.started_at.max(range_start);
            if session_end <= cursor {
                continue;
            }

            while cursor < session_end {
                let current_day = OffsetDateTime::from_unix_timestamp(cursor as i64)
                    .unwrap_or(OffsetDateTime::UNIX_EPOCH)
                    .to_offset(offset)
                    .date();
                let Some(bucket) = buckets.get_mut(&current_day) else {
                    break;
                };
                let segment_end =
                    session_end.min(midnight_timestamp(current_day + Duration::days(1), offset));
                bucket.0 += (segment_end - cursor).max(0.0);
                bucket.1 += 1;
                cursor = segment_end;
            }
        }

        buckets
            .into_iter()
            .map(|(day, (seconds, sessions))| DailyPlaytime {
                day,
                seconds,
                sessions,
            })
            .collect()
    }

    /// Totals, session statistics and the amount of legacy aggregate data.
    pub fn history_statistics(&self, account_id: Option<&str>) -> PlaytimeStatistics {
        let sessions = self.session_history(account_id, true);
        let completed: f64 = sessions
            .iter()
            .filter(|session| !session.active)
            .map(|session| session.duration_seconds)
            .sum();
        let active: f64 = sessions
            .iter()
            .filter(|session| session.active)
            .map(|session| session.duration_seconds)
            .sum();

        let persisted_total = match account_id {
            Some(account_id) => self.account_seconds(account_id, false),
            None => self.global_seconds(false),
        };
        let durations: Vec<f64> = sessions
            .iter()
            .map(|session| session.duration_seconds)
            .collect();

        PlaytimeStatistics {
            total_seconds: persisted_total + active,
            detailed_seconds: completed + active,
            legacy_seconds: (persisted_total - completed).max(0.0),
            session_count: sessions.len(),
            average_session_seconds: if durations.is_empty() {
                0.0
            } else {
                durations.iter().sum::<f64>() / durations.len() as f64
            },
            longest_session_seconds: durations.iter().copied().fold(0.0, f64::max),
            first_session_at: sessions.iter().map(|session| session.started_at).fold(
                None,
                |current: Option<f64>, value| {
                    Some(current.map_or(value, |existing| existing.min(value)))
                },
            ),
            history_started_at: self.state.history_started_at,
        }
    }

    // ----------------------------------------------------------- event handling

    fn handle_start(&mut self, event: &GameProcessEvent, now: f64) -> bool {
        let Some(pid) = event.pid else {
            return false;
        };

        if let Some(existing) = self.state.active_sessions.get(&event.app_id) {
            if existing.root_pid == Some(pid) {
                return false;
            }

            let facts = self.probe_matching_process(existing);
            if facts.running {
                return false;
            }

            let ended_at = event.timestamp.max(existing.started_at);
            self.finish_session(&event.app_id, ended_at, "replaced_by_new_launch", None);
        }

        let historical = event.timestamp < self.instance_started_at - FRESH_EVENT_TOLERANCE;
        let (started_at, tracking_mode) = if historical {
            let facts = self.probe_event_process(event);
            if !facts.running {
                return false;
            }
            (now, "partial")
        } else {
            (event.timestamp, "full")
        };

        self.start_session(event, started_at, now, tracking_mode)
    }

    fn handle_end(&mut self, event: &GameProcessEvent) -> bool {
        let Some(session) = self.state.active_sessions.get(&event.app_id) else {
            return false;
        };
        if event.pid.is_none() || session.root_pid != event.pid {
            return false;
        }
        self.finish_session(
            &event.app_id,
            event.timestamp,
            "root_process_exit_log",
            event.exit_code,
        )
    }

    fn start_session(
        &mut self,
        event: &GameProcessEvent,
        started_at: f64,
        observed_at: f64,
        tracking_mode: &str,
    ) -> bool {
        let Some(account) = self.resolve_active_account() else {
            return false;
        };
        if account.steam_id.is_empty() {
            return false;
        }

        let facts = self.probe_event_process(event);
        let game_name = self.catalog.name_for(&event.app_id);

        let account_state = self
            .state
            .accounts
            .entry(account.steam_id.clone())
            .or_default();
        account_state.account_name = account.account_name.clone();
        account_state.persona_name = account.persona_name.clone();

        self.state.active_sessions.insert(
            event.app_id.clone(),
            SessionRecord {
                session_id: new_session_id(),
                account_id: account.steam_id,
                account_name: account.account_name,
                persona_name: account.persona_name,
                app_id: event.app_id.clone(),
                game_name,
                root_pid: event.pid,
                process_create_time: if facts.running {
                    facts.start_time
                } else {
                    None
                },
                launch_timestamp: event.timestamp,
                detected_at: observed_at,
                started_at,
                launch_command: event.command.clone(),
                last_seen_at: Some(observed_at),
                tracking_mode: tracking_mode.to_string(),
                ..SessionRecord::default()
            },
        );
        true
    }

    fn finish_session(
        &mut self,
        app_id: &str,
        ended_at: f64,
        end_reason: &str,
        exit_code: Option<i32>,
    ) -> bool {
        let Some(session) = self.state.active_sessions.remove(app_id) else {
            return false;
        };

        let started_at = if session.started_at > 0.0 {
            session.started_at
        } else {
            ended_at
        };
        let ended_at = ended_at.max(started_at);
        let seconds = ended_at - started_at;
        let game_name = self.game_name(&session);

        if let Some(account) = self.state.accounts.get_mut(&session.account_id) {
            account.total_seconds += seconds;
            let game = account.games.entry(session.app_id.clone()).or_default();
            game.name = game_name.clone();
            game.total_seconds += seconds;
            game.sessions += 1;
            game.last_played_at = Some(ended_at);
        }

        let session_id = if session.session_id.is_empty() {
            new_session_id()
        } else {
            session.session_id.clone()
        };

        if !self
            .state
            .sessions
            .iter()
            .any(|record| record.session_id == session_id)
        {
            self.state.sessions.push(SessionRecord {
                root_pid: session.root_pid,
                process_create_time: session.process_create_time,
                ended_at: Some(ended_at),
                duration_seconds: Some(seconds),
                tracked_started_at: Some(started_at),
                session_id,
                end_reason: end_reason.to_string(),
                exit_code,
                ..session
            });
        }

        self.loaded_active_app_ids.remove(app_id);
        true
    }

    /// Closes sessions whose process disappeared while the app was not running.
    fn reconcile_processes(&mut self, now: f64) -> bool {
        let mut changed = false;
        let app_ids: Vec<String> = self.state.active_sessions.keys().cloned().collect();

        for app_id in app_ids {
            let Some(session) = self.state.active_sessions.get(&app_id) else {
                continue;
            };
            let facts = self.probe_matching_process(session);
            if facts.running {
                continue;
            }

            let loaded = self.loaded_active_app_ids.contains(&app_id);
            let ended_at = if loaded {
                let last_seen = session.last_seen_at.unwrap_or(now);
                now.min(last_seen + FRESH_EVENT_TOLERANCE)
            } else {
                now
            };
            let reason = if loaded {
                "process_missing_after_restart"
            } else {
                "process_exit"
            };
            changed |= self.finish_session(&app_id, ended_at, reason, None);
        }

        changed
    }

    fn resolve_active_account(&self) -> Option<SteamAccount> {
        (self.account_provider)()
    }

    fn game_name(&self, session: &SessionRecord) -> String {
        if !session.game_name.is_empty() {
            return session.game_name.clone();
        }
        self.catalog.display_name_for(&session.app_id)
    }

    fn probe_event_process(&self, event: &GameProcessEvent) -> ProcessFacts {
        let Some(pid) = event.pid else {
            return ProcessFacts {
                running: false,
                start_time: None,
            };
        };

        let facts = (self.probe)(pid);
        if !facts.running {
            return facts;
        }

        // Steam wrote the launch event long before this instance started, so a
        // process that exists now with a very different start time is a reused
        // pid and must not be attributed to this session.
        match facts.start_time {
            Some(created_at) if (created_at - event.timestamp).abs() > 180.0 => ProcessFacts {
                running: false,
                start_time: facts.start_time,
            },
            _ => facts,
        }
    }

    fn probe_matching_process(&self, session: &SessionRecord) -> ProcessFacts {
        let Some(pid) = session.root_pid else {
            return ProcessFacts {
                running: false,
                start_time: None,
            };
        };

        let facts = (self.probe)(pid);
        if !facts.running {
            return facts;
        }

        match (facts.start_time, session.process_create_time) {
            (Some(created_at), Some(expected)) if (created_at - expected).abs() > 2.0 => {
                ProcessFacts {
                    running: false,
                    start_time: facts.start_time,
                }
            }
            _ => facts,
        }
    }

    fn session_view(
        &self,
        record: &SessionRecord,
        now: f64,
        active: bool,
    ) -> Option<SessionPlaytime> {
        if record.account_id.is_empty() || record.app_id.is_empty() {
            return None;
        }

        let started_at = record
            .tracked_started_at
            .unwrap_or(if record.started_at > 0.0 {
                record.started_at
            } else {
                now
            });
        let ended_at = if active { None } else { record.ended_at };
        let duration_seconds = match (record.duration_seconds, ended_at) {
            (Some(duration), _) => duration,
            (None, Some(ended_at)) => (ended_at - started_at).max(0.0),
            (None, None) => (now - started_at).max(0.0),
        };

        Some(SessionPlaytime {
            session_id: record.session_id.clone(),
            account_id: record.account_id.clone(),
            account_name: record.account_name.clone(),
            persona_name: record.persona_name.clone(),
            app_id: record.app_id.clone(),
            game_name: self.game_name(record),
            launch_timestamp: if record.launch_timestamp > 0.0 {
                record.launch_timestamp
            } else {
                started_at
            },
            started_at,
            ended_at,
            duration_seconds,
            active,
            tracking_mode: if record.tracking_mode.is_empty() {
                "partial".to_string()
            } else {
                record.tracking_mode.clone()
            },
            end_reason: if active {
                String::new()
            } else if record.end_reason.is_empty() {
                "unknown".to_string()
            } else {
                record.end_reason.clone()
            },
            exit_code: record.exit_code,
            root_pid: record.root_pid,
            process_create_time: record.process_create_time,
        })
    }

    // --------------------------------------------------------------- log tail

    fn has_valid_source(&self) -> bool {
        self.state.source.contains_key("offset") && self.state.source.contains_key("device")
    }

    fn initialize_at_log_end(&mut self) {
        match std::fs::metadata(&self.log_path) {
            Ok(metadata) => {
                let (device, inode, modified) = file_identity(&metadata);
                self.state.source = source_map(device, inode, metadata.len(), modified);
            }
            Err(_) => {
                self.state.source = source_map(0, 0, 0, 0.0);
            }
        }
    }

    /// Scans the tail of the log for games that are already running so a
    /// launch that happened before the app started is still counted.
    fn discover_games_already_running(&mut self) {
        let Ok(mut handle) = File::open(&self.log_path) else {
            return;
        };

        let Ok(metadata) = handle.metadata() else {
            return;
        };
        let offset = metadata.len().saturating_sub(DISCOVERY_READ_SIZE);
        if handle.seek(SeekFrom::Start(offset)).is_err() {
            return;
        }

        let mut data = String::new();
        if handle.read_to_string(&mut data).is_err() {
            return;
        }
        let mut lines: Vec<&str> = data.lines().collect();
        if offset > 0 && !lines.is_empty() {
            lines.remove(0);
        }

        let mut candidates: HashMap<String, GameProcessEvent> = HashMap::new();
        for line in lines {
            let Some(event) = parse_gameprocess_line(line) else {
                if line.contains("Client version:") {
                    candidates.clear();
                }
                continue;
            };
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

    fn read_new_lines(&mut self) -> (Vec<String>, bool) {
        let Ok(mut handle) = File::open(&self.log_path) else {
            return (Vec::new(), false);
        };
        let Ok(metadata) = handle.metadata() else {
            return (Vec::new(), false);
        };

        let (device, inode, modified) = file_identity(&metadata);
        let same_file = self
            .state
            .source
            .get("device")
            .and_then(Value::as_u64)
            .unwrap_or(u64::MAX)
            == device
            && self
                .state
                .source
                .get("inode")
                .and_then(Value::as_u64)
                .unwrap_or(u64::MAX)
                == inode;

        let mut offset = if same_file {
            self.state
                .source
                .get("offset")
                .and_then(Value::as_u64)
                .unwrap_or(0)
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
                    self.state.source.get("offset").and_then(Value::as_u64) != Some(new_offset);
                self.state.source = source_map(device, inode, new_offset, modified);
                String::from_utf8_lossy(consumed).to_string()
            }
            None => {
                if source_changed {
                    self.state.source = source_map(device, inode, offset, modified);
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

    fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.state).map_err(|error| {
            SteamError::config(format!("Unable to encode playtime data: {error}"))
        })?;
        write_atomic(&self.state_path, &json).map_err(|error| {
            SteamError::io(
                format!("Unable to write {}", self.state_path.display()),
                error,
            )
        })
    }
}

fn source_map(device: u64, inode: u64, offset: u64, modified: f64) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert("device".to_string(), Value::from(device));
    map.insert("inode".to_string(), Value::from(inode));
    map.insert("offset".to_string(), Value::from(offset));
    if modified > 0.0 {
        map.insert("modified".to_string(), Value::from(modified));
    }
    map
}

/// Unix timestamp of the local midnight starting `day`.
fn midnight_timestamp(day: Date, offset: time::UtcOffset) -> f64 {
    day.midnight().assume_offset(offset).unix_timestamp() as f64
}

#[cfg(unix)]
fn file_identity(metadata: &std::fs::Metadata) -> (u64, u64, f64) {
    use std::os::unix::fs::MetadataExt;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs_f64())
        .unwrap_or(0.0);
    (metadata.dev(), metadata.ino(), modified)
}

#[cfg(not(unix))]
fn file_identity(metadata: &std::fs::Metadata) -> (u64, u64, f64) {
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs_f64())
        .unwrap_or(0.0);
    (0, 0, modified)
}

fn load_state(path: &Path, now: f64) -> (PlaytimeState, bool) {
    let Ok(text) = std::fs::read_to_string(path) else {
        return (PlaytimeState::fresh(now), false);
    };
    match serde_json::from_str::<PlaytimeState>(&text) {
        Ok(state) => (state, true),
        Err(_) => (PlaytimeState::fresh(now), false),
    }
}

fn migrate_state(state: &mut PlaytimeState, now: f64) -> bool {
    let mut migrated = false;
    if state.version < CURRENT_STATE_VERSION {
        state.version = CURRENT_STATE_VERSION;
        state.history_started_at = now;
        migrated = true;
    }
    if state.tracking_started_at <= 0.0 {
        state.tracking_started_at = now;
    }
    if state.history_started_at <= 0.0 {
        state.history_started_at = now;
        migrated = true;
    }
    migrated
}

/// Unique session id: timestamp plus a per-process counter.
fn new_session_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("{nanos:032x}{count:08x}")
}

/// Current unix time, used when the caller does not inject a clock.
pub fn system_clock() -> ClockRef {
    Arc::new(now_unix)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::InstallKind;
    use std::sync::Mutex;

    struct TestClock(Mutex<f64>);

    impl TestClock {
        fn new(value: f64) -> Arc<Self> {
            Arc::new(Self(Mutex::new(value)))
        }

        fn set(&self, value: f64) {
            *self.0.lock().unwrap() = value;
        }

        fn reference(self: &Arc<Self>) -> ClockRef {
            let clock = Arc::clone(self);
            Arc::new(move || *clock.0.lock().unwrap())
        }
    }

    fn timestamp(value: f64) -> String {
        let local = OffsetDateTime::from_unix_timestamp(value as i64)
            .unwrap_or(OffsetDateTime::UNIX_EPOCH)
            .to_offset(local_offset());
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            local.year(),
            local.month() as u8,
            local.day(),
            local.hour(),
            local.minute(),
            local.second()
        )
    }

    fn start_line(value: f64, app_id: &str, pid: u32) -> String {
        format!(
            "[{}] AppID {} adding PID {} as a tracked process \"ENV_VAR=\"quoted value\" steam-launch-wrapper SteamLaunch AppId={} -- game\"",
            timestamp(value),
            app_id,
            pid,
            app_id
        )
    }

    fn end_line(value: f64, app_id: &str, pid: u32) -> String {
        format!(
            "[{}] AppID {} no longer tracking PID {}, exit code 0",
            timestamp(value),
            app_id,
            pid
        )
    }

    struct Harness {
        directory: PathBuf,
        log_path: PathBuf,
        clock: Arc<TestClock>,
        running: Arc<Mutex<HashMap<u32, ProcessFacts>>>,
        tracker: PlaytimeTracker,
    }

    impl Harness {
        fn new(start_time: f64, with_log: bool) -> Self {
            let directory = std::env::temp_dir().join(format!(
                "steam-core-playtime-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|duration| duration.as_nanos())
                    .unwrap_or(0)
            ));
            let _ = std::fs::remove_dir_all(&directory);
            std::fs::create_dir_all(directory.join("logs")).expect("logs dir");
            std::fs::create_dir_all(directory.join("config")).expect("config dir");
            let log_path = directory.join("logs/gameprocess_log.txt");
            if with_log {
                std::fs::write(&log_path, "").expect("empty log");
            }

            let clock = TestClock::new(start_time);
            let running: Arc<Mutex<HashMap<u32, ProcessFacts>>> =
                Arc::new(Mutex::new(HashMap::new()));

            let installation = SteamInstallation {
                kind: InstallKind::Native,
                root: directory.clone(),
                config_dir: directory.join("config"),
                loginusers_path: directory.join("config/loginusers.vdf"),
                registry_path: directory.join("registry.vdf"),
                userdata_dir: directory.join("userdata"),
                launch_command: vec!["steam".to_string()],
                prefix: None,
            };

            let probe_state = Arc::clone(&running);
            let probe: ProbeRef = Arc::new(move |pid| {
                probe_state
                    .lock()
                    .unwrap()
                    .get(&pid)
                    .copied()
                    .unwrap_or(ProcessFacts {
                        running: false,
                        start_time: None,
                    })
            });

            let account_provider: AccountProvider = Arc::new(|| {
                Some(SteamAccount {
                    steam_id: "76561198000000001".to_string(),
                    account_name: "local-test".to_string(),
                    persona_name: "Local Test".to_string(),
                    avatar_path: None,
                    most_recent: true,
                    remember_password: true,
                    allow_auto_login: true,
                    wants_offline_mode: false,
                    skip_offline_mode_warning: false,
                    timestamp: None,
                })
            });

            let tracker =
                PlaytimeTracker::new(&installation, clock.reference(), probe, account_provider);

            Self {
                directory,
                log_path,
                clock,
                running,
                tracker,
            }
        }

        fn append(&self, line: &str) {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(&self.log_path)
                .expect("open log");
            writeln!(file, "{line}").expect("append log");
        }

        fn set_process(&self, pid: u32, facts: ProcessFacts) {
            self.running.lock().unwrap().insert(pid, facts);
        }
    }

    impl Drop for Harness {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.directory);
        }
    }

    #[test]
    fn parser_accepts_commands_with_nested_quotes() {
        let event =
            parse_gameprocess_line(&start_line(1_700_000_000.0, "570", 99)).expect("start event");
        assert_eq!(event.kind, EventKind::Start);
        assert_eq!(event.app_id, "570");
        assert_eq!(event.pid, Some(99));
        assert!(event.command.contains("SteamLaunch AppId=570"));

        let end = parse_gameprocess_line(&end_line(1_700_000_100.0, "570", 99)).expect("end event");
        assert_eq!(end.kind, EventKind::End);
        assert_eq!(end.pid, Some(99));
        assert_eq!(end.exit_code, Some(0));

        let crash = parse_gameprocess_line(
            "[2026-01-10 12:00:00] AppID 570 no longer tracking PID 99, exit code 139",
        )
        .expect("crash event");
        assert_eq!(crash.exit_code, Some(139));

        let no_code =
            parse_gameprocess_line("[2026-01-10 12:00:00] AppID 570 no longer tracking PID 99")
                .expect("end event without a code");
        assert_eq!(no_code.exit_code, None);

        let remove = parse_gameprocess_line("[2026-01-10 12:00:00] Remove 570 from running list")
            .expect("remove event");
        assert_eq!(remove.kind, EventKind::Remove);

        assert!(parse_gameprocess_line("[2026-01-10 12:00:00] nothing to see").is_none());
    }

    #[test]
    fn existing_history_is_not_imported() {
        let mut harness = Harness::new(1_700_000_000.0, true);
        harness.append(&start_line(1_600_000_000.0, "570", 42));
        harness.append(&end_line(1_600_000_100.0, "570", 42));
        harness.set_process(
            42,
            ProcessFacts {
                running: false,
                start_time: None,
            },
        );

        harness.tracker.poll();

        assert_eq!(harness.tracker.session_history(None, true).len(), 0);
        assert_eq!(harness.tracker.global_seconds(false), 0.0);
    }

    #[test]
    fn live_and_completed_sessions_are_counted_for_the_active_account() {
        let mut harness = Harness::new(1_700_000_000.0, true);
        harness.set_process(
            4242,
            ProcessFacts {
                running: true,
                start_time: Some(1_700_000_100.0),
            },
        );

        harness.append(&start_line(1_700_000_100.0, "570", 4242));
        harness.tracker.poll();
        assert_eq!(harness.tracker.active_games().len(), 1);

        let later = 1_700_000_700.0;
        harness.clock.set(later);
        harness.tracker.poll();
        assert!((harness.tracker.global_seconds(true) - 600.0).abs() < 1.0);

        harness.append(&end_line(later, "570", 4242));
        harness.set_process(
            4242,
            ProcessFacts {
                running: false,
                start_time: None,
            },
        );
        harness.tracker.poll();

        assert!(harness.tracker.active_games().is_empty());
        let history = harness.tracker.session_history(None, true);
        assert_eq!(history.len(), 1);
        assert!((history[0].duration_seconds - 600.0).abs() < 1.0);
        assert_eq!(history[0].end_reason, "root_process_exit_log");
        assert_eq!(history[0].exit_code, Some(0));
        assert!((harness.tracker.account_seconds("76561198000000001", true) - 600.0).abs() < 1.0);

        let games = harness.tracker.account_games("76561198000000001");
        assert_eq!(games.len(), 1);

        harness.tracker.close();
    }

    #[test]
    fn runs_left_over_from_a_previous_instance_are_closed_at_restart() {
        let mut harness = Harness::new(1_700_000_000.0, true);
        harness.set_process(
            777,
            ProcessFacts {
                running: true,
                start_time: Some(1_700_000_100.0),
            },
        );
        harness.append(&start_line(1_700_000_100.0, "730", 777));
        harness.tracker.poll();
        let saved = harness.tracker.state_path().to_path_buf();
        harness.tracker.close();

        // Simulate a new application start: the process is gone but the state
        // still lists the session as active.
        let second = Harness::new(1_700_000_900.0, true);
        std::fs::copy(&saved, second.tracker.state_path()).expect("copy state");
        let installation_state =
            std::fs::read_to_string(second.tracker.state_path()).expect("read");
        assert!(installation_state.contains("active_sessions"));

        let mut tracker = PlaytimeTracker::new(
            &SteamInstallation {
                kind: InstallKind::Native,
                root: second.directory.clone(),
                config_dir: second.directory.join("config"),
                loginusers_path: second.directory.join("config/loginusers.vdf"),
                registry_path: second.directory.join("registry.vdf"),
                userdata_dir: second.directory.join("userdata"),
                launch_command: vec!["steam".to_string()],
                prefix: None,
            },
            second.clock.reference(),
            {
                let facts = ProcessFacts {
                    running: false,
                    start_time: None,
                };
                Arc::new(move |_pid| facts)
            },
            Arc::new(|| None),
        );

        tracker.poll();
        let history = tracker.session_history(None, true);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].end_reason, "process_missing_after_restart");
    }

    #[test]
    fn daily_history_splits_a_session_at_local_midnight() {
        let offset = local_offset();
        let day = OffsetDateTime::from_unix_timestamp(1_700_000_000)
            .unwrap()
            .to_offset(offset)
            .date();
        let midnight = midnight_timestamp(day + Duration::days(1), offset);

        let mut harness = Harness::new(midnight - 3600.0, true);
        harness.set_process(
            11,
            ProcessFacts {
                running: true,
                start_time: Some(midnight - 3600.0),
            },
        );
        harness.append(&start_line(midnight - 3600.0, "570", 11));
        harness.clock.set(midnight + 1800.0);
        harness.tracker.poll();
        harness.append(&end_line(midnight + 1800.0, "570", 11));
        harness.set_process(
            11,
            ProcessFacts {
                running: false,
                start_time: None,
            },
        );
        harness.clock.set(midnight + 1900.0);
        harness.tracker.poll();

        let daily = harness.tracker.daily_playtime(7, None);
        let first = daily
            .iter()
            .find(|entry| entry.day == day)
            .expect("first day bucket");
        let second = daily
            .iter()
            .find(|entry| entry.day == day + Duration::days(1))
            .expect("second day bucket");

        assert!((first.seconds - 3600.0).abs() < 1.0);
        assert!((second.seconds - 1800.0).abs() < 1.0);
    }

    #[test]
    fn machine_breakdown_aggregates_games_across_accounts() {
        let mut harness = Harness::new(1_700_000_000.0, true);
        harness.set_process(
            5,
            ProcessFacts {
                running: true,
                start_time: Some(1_700_000_000.0),
            },
        );
        harness.append(&start_line(1_700_000_000.0, "570", 5));
        harness.clock.set(1_700_000_300.0);
        harness.tracker.poll();
        harness.append(&end_line(1_700_000_300.0, "570", 5));
        harness.set_process(
            5,
            ProcessFacts {
                running: false,
                start_time: None,
            },
        );
        harness.tracker.poll();

        let games = harness.tracker.machine_games();
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].app_id, "570");
        assert_eq!(games[0].account_count, 1);
        assert_eq!(games[0].sessions, 1);
        assert!((games[0].seconds - 300.0).abs() < 1.0);

        let statistics = harness.tracker.history_statistics(None);
        assert_eq!(statistics.session_count, 1);
    }
}
