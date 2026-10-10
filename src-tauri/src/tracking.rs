use std::collections::BTreeMap;

use steam_core::error_code::ErrorCode;
use steam_core::model::SteamInstallation;
use steam_core::playtime::{
    clear_stored_history, installation_key, recorded_account_count, stored_installations,
};
use steam_core::storage::Database;
use tauri::State;

use crate::commands::dto::{TrackingHistoryDto, TrackingInstallationDto};
use crate::commands::{build_snapshot, SnapshotDto};
use crate::state::AppState;

/// One row per installation that has stored history or is known to the app.
pub(crate) fn tracking_history(
    database: &Database,
    installations: &[SteamInstallation],
) -> Vec<TrackingHistoryDto> {
    let mut rows: BTreeMap<String, Option<TrackingInstallationDto>> = BTreeMap::new();
    for installation in installations {
        rows.insert(
            installation_key(installation),
            Some(TrackingInstallationDto {
                kind: installation.kind.as_str().to_string(),
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
        let mut inner = self.lock();
        let previous = inner.settings.data().tracking_enabled;
        if previous == enabled {
            return Ok(());
        }
        let tracker = inner.tracker.clone();
        let mut tracker = tracker
            .as_ref()
            .map(|tracker| tracker.lock())
            .transpose()
            .map_err(|_| ErrorCode::PlaytimeBusy.to_string())?;
        inner.settings.data_mut().tracking_enabled = enabled;
        if let Err(error) = inner.settings.save() {
            inner.settings.data_mut().tracking_enabled = previous;
            return Err(error);
        }
        if let Some(tracker) = tracker.as_mut() {
            if let Err(error) = tracker.set_enabled(enabled) {
                inner.settings.data_mut().tracking_enabled = previous;
                if inner.settings.save().is_err() {
                    crate::logging::write("action=rollback_tracking_preference status=failed category=settings_save_failed");
                }
                return Err(error.to_string());
            }
        }
        Ok(())
    }

    /// Erases the stored history of every installation. The running tracker, when
    /// there is one, is reset in the same transaction, so a failure keeps everything.
    pub(crate) fn clear_tracking_data(&self, confirmed: bool) -> Result<(), String> {
        if !confirmed {
            return Err(ErrorCode::ConfirmationRequired.to_string());
        }
        let inner = self.lock();
        let tracker = inner
            .tracker
            .as_ref()
            .map(|tracker| tracker.lock())
            .transpose()
            .map_err(|_| ErrorCode::PlaytimeBusy.to_string())?;
        match tracker {
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
pub fn clear_playtime_data(
    state: State<'_, AppState>,
    confirmed: bool,
) -> Result<SnapshotDto, String> {
    crate::logging::run("clear_playtime_data", || {
        state.clear_tracking_data(confirmed)?;
        Ok(build_snapshot(&state))
    })
}
