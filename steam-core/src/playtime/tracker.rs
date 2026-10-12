use super::log::{parse_gameprocess_line, EventKind};
use super::model::{LogCursor, PlaytimeState};
use super::store::{load_state, save_state, SaveScope};
use super::{installation_key, AccountProvider, ClockRef, ProbeRef, CHECKPOINT_INTERVAL};
use crate::error::{Result, SteamError};
use crate::games::SteamGameCatalog;
use crate::model::SteamInstallation;
use crate::storage::Database;
use crate::util::now_unix;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod ingest;
mod reports;
mod sessions;

pub struct PlaytimeTracker {
    enabled: bool,
    database: Database,
    installation: String,
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
    /// Set when the in-memory state has not been written yet; the next poll retries.
    dirty: bool,
    /// ! Set when the stored history could not be read: saving is refused so an
    /// ! unreadable database is never overwritten with an empty history.
    blocked: bool,
    /// Finished sessions already in the database, so a save writes only new ones.
    stored_sessions: usize,
}

impl PlaytimeTracker {
    pub fn new(
        installation: &SteamInstallation,
        database: Database,
        clock: ClockRef,
        probe: ProbeRef,
        account_provider: AccountProvider,
    ) -> Self {
        Self::with_tracking(installation, database, clock, probe, account_provider, true)
    }

    pub fn with_tracking(
        installation: &SteamInstallation,
        database: Database,
        clock: ClockRef,
        probe: ProbeRef,
        account_provider: AccountProvider,
        enabled: bool,
    ) -> Self {
        let instance_started_at = clock();
        let log_path = installation.root.join("logs/gameprocess_log.txt");
        let key = installation_key(installation);

        let (state, loaded, blocked) = match load_state(&database, &key) {
            Ok(Some(state)) => (state, true, false),
            Ok(None) => (PlaytimeState::fresh(instance_started_at), false, false),
            Err(_) => (PlaytimeState::fresh(instance_started_at), false, true),
        };
        let loaded_active_app_ids = if loaded {
            state.active_sessions.keys().cloned().collect()
        } else {
            HashSet::new()
        };

        let stored_sessions = state.sessions.len();
        let mut tracker = Self {
            enabled,
            database,
            installation: key,
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
            dirty: false,
            blocked,
            stored_sessions,
        };

        if !enabled {
            let sessions: Vec<_> = tracker
                .state
                .active_sessions
                .iter()
                .map(|(id, session)| {
                    (
                        id.clone(),
                        session.last_seen_at.unwrap_or(session.started_at),
                    )
                })
                .collect();
            for (id, end) in sessions {
                tracker.finish_session(&id, end, "tracking_disabled", None);
            }
            tracker.loaded_active_app_ids.clear();
        } else if !loaded || !tracker.has_valid_source() {
            tracker.initialize_at_log_end();
            tracker.discover_games_already_running();
            // A failed save leaves `dirty` set, so the first poll retries and reports it.
            let _ = tracker.persist();
        } else if tracker.state.active_sessions.is_empty() {
            tracker.discover_games_already_running();
            if !tracker.state.active_sessions.is_empty() {
                let _ = tracker.persist();
            }
        }

        tracker
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()> {
        if self.enabled == enabled {
            return Ok(());
        }
        if self.enabled {
            self.poll()?;
        }
        self.persist_transition(|tracker| {
            let now = (tracker.clock)();
            let ids: Vec<_> = tracker.state.active_sessions.keys().cloned().collect();
            for id in ids {
                tracker.finish_session(&id, now, "tracking_disabled", None);
            }
            tracker.enabled = enabled;
            tracker.instance_started_at = now;
            tracker.last_checkpoint = now;
            tracker.pending.clear();
            tracker.loaded_active_app_ids.clear();
            tracker.initialize_at_log_end();
            if enabled {
                tracker.discover_games_already_running();
            }
        })
    }

    pub fn clear_history(&mut self) -> Result<()> {
        self.clear_with(SaveScope::ThisInstallation)
    }

    /// Erases the history of every installation in the same transaction that stores
    /// this tracker's reset state, so a failure leaves all of it in place.
    pub fn clear_all_history(&mut self) -> Result<()> {
        self.clear_with(SaveScope::EveryInstallation)
    }

    fn clear_with(&mut self, scope: SaveScope) -> Result<()> {
        self.persist_transition_scoped(scope, |tracker| {
            let now = (tracker.clock)();
            tracker.state = PlaytimeState::fresh(now);
            tracker.instance_started_at = now;
            tracker.last_checkpoint = now;
            tracker.pending.clear();
            tracker.loaded_active_app_ids.clear();
            tracker.initialize_at_log_end();
            if tracker.enabled {
                tracker.discover_games_already_running();
            }
        })
    }

    fn persist_transition(&mut self, action: impl FnOnce(&mut Self)) -> Result<()> {
        self.persist_transition_scoped(SaveScope::ThisInstallation, action)
    }

    fn persist_transition_scoped(
        &mut self,
        scope: SaveScope,
        action: impl FnOnce(&mut Self),
    ) -> Result<()> {
        let previous = (
            self.state.clone(),
            self.enabled,
            self.instance_started_at,
            self.last_checkpoint,
            self.pending.clone(),
            self.loaded_active_app_ids.clone(),
            self.dirty,
        );
        action(self);
        if let Err(error) = self.persist_scoped(scope) {
            (
                self.state,
                self.enabled,
                self.instance_started_at,
                self.last_checkpoint,
                self.pending,
                self.loaded_active_app_ids,
                self.dirty,
            ) = previous;
            return Err(error);
        }
        Ok(())
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

    /// Returns whether the state changed. A failed write is an error, never a
    /// silent `false`, and is retried by the next poll.
    pub fn poll(&mut self) -> Result<bool> {
        if !self.enabled {
            return Ok(false);
        }
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

        if changed || self.dirty {
            self.persist()?;
        }
        Ok(changed)
    }

    pub fn close(&mut self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        self.poll()?;
        let now = (self.clock)();
        for session in self.state.active_sessions.values_mut() {
            session.last_seen_at = Some(now);
        }
        self.persist()
    }

    fn persist(&mut self) -> Result<()> {
        self.persist_scoped(SaveScope::ThisInstallation)
    }

    fn persist_scoped(&mut self, scope: SaveScope) -> Result<()> {
        let result = self.save(scope);
        self.dirty = result.is_err();
        if result.is_ok() {
            self.stored_sessions = self.state.sessions.len();
        }
        result
    }

    fn save(&self, scope: SaveScope) -> Result<()> {
        if self.blocked {
            return Err(SteamError::io(
                "Save playtime history",
                std::io::Error::other("stored history could not be read"),
            ));
        }
        save_state(
            &self.database,
            &self.installation,
            &self.state,
            scope,
            self.stored_sessions,
        )
    }
}

pub(super) fn log_cursor(
    device: u64,
    inode: u64,
    offset: u64,
    modified: f64,
    head: &[u8],
) -> LogCursor {
    LogCursor {
        known: true,
        device,
        inode,
        modified,
        offset,
        head: head.to_vec(),
    }
}

pub(super) fn new_session_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("{nanos:032x}{count:08x}")
}

pub fn system_clock() -> ClockRef {
    Arc::new(now_unix)
}
