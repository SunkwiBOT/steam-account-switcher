use super::*;

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
    /// Set when the stored history could not be read. Saving is refused then, so
    /// an unreadable database never gets overwritten with an empty history.
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
        probe_event_process(event, self.probe.as_ref())
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

    fn has_valid_source(&self) -> bool {
        self.state.source.known
    }

    fn initialize_at_log_end(&mut self) {
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
    fn discover_games_already_running(&mut self) {
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

    fn read_new_lines(&mut self) -> (Vec<String>, bool) {
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
        // A rotated log can reuse the device and inode (Windows always reports 0),
        // so a changed head also means a different file.
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

pub(super) fn midnight_timestamp(day: Date, offset: time::UtcOffset) -> f64 {
    day.midnight().assume_offset(offset).unix_timestamp() as f64
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
