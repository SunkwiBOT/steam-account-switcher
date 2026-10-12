use std::sync::atomic::Ordering;
use std::sync::Arc;

use steam_core::model::SteamAccount;

use super::{AppState, SteamOperationGuard};

impl AppState {
    /// Called while the catalog is locked so older responses can be discarded.
    pub fn next_snapshot_revision(&self) -> u64 {
        self.snapshot_revision.fetch_add(1, Ordering::Relaxed) + 1
    }

    pub fn set_privacy(
        &self,
        masked: Option<bool>,
        visible_ids: Option<bool>,
    ) -> Result<(), String> {
        let mut inner = self.lock();
        let previous = inner.settings.data().clone();
        if let Some(masked) = masked {
            inner.settings.data_mut().show_account_names = !masked;
        }
        if let Some(visible) = visible_ids {
            inner.settings.data_mut().show_steam_ids = visible;
        }
        if let Err(error) = inner.settings.save() {
            *inner.settings.data_mut() = previous;
            return Err(error);
        }
        inner.masked = !inner.settings.data().show_account_names;
        inner.steam_ids_visible = inner.settings.data().show_steam_ids;
        Ok(())
    }

    pub fn display_accounts(&self) -> Vec<SteamAccount> {
        let (mut ids, stored, indexed) = {
            let guard = self
                .authenticator
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            (
                guard.configured_ids(),
                guard.accounts(),
                guard.status().indexed,
            )
        };
        let mut inner = self.lock();
        let local = inner.accounts.clone();
        let previous = inner.settings.data().guard_account_profiles.clone();
        if !indexed {
            ids.extend(
                previous
                    .keys()
                    .filter(|id| !ids.contains(id))
                    .cloned()
                    .collect::<Vec<_>>(),
            );
        }
        let profiles = &mut inner.settings.data_mut().guard_account_profiles;
        let changed = crate::retained_accounts::remember_profiles(profiles, &local, &stored, &ids);
        let accounts = crate::retained_accounts::display_accounts(&local, &ids, profiles);
        if changed {
            if let Err(error) = inner.settings.save() {
                inner.settings.data_mut().guard_account_profiles = previous;
                crate::logging::write(&format!(
                    "action=retain_guard_accounts status=failed error={}",
                    crate::logging::error_category(&error)
                ));
            }
        }
        accounts
    }

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

    pub fn begin_steam_operation(&self) -> Result<SteamOperationGuard, String> {
        SteamOperationGuard::acquire(Arc::clone(&self.switching))
    }

    pub fn is_switching(&self) -> bool {
        self.switching.load(Ordering::SeqCst)
    }
}
