//! Read-only views over the recorded playtime: per-account and per-game totals,
//! session history, daily buckets and statistics.

use super::PlaytimeTracker;
use crate::playtime::model::{
    AccountPlaytime, ActiveGame, DailyPlaytime, GamePlaytime, MachineGamePlaytime,
    PlaytimeStatistics, SessionPlaytime, SessionRecord,
};
use crate::util::{local_date, local_midnight};
use std::collections::{BTreeMap, HashMap, HashSet};
use time::{Date, Duration};

impl PlaytimeTracker {
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
        let today = local_date(now as i64);
        let first_day = today - Duration::days(days - 1);
        let range_start = midnight_timestamp(first_day);

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
                let current_day = local_date(cursor as i64);
                let Some(bucket) = buckets.get_mut(&current_day) else {
                    break;
                };
                let segment_end =
                    session_end.min(midnight_timestamp(current_day + Duration::days(1)));
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

    pub(super) fn session_view(
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
}

pub(super) fn midnight_timestamp(day: Date) -> f64 {
    local_midnight(day) as f64
}
