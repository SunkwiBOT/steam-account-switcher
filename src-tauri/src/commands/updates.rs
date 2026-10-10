use serde::Serialize;
use std::sync::Mutex;
use steam_core::error_code::ErrorCode;
use tauri::{Emitter, State};
use tauri_plugin_updater::Update;

#[derive(Serialize)]
pub struct UpdateInfoDto {
    pub version: String,
    pub current_version: String,
    pub notes: Option<String>,
}
#[tauri::command]
pub async fn check_for_update(
    app: tauri::AppHandle,
    state: State<'_, crate::state::AppState>,
) -> Result<Option<UpdateInfoDto>, String> {
    let result = crate::logging::run_async("check_for_update", async {
        use tauri_plugin_updater::UpdaterExt;

        let updater = app
            .updater()
            .map_err(|_| ErrorCode::UpdateFailed.to_string())?;
        let update = updater
            .check()
            .await
            .map_err(|_| ErrorCode::UpdateFailed.to_string())?;

        Ok(update.map(|update| UpdateInfoDto {
            version: update.version.clone(),
            current_version: update.current_version.clone(),
            notes: update.body.clone(),
        }))
    })
    .await;
    let mut inner = state.lock();
    let previous = inner.settings.data().last_update_check_at;
    inner.settings.data_mut().last_update_check_at = Some(steam_core::util::now_unix() as u64);
    if inner.settings.save().is_err() {
        inner.settings.data_mut().last_update_check_at = previous;
        crate::logging::write(
            "action=save_update_check status=failed category=settings_save_failed",
        );
    }
    result
}

#[derive(Clone, Serialize)]
pub struct UpdateProgressDto {
    pub downloaded: u64,
    pub total: Option<u64>,
}
#[derive(Default)]
pub struct PendingUpdate(Mutex<Option<DownloadedUpdate>>);

impl PendingUpdate {
    pub fn new() -> Self {
        Self::default()
    }
}

struct DownloadedUpdate {
    update: Update,
    bytes: Vec<u8>,
}

#[tauri::command]
pub async fn download_update(
    app: tauri::AppHandle,
    pending: State<'_, PendingUpdate>,
) -> Result<String, String> {
    crate::logging::run_async("download_update", async {
        use tauri_plugin_updater::UpdaterExt;

        let updater = app
            .updater()
            .map_err(|_| ErrorCode::UpdateFailed.to_string())?;
        let update = updater
            .check()
            .await
            .map_err(|_| ErrorCode::UpdateFailed.to_string())?
            .ok_or_else(|| ErrorCode::NoUpdate.as_str().to_string())?;
        let version = update.version.clone();

        let emitter = app.clone();
        let mut downloaded: u64 = 0;
        let bytes = update
            .download(
                move |chunk_length, total| {
                    downloaded += chunk_length as u64;
                    let _ =
                        emitter.emit("update-progress", UpdateProgressDto { downloaded, total });
                },
                || {},
            )
            .await
            .map_err(|_| ErrorCode::UpdateFailed.to_string())?;

        let mut slot = pending
            .0
            .lock()
            .map_err(|_| ErrorCode::UpdateFailed.to_string())?;
        *slot = Some(DownloadedUpdate { update, bytes });
        drop(slot);

        let _ = app.emit("update-downloaded", version.clone());
        Ok(version)
    })
    .await
}
#[tauri::command]
pub async fn apply_update(
    app: tauri::AppHandle,
    pending: State<'_, PendingUpdate>,
    version: String,
) -> Result<(), String> {
    crate::logging::run_async("apply_update", async {
        // Only the version the user was shown may be installed: a different
        // downloaded build stays pending and is not taken.
        let downloaded = {
            let mut slot = pending
                .0
                .lock()
                .map_err(|_| ErrorCode::UpdateFailed.to_string())?;
            match slot.as_ref() {
                Some(candidate) if candidate.update.version == version => slot.take(),
                _ => None,
            }
        }
        .ok_or_else(|| ErrorCode::NoDownloadedUpdate.as_str().to_string())?;

        crate::logging::write("action=apply_update stage=installing");
        downloaded
            .update
            .install(&downloaded.bytes)
            .map_err(|_| ErrorCode::UpdateFailed.to_string())?;

        crate::logging::write("action=apply_update status=succeeded restart=true");

        app.restart();
    })
    .await
}
