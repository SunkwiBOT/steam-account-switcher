//! Session lifecycle: turning Steam's process events into started and finished
//! sessions, and reconciling them against the processes still alive.

use super::{new_session_id, PlaytimeTracker};
use crate::model::SteamAccount;
use crate::playtime::log::{probe_event_process, GameProcessEvent};
use crate::playtime::model::SessionRecord;
use crate::playtime::FRESH_EVENT_TOLERANCE;
use crate::processes::ProcessFacts;

impl PlaytimeTracker {
    pub(super) fn handle_start(&mut self, event: &GameProcessEvent, now: f64) -> bool {
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

    pub(super) fn handle_end(&mut self, event: &GameProcessEvent) -> bool {
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

    pub(super) fn start_session(
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

    pub(super) fn finish_session(
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

    pub(super) fn reconcile_processes(&mut self, now: f64) -> bool {
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

    pub(super) fn resolve_active_account(&self) -> Option<SteamAccount> {
        (self.account_provider)()
    }

    pub(super) fn game_name(&self, session: &SessionRecord) -> String {
        if !session.game_name.is_empty() {
            return session.game_name.clone();
        }
        self.catalog.display_name_for(&session.app_id)
    }

    pub(super) fn probe_event_process(&self, event: &GameProcessEvent) -> ProcessFacts {
        probe_event_process(event, self.probe.as_ref())
    }

    pub(super) fn probe_matching_process(&self, session: &SessionRecord) -> ProcessFacts {
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
}
