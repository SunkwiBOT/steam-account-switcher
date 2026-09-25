//! Shared application state: installations, accounts, ranks and tracking.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use steam_core::accounts::SteamManager;
use steam_core::metadata::RankStore;
use steam_core::model::{SteamAccount, SteamInstallation};
use steam_core::pathdetect;
use steam_core::playtime::{system_clock, AccountProvider, PlaytimeTracker, ProbeRef};
use steam_core::processes::ProcessMonitor;

use crate::settings::SettingsStore;

/// Canonical key used to compare installations between runs.
pub fn root_key(installation: &SteamInstallation) -> String {
    steam_core::model::resolve(&installation.root)
        .display()
        .to_string()
}

/// Account Steam is currently logged into.
pub fn active_account_for(manager: &SteamManager) -> Option<SteamAccount> {
    manager.current_account().ok().flatten()
}

pub struct Inner {
    /// Every installation found automatically, including the hidden ones.
    pub detected: Vec<SteamInstallation>,
    /// Roots added by hand in the settings panel.
    pub custom: Vec<SteamInstallation>,
    /// Selectable installations: detected minus hidden, plus custom.
    pub installations: Vec<SteamInstallation>,
    pub selected: usize,
    pub manager: Option<Arc<SteamManager>>,
    pub accounts: Vec<SteamAccount>,
    pub auto_login_user: String,
    pub ranks: Option<RankStore>,
    pub tracker: Option<Arc<Mutex<PlaytimeTracker>>>,
    pub monitor: Arc<Mutex<ProcessMonitor>>,
    /// Account names are hidden by default.
    pub masked: bool,
    pub steam_ids_visible: bool,
    pub message: String,
    pub settings: SettingsStore,
}

pub struct AppState {
    inner: Mutex<Inner>,
    /// Window focused and visible: the tracking loop runs at full speed.
    window_active: AtomicBool,
    /// Whether a tray icon exists, which decides what closing the window does.
    tray_available: AtomicBool,
    /// Serializes operations that stop Steam or change its login files.
    switching: Arc<AtomicBool>,
}

/// Releases the operation slot on success, error or cancellation.
pub struct SteamOperationGuard(Arc<AtomicBool>);

impl SteamOperationGuard {
    fn acquire(busy: Arc<AtomicBool>) -> Result<Self, String> {
        busy.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .map(|_| Self(busy))
            .map_err(|_| "A Steam operation is already running. Wait for it to finish.".to_string())
    }
}

impl Drop for SteamOperationGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

impl AppState {
    pub fn new() -> Self {
        let state = Self {
            inner: Mutex::new(Inner {
                detected: Vec::new(),
                custom: Vec::new(),
                installations: Vec::new(),
                selected: 0,
                manager: None,
                accounts: Vec::new(),
                auto_login_user: String::new(),
                ranks: None,
                tracker: None,
                monitor: Arc::new(Mutex::new(ProcessMonitor::new())),
                masked: true,
                steam_ids_visible: false,
                message: String::new(),
                settings: SettingsStore::load(),
            }),
            window_active: AtomicBool::new(true),
            tray_available: AtomicBool::new(false),
            switching: Arc::new(AtomicBool::new(false)),
        };
        state.reload_installations();
        state
    }

    /// Locks the state, recovering from a poisoned mutex instead of panicking.
    pub fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|error| error.into_inner())
    }

    /// Detects every Steam installation and selects the previous one if it is
    /// still present.
    pub fn reload_installations(&self) {
        let (previous, hidden, custom_paths) = {
            let inner = self.lock();
            let previous = inner
                .manager
                .as_ref()
                .map(|manager| root_key(&manager.installation));
            let settings = inner.settings.data();
            (
                previous,
                settings.hidden_installations.clone(),
                settings.custom_installations.clone(),
            )
        };

        let detected = pathdetect::detect_all();
        let custom: Vec<SteamInstallation> = custom_paths
            .iter()
            .filter_map(|path| pathdetect::from_custom_path(Path::new(path)))
            .collect();

        let mut installations: Vec<SteamInstallation> = detected
            .iter()
            .filter(|installation| {
                let key = root_key(installation);
                !hidden.iter().any(|entry| same_path(entry, &key))
            })
            .cloned()
            .collect();

        for installation in &custom {
            let key = root_key(installation);
            if !installations
                .iter()
                .any(|existing| root_key(existing) == key)
            {
                installations.push(installation.clone());
            }
        }

        let selected = previous
            .and_then(|root| {
                installations
                    .iter()
                    .position(|installation| root_key(installation) == root)
            })
            .unwrap_or(0);

        {
            let mut inner = self.lock();
            inner.detected = detected;
            inner.custom = custom;
            inner.installations = installations;
            let last = inner.installations.len().saturating_sub(1);
            inner.selected = selected.min(last);
        }

        let selected = self.lock().selected;
        if let Err(error) = self.select_installation(selected) {
            let mut inner = self.lock();
            inner.manager = None;
            inner.accounts.clear();
            inner.tracker = None;
            inner.message = error;
        }
    }

    /// Points the application at another detected installation.
    pub fn select_installation(&self, index: usize) -> Result<(), String> {
        let installation = {
            let inner = self.lock();
            inner
                .installations
                .get(index)
                .cloned()
                .ok_or_else(|| "No Steam installation found.".to_string())?
        };

        let manager = Arc::new(SteamManager::new(installation.clone()));
        let monitor = Arc::clone(&self.lock().monitor);
        let tracker = Arc::new(Mutex::new(build_tracker(&manager, monitor)));
        let ranks = RankStore::for_installation(&installation);

        let accounts = manager.list_accounts().unwrap_or_default();
        let auto_login_user = manager.auto_login_user().unwrap_or_default();

        let mut inner = self.lock();
        inner.selected = index;
        inner.manager = Some(manager);
        inner.ranks = Some(ranks);
        inner.tracker = Some(tracker);
        inner.accounts = accounts;
        inner.auto_login_user = auto_login_user;
        Ok(())
    }

    /// Stores the display preferences and returns whether anything changed.
    pub fn set_ui_settings(
        &self,
        show_installation_selector: bool,
        show_machine_playtime: bool,
    ) -> Result<(), String> {
        let mut inner = self.lock();
        let settings = inner.settings.data_mut();
        settings.show_installation_selector = show_installation_selector;
        settings.show_machine_playtime = show_machine_playtime;
        inner.settings.save()
    }

    /// Hides or reveals an automatically detected installation.
    pub fn set_installation_hidden(&self, root: &str, hidden: bool) -> Result<(), String> {
        {
            let mut inner = self.lock();
            let settings = inner.settings.data_mut();
            if hidden {
                settings.hide(root);
            } else {
                settings.show(root);
            }
            inner.settings.save()?;
        }
        self.reload_installations();

        // Hiding the selected installation hands the selection to another one,
        // so revealing it again must select it back. Without this the account
        // list stays on whichever installation took over.
        if !hidden {
            let wanted = key_for_path(root);
            let index = {
                let inner = self.lock();
                inner
                    .installations
                    .iter()
                    .position(|installation| root_key(installation) == wanted)
            };
            if let Some(index) = index {
                self.select_installation(index)?;
            }
        }
        Ok(())
    }

    /// Adds a Steam root typed by the user after validating it.
    pub fn add_custom_installation(&self, path: &str) -> Result<String, String> {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            return Err("Enter a Steam folder first.".to_string());
        }

        let installation = pathdetect::from_custom_path(Path::new(trimmed)).ok_or_else(|| {
            // Explain what is missing rather than "not found": Steam is often
            // installed but never signed in, so there is no login history yet.
            format!(
                "{} ({trimmed})",
                pathdetect::classify_custom_path(Path::new(trimmed)).message()
            )
        })?;
        let key = root_key(&installation);

        {
            let mut inner = self.lock();
            inner.settings.data_mut().add_custom(&key);
            inner.settings.save()?;
        }
        self.reload_installations();
        Ok(key)
    }

    /// Removes a manually added installation.
    pub fn remove_custom_installation(&self, root: &str) -> Result<(), String> {
        {
            let mut inner = self.lock();
            inner.settings.data_mut().remove_custom(root);
            inner.settings.save()?;
        }
        self.reload_installations();
        Ok(())
    }

    /// Re-reads `loginusers.vdf` and the auto-login account.
    pub fn refresh_accounts(&self) {
        let manager = self.lock().manager.clone();
        let Some(manager) = manager else {
            return;
        };

        match manager.list_accounts() {
            Ok(accounts) => {
                let auto_login_user = manager.auto_login_user().unwrap_or_default();
                let mut inner = self.lock();
                inner.accounts = accounts;
                inner.auto_login_user = auto_login_user;
            }
            Err(error) => {
                self.lock().message = error.to_string();
            }
        }
    }

    pub fn tracker(&self) -> Option<Arc<Mutex<PlaytimeTracker>>> {
        self.lock().tracker.clone()
    }

    /// Driven by the window focus/blur events. While the window is in the
    /// background the tracking loop only samples every few seconds: enough for
    /// local playtime, and the application stays out of the way.
    pub fn set_window_active(&self, active: bool) {
        self.window_active.store(active, Ordering::Relaxed);
    }

    pub fn window_active(&self) -> bool {
        self.window_active.load(Ordering::Relaxed)
    }

    pub fn set_tray_available(&self, available: bool) {
        self.tray_available.store(available, Ordering::Relaxed);
    }

    pub fn tray_available(&self) -> bool {
        self.tray_available.load(Ordering::Relaxed)
    }

    /// Reserves exclusive access to Steam's login files and process lifecycle.
    pub fn begin_steam_operation(&self) -> Result<SteamOperationGuard, String> {
        SteamOperationGuard::acquire(Arc::clone(&self.switching))
    }

    pub fn is_switching(&self) -> bool {
        self.switching.load(Ordering::SeqCst)
    }

    // ------------------------------------------------------ folders & colours

    pub fn create_folder(&self, name: &str) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("Give the folder a name.".to_string());
        }
        let mut inner = self.lock();
        inner.settings.data_mut().create_folder(name);
        inner.settings.save()
    }

    pub fn rename_folder(&self, id: &str, name: &str) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("Give the folder a name.".to_string());
        }
        let mut inner = self.lock();
        if !inner.settings.data_mut().rename_folder(id, name) {
            return Err("Unknown folder.".to_string());
        }
        inner.settings.save()
    }

    pub fn delete_folder(&self, id: &str) -> Result<(), String> {
        let mut inner = self.lock();
        inner.settings.data_mut().delete_folder(id);
        inner.settings.save()
    }

    pub fn set_account_folder(&self, steam_id: &str, folder: Option<&str>) -> Result<(), String> {
        let mut inner = self.lock();
        inner
            .settings
            .data_mut()
            .set_account_folder(steam_id, folder);
        inner.settings.save()
    }

    pub fn set_account_color(&self, steam_id: &str, color: Option<&str>) -> Result<(), String> {
        let mut inner = self.lock();
        inner.settings.data_mut().set_account_color(steam_id, color);
        inner.settings.save()
    }

    /// Stops the tracker cleanly so active sessions are checkpointed.
    pub fn close_tracker(&self) {
        if let Some(tracker) = self.tracker() {
            if let Ok(mut tracker) = tracker.lock() {
                tracker.close();
            }
        }
    }
}

fn same_path(left: &str, right: &str) -> bool {
    steam_core::model::resolve(Path::new(left)) == steam_core::model::resolve(Path::new(right))
}

/// Canonical string used as the installation key.
pub fn key_for_path(path: &str) -> String {
    steam_core::model::resolve(Path::new(path))
        .display()
        .to_string()
}

fn build_tracker(
    manager: &Arc<SteamManager>,
    monitor: Arc<Mutex<ProcessMonitor>>,
) -> PlaytimeTracker {
    let probe: ProbeRef = Arc::new(move |pid| {
        let mut monitor = monitor.lock().unwrap_or_else(|error| error.into_inner());
        monitor.process_facts(pid)
    });

    let provider: AccountProvider = {
        let manager = Arc::clone(manager);
        Arc::new(move || active_account_for(&manager))
    };

    PlaytimeTracker::new(&manager.installation, system_clock(), probe, provider)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steam_operations_are_exclusive_and_release_after_an_error() {
        let busy = Arc::new(AtomicBool::new(false));
        let fail = || -> Result<(), String> {
            let _operation = SteamOperationGuard::acquire(Arc::clone(&busy))?;
            assert!(SteamOperationGuard::acquire(Arc::clone(&busy)).is_err());
            Err("operation failed".to_string())
        };
        assert!(fail().is_err());
        let operation =
            SteamOperationGuard::acquire(Arc::clone(&busy)).expect("slot released after error");
        drop(operation);
        assert!(!busy.load(Ordering::SeqCst));
    }
}
