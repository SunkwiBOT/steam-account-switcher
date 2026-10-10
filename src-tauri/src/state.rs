use std::path::Path;
mod folders;
mod installations;
mod runtime;
mod vault;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;
use steam_core::error_code::ErrorCode;

use steam_core::accounts::SteamManager;
use steam_core::authenticator::Authenticator;
use steam_core::metadata::RankStore;
use steam_core::model::{SteamAccount, SteamInstallation};
use steam_core::pathdetect;
use steam_core::playtime::{system_clock, AccountProvider, PlaytimeTracker, ProbeRef};
use steam_core::processes::ProcessMonitor;
use steam_core::storage::Database;

use crate::settings::{Settings, SettingsStore};

/// Writes the settings, putting `previous` back in memory when the write fails
/// so the interface never shows a change that is not on disk.
fn save_or_restore(settings: &mut SettingsStore, previous: Settings) -> Result<(), String> {
    settings.save().inspect_err(|_| {
        *settings.data_mut() = previous;
    })
}

pub fn root_key(installation: &SteamInstallation) -> String {
    steam_core::model::resolve(&installation.root)
        .display()
        .to_string()
}

pub fn active_account_for(manager: &SteamManager) -> Option<SteamAccount> {
    manager.current_account().ok().flatten()
}

pub struct Inner {
    pub detected: Vec<SteamInstallation>,
    pub custom: Vec<SteamInstallation>,
    pub installations: Vec<SteamInstallation>,
    pub selected: usize,
    pub manager: Option<Arc<SteamManager>>,
    pub accounts: Vec<SteamAccount>,
    pub auto_login_user: String,
    pub ranks: Option<RankStore>,
    pub tracker: Option<Arc<Mutex<PlaytimeTracker>>>,
    pub monitor: Arc<Mutex<ProcessMonitor>>,
    pub masked: bool,
    pub steam_ids_visible: bool,
    last_error: String,
    pub unlocked_at: Option<Instant>,
    pub settings: SettingsStore,
}

pub struct AppState {
    pub database: Database,
    inner: Mutex<Inner>,
    snapshot_revision: AtomicU64,
    /// Steam Guard secrets and the interactive login/link sessions. Kept in its
    /// own mutex so a long network call never blocks the account list.
    authenticator: Arc<Mutex<Authenticator>>,
    window_active: AtomicBool,
    tray_available: AtomicBool,
    /// Serializes operations that stop Steam or change its login files.
    switching: Arc<AtomicBool>,
}

pub struct SteamOperationGuard(Arc<AtomicBool>);

impl SteamOperationGuard {
    fn acquire(busy: Arc<AtomicBool>) -> Result<Self, String> {
        busy.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .map(|_| Self(busy))
            .map_err(|_| ErrorCode::SteamOperationBusy.as_str().to_string())
    }
}

impl Drop for SteamOperationGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

impl AppState {
    pub fn new() -> Self {
        let database = open_database();
        let settings = SettingsStore::load(database.clone());
        let masked = !settings.data().show_account_names;
        let steam_ids_visible = settings.data().show_steam_ids;
        let state = Self {
            database: database.clone(),
            inner: Mutex::new(Inner {
                detected: Vec::new(),
                custom: Vec::new(),
                installations: Vec::new(),
                selected: 0,
                manager: None,
                accounts: Vec::new(),
                auto_login_user: String::new(),
                ranks: Some(RankStore::load(database.clone())),
                tracker: None,
                monitor: Arc::new(Mutex::new(ProcessMonitor::new())),
                masked,
                steam_ids_visible,
                last_error: String::new(),
                unlocked_at: None,
                settings,
            }),
            snapshot_revision: AtomicU64::new(0),
            authenticator: Arc::new(Mutex::new(Authenticator::new(database))),
            window_active: AtomicBool::new(true),
            tray_available: AtomicBool::new(false),
            switching: Arc::new(AtomicBool::new(false)),
        };
        state.reload_installations();
        state
    }

    pub fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|error| error.into_inner())
    }
}

fn same_path(left: &str, right: &str) -> bool {
    steam_core::model::resolve(Path::new(left)) == steam_core::model::resolve(Path::new(right))
}

pub fn key_for_path(path: &str) -> String {
    steam_core::model::resolve(Path::new(path))
        .display()
        .to_string()
}

/// The app cannot run without its database: a session that could not save a new
/// Steam Guard secret would lose it on restart, so startup stops instead.
fn open_database() -> Database {
    let opened = Database::open(&crate::settings::database_path())
        .expect("the application database must open");
    if opened.moved_aside.is_some() {
        crate::logging::write(
            "action=open_database status=recovered category=database_unreadable backup=steam-account-switcher.db.bak",
        );
    }
    opened.database
}

pub(crate) fn build_tracker(
    manager: &Arc<SteamManager>,
    database: Database,
    monitor: Arc<Mutex<ProcessMonitor>>,
    enabled: bool,
) -> PlaytimeTracker {
    let probe: ProbeRef = Arc::new(move |pid| {
        let mut monitor = monitor.lock().unwrap_or_else(|error| error.into_inner());
        monitor.process_facts(pid)
    });

    let provider: AccountProvider = {
        let manager = Arc::clone(manager);
        Arc::new(move || active_account_for(&manager))
    };

    PlaytimeTracker::with_tracking(
        &manager.installation,
        database,
        system_clock(),
        probe,
        provider,
        enabled,
    )
}
