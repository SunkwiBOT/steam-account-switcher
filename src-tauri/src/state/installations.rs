use std::path::Path;
use std::sync::{Arc, Mutex};

use steam_core::accounts::SteamManager;
use steam_core::error_code::ErrorCode;
use steam_core::model::SteamInstallation;
use steam_core::pathdetect;
use steam_core::playtime::PlaytimeTracker;

use super::{build_tracker, key_for_path, root_key, same_path, save_or_restore, AppState};

impl AppState {
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
            inner.last_error = error;
        }
    }

    pub fn select_installation(&self, index: usize) -> Result<(), String> {
        let installation = {
            let inner = self.lock();
            inner
                .installations
                .get(index)
                .cloned()
                .ok_or_else(|| ErrorCode::NoSteamInstallation.as_str().to_string())?
        };

        let manager = Arc::new(SteamManager::new(installation.clone()));
        let accounts = manager.list_accounts().unwrap_or_default();
        let auto_login_user = manager.auto_login_user().unwrap_or_default();

        let mut inner = self.lock();
        if let Some(tracker) = &inner.tracker {
            tracker
                .lock()
                .map_err(|_| ErrorCode::PlaytimeBusy.to_string())?
                .set_enabled(false)
                .map_err(|error| error.to_string())?;
        }
        let tracker = Arc::new(Mutex::new(build_tracker(
            &manager,
            self.database.clone(),
            Arc::clone(&inner.monitor),
            inner.settings.data().tracking_enabled,
        )));
        inner.selected = index;
        inner.manager = Some(manager);
        inner.tracker = Some(tracker);
        inner.accounts = accounts;
        inner.auto_login_user = auto_login_user;
        Ok(())
    }

    pub fn set_ui_settings(&self, show_installation_selector: bool) -> Result<(), String> {
        let mut inner = self.lock();
        let previous = inner.settings.data().show_installation_selector;
        let settings = inner.settings.data_mut();
        settings.show_installation_selector = show_installation_selector;
        if let Err(error) = inner.settings.save() {
            inner.settings.data_mut().show_installation_selector = previous;
            return Err(error);
        }
        Ok(())
    }

    pub fn set_installation_hidden(&self, root: &str, hidden: bool) -> Result<(), String> {
        {
            let mut inner = self.lock();
            let previous = inner.settings.data().clone();
            let settings = inner.settings.data_mut();
            if hidden {
                settings.hide(root);
            } else {
                settings.show(root);
            }
            save_or_restore(&mut inner.settings, previous)?;
        }
        self.reload_installations();

        // Hiding the selected installation moved the selection elsewhere;
        // revealing it must select it back or the list stays on the one that took over.
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

    pub fn add_custom_installation(&self, path: &str) -> Result<String, String> {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            return Err(ErrorCode::SteamFolderRequired.as_str().to_string());
        }

        let installation = pathdetect::from_custom_path(Path::new(trimmed)).ok_or_else(|| {
            pathdetect::classify_custom_path(Path::new(trimmed))
                .message()
                .to_string()
        })?;
        let key = root_key(&installation);

        {
            let mut inner = self.lock();
            let previous = inner.settings.data().clone();
            inner.settings.data_mut().add_custom(&key);
            save_or_restore(&mut inner.settings, previous)?;
        }
        self.reload_installations();
        Ok(key)
    }

    pub fn remove_custom_installation(&self, root: &str) -> Result<(), String> {
        {
            let mut inner = self.lock();
            let previous = inner.settings.data().clone();
            inner.settings.data_mut().remove_custom(root);
            save_or_restore(&mut inner.settings, previous)?;
        }
        self.reload_installations();
        Ok(())
    }

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
                let message = error.to_string();
                let mut inner = self.lock();
                if inner.last_error != message {
                    crate::logging::write(&format!(
                        "action=refresh_accounts status=failed error={}",
                        crate::logging::error_category(&message)
                    ));
                }
                inner.last_error = message;
            }
        }
    }

    pub fn tracker(&self) -> Option<Arc<Mutex<PlaytimeTracker>>> {
        self.lock().tracker.clone()
    }

    /// Stops the tracker cleanly so active sessions are checkpointed.
    pub fn close_tracker(&self) {
        if let Some(tracker) = self.tracker() {
            if let Ok(mut tracker) = tracker.lock() {
                if tracker.close().is_err() {
                    crate::logging::write(
                        "action=playtime_close status=failed category=state_not_saved",
                    );
                }
            }
        }
    }
}
