use std::path::Path;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
mod folders;
mod installations;
mod runtime;
mod vault;

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;
use steam_core::error_code::ErrorCode;

use steam_core::accounts::SteamManager;
use steam_core::authenticator::Authenticator;
use steam_core::metadata::RankStore;
use steam_core::model::{same_path, SteamAccount, SteamInstallation};
use steam_core::playtime::{system_clock, AccountProvider, PlaytimeTracker, ProbeRef};
use steam_core::processes::ProcessMonitor;
use steam_core::storage::Database;

use crate::settings::{Settings, SettingsStore};

// ! Writes the settings, putting `previous` back in memory when the write fails so the interface never shows a change that is not on disk.
pub(crate) fn save_or_restore(
    settings: &mut SettingsStore,
    previous: Settings,
) -> Result<(), String> {
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
    manager.most_recently_played_account().ok().flatten()
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
    pub accounts_error: Option<ErrorCode>,
    pub unlocked_at: Option<Instant>,
    pub settings: SettingsStore,
}

pub struct AppState {
    pub database: Database,
    inner: Mutex<Inner>,
    snapshot_revision: AtomicU64,
    // ! Steam Guard secrets and the login/link sessions, in their own mutex so a long network call never blocks the account list.
    authenticator: Arc<Mutex<Authenticator>>,
    window_active: AtomicBool,
    tray_available: AtomicBool,
    // ! Serializes operations that stop Steam or change its login files.
    switching: Arc<AtomicBool>,
    // ! The app must not exit (window close, tray Quit, update restart) while this is non-zero, or a link Steam already accepted could be lost mid-flight.
    link_operations: AtomicUsize,
    // ! The corrupt file's name, set once at `new` when `open_database` had to move one aside; taken by `take_corrupt_db_backup` to show the notice exactly once, right after startup.
    corrupt_db_backup: Mutex<Option<String>>,
    // ! Serializes `set_tracking`, `clear_tracking_data` and `select_installation`'s tracker rebuild, so a toggle and an installation switch racing each other cannot leave the setting and the live tracker disagreeing.
    tracking_switch: Mutex<()>,
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

// ! RAII guard held for the duration of one link/finalize/transfer step; dropping it (including on an early return or panic unwind) always decrements the counter back.
pub struct LinkOperationGuard<'a> {
    counter: &'a AtomicUsize,
}

impl Drop for LinkOperationGuard<'_> {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::SeqCst);
    }
}

impl AppState {
    pub fn new() -> Self {
        let (database, corrupt_db_backup) = open_database();
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
                accounts_error: None,
                unlocked_at: None,
                settings,
            }),
            snapshot_revision: AtomicU64::new(0),
            authenticator: Arc::new(Mutex::new(Authenticator::new(database))),
            window_active: AtomicBool::new(true),
            tray_available: AtomicBool::new(false),
            switching: Arc::new(AtomicBool::new(false)),
            link_operations: AtomicUsize::new(0),
            corrupt_db_backup: Mutex::new(corrupt_db_backup),
            tracking_switch: Mutex::new(()),
        };
        state.reload_installations();
        state
    }

    pub fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|error| error.into_inner())
    }

    pub(crate) fn tracking_switch(&self) -> MutexGuard<'_, ()> {
        self.tracking_switch
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }

    // ! The corrupt file's name recovered at startup, once: `None` on every later call, including when nothing was ever recovered this run.
    pub fn take_corrupt_db_backup(&self) -> Option<String> {
        self.corrupt_db_backup
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take()
    }

    // ! Marks one link/finalize/transfer step as running; the app refuses to exit until every returned guard has dropped.
    pub fn begin_link_operation(&self) -> LinkOperationGuard<'_> {
        self.link_operations.fetch_add(1, Ordering::SeqCst);
        LinkOperationGuard {
            counter: &self.link_operations,
        }
    }

    pub fn link_operation_in_flight(&self) -> bool {
        self.link_operations.load(Ordering::SeqCst) > 0
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

// ! Refuses exit while a link/finalize/transfer step is in flight, showing the main window and a translated native message; returns `true` when the caller must not exit.
pub fn exit_blocked(app: &tauri::AppHandle) -> bool {
    use tauri::Manager;
    let in_flight = app.state::<AppState>().link_operation_in_flight();
    if in_flight {
        crate::logging::write("action=request_exit status=blocked reason=link_in_flight");
        crate::show_main_window(app);
        let locale = app
            .state::<AppState>()
            .lock()
            .settings
            .data()
            .language
            .resolved();
        // ! Non-blocking: this runs inside tao's event loop, which owns the GLib idle dispatch a blocking rfd dialog needs.
        app.dialog()
            .message(crate::catalog::text(locale, "exit_blocked_link_in_flight"))
            .title("Steam Account Switcher")
            .kind(MessageDialogKind::Warning)
            .buttons(MessageDialogButtons::Ok)
            .show(|_| {});
    }
    in_flight
}

// ! Single exit point for the tray Quit item and the update-restart command: refuses to exit while `exit_blocked` says a link step is in flight instead of exiting under it.
pub fn request_exit(app: &tauri::AppHandle) {
    if !exit_blocked(app) {
        app.exit(0);
    }
}

// ! Single exit point for the window close button (close-to-tray disabled): vetoes the close instead of letting the window be destroyed while a link step runs.
pub fn request_close(app: &tauri::AppHandle, api: &tauri::CloseRequestApi) {
    if exit_blocked(app) {
        api.prevent_close();
    } else {
        app.exit(0);
    }
}

pub fn key_for_path(path: &str) -> String {
    steam_core::model::resolve(Path::new(path))
        .display()
        .to_string()
}

// ! Startup stops without the database: running on would lose a newly saved Steam Guard secret on restart, and the file must stay untouched so a build with the matching schema can still open it.
fn open_database() -> (Database, Option<String>) {
    let locale = crate::locale::Language::Auto.resolved();
    let opened = match Database::open(&crate::settings::database_path()) {
        Ok(opened) => opened,
        Err(error) => {
            // ! No window or panic hook yet: log the reason, then show a native message box so the failure is not silent, then stop.
            let code = error.code();
            let mut event = format!("action=open_database status=failed code={code}");
            if let Some(detail) = error.detail() {
                event.push_str(&format!(" detail={detail}"));
            }
            crate::logging::write(&event);
            show_startup_failure(locale, code);
            std::process::exit(1);
        }
    };
    let backup_name = opened.moved_aside.as_ref().map(|backup| {
        let name = backup
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        crate::logging::write(&format!(
            "action=open_database status=recovered category=database_corrupt backup={name}"
        ));
        name
    });
    (opened.database, backup_name)
}

// ! A plain native dialog, shown before any window exists, so a startup failure the user cannot fix by retrying is not silent.
fn show_startup_failure(locale: &str, code: ErrorCode) {
    let key = match code {
        ErrorCode::DatabaseVersionMismatch => "startup_database_version_mismatch",
        _ => "startup_database_open_failed",
    };
    rfd::MessageDialog::new()
        .set_title("Steam Account Switcher")
        .set_level(rfd::MessageLevel::Error)
        .set_buttons(rfd::MessageButtons::Ok)
        .set_description(crate::catalog::text(locale, key))
        .show();
}

// ! Shown once, right after startup, when `open_database` had to move a corrupted file aside: the recovered file name, never its contents.
pub fn show_database_recovered_notice(app: &tauri::AppHandle, locale: &str, backup: &str) {
    let message =
        crate::catalog::text(locale, "startup_database_recovered").replacen("{backup}", backup, 1);
    // ! Non-blocking: this runs inside tao's event loop, which owns the GLib idle dispatch a blocking rfd dialog needs.
    app.dialog()
        .message(message)
        .title("Steam Account Switcher")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::Ok)
        .show(|_| {});
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
