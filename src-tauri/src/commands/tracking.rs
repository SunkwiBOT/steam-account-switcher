use std::collections::BTreeMap;

use steam_core::error_code::ErrorCode;
use steam_core::model::SteamInstallation;
use steam_core::playtime::{
    clear_stored_history, installation_key, recorded_account_count, stored_installations,
};
use steam_core::storage::Database;
use tauri::State;

use super::dto::{TrackingHistoryDto, TrackingInstallationDto};
use super::{build_snapshot, SnapshotDto};
use crate::state::AppState;

// ! One row per installation that has stored history or is known to the app.
pub(crate) fn tracking_history(
    database: &Database,
    installations: &[SteamInstallation],
) -> Vec<TrackingHistoryDto> {
    let mut rows: BTreeMap<String, Option<TrackingInstallationDto>> = BTreeMap::new();
    for installation in installations {
        rows.insert(
            installation_key(installation),
            Some(TrackingInstallationDto {
                kind: installation.kind,
                root: installation.root.display().to_string(),
            }),
        );
    }
    for key in stored_installations(database) {
        rows.entry(key).or_insert(None);
    }
    rows.into_iter()
        .map(|(key, installation)| TrackingHistoryDto {
            installation,
            account_count: recorded_account_count(database, &key).unwrap_or(0),
        })
        .collect()
}

impl AppState {
    pub(crate) fn set_tracking(&self, enabled: bool) -> Result<(), String> {
        let _switch = self.tracking_switch();
        let tracker = {
            let mut inner = self.lock();
            let previous = inner.settings.data().clone();
            if previous.tracking_enabled == enabled {
                return Ok(());
            }
            inner.settings.data_mut().tracking_enabled = enabled;
            crate::state::save_or_restore(&mut inner.settings, previous)?;
            inner.tracker.clone()
        };
        let mut tracker = tracker
            .as_ref()
            .map(|tracker| tracker.lock())
            .transpose()
            .map_err(|_| ErrorCode::PlaytimeBusy.to_string())?;
        if let Some(tracker) = tracker.as_mut() {
            if let Err(error) = tracker.set_enabled(enabled) {
                let mut inner = self.lock();
                let restored = inner.settings.data().clone();
                inner.settings.data_mut().tracking_enabled = !enabled;
                if crate::state::save_or_restore(&mut inner.settings, restored).is_err() {
                    crate::logging::write("action=rollback_tracking_preference status=failed category=settings_save_failed");
                }
                return Err(error.to_string());
            }
        }
        Ok(())
    }

    // ! Clears every installation's history atomically; the tracker's lock is taken only after the state lock is released, so this never blocks the account list.
    pub(crate) fn clear_tracking_data(&self) -> Result<(), String> {
        let _switch = self.tracking_switch();
        let tracker = self.lock().tracker.clone();
        let locked = tracker
            .as_ref()
            .map(|tracker| tracker.lock())
            .transpose()
            .map_err(|_| ErrorCode::PlaytimeBusy.to_string())?;
        match locked {
            Some(mut tracker) => tracker.clear_all_history(),
            None => clear_stored_history(&self.database, None),
        }
        .map_err(|error| error.to_string())
    }
}

#[tauri::command(async)]
pub fn set_tracking_enabled(
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<SnapshotDto, String> {
    crate::logging::run("set_tracking_enabled", || {
        state.set_tracking(enabled)?;
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn clear_playtime_data(state: State<'_, AppState>) -> Result<SnapshotDto, String> {
    crate::logging::run("clear_playtime_data", || {
        state.clear_tracking_data()?;
        Ok(build_snapshot(&state))
    })
}
