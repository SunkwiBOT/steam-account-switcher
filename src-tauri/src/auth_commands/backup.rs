use std::path::Path;

use steam_core::error_code::ErrorCode;
use tauri::State;

use super::vault::vault_snapshot;
use crate::commands::SnapshotDto;
use crate::state::AppState;

#[derive(serde::Serialize)]
pub struct BackupImportDto {
    pub imported: usize,
    pub skipped: usize,
    pub snapshot: SnapshotDto,
}

/// Extension of the backup files, shared with the interface's file filter.
const BACKUP_EXTENSION: &str = "sasguard";

/// ! The destination comes from a native save dialog opened here, never from
/// the page, so a script in the webview can never choose where the secrets are
/// written. `None` means cancelled.
#[tauri::command]
pub async fn steam_guard_export_backup(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    steam_id: Option<String>,
    password: Option<String>,
) -> Result<Option<usize>, String> {
    use tauri::Manager;
    use tauri_plugin_dialog::DialogExt;

    crate::logging::run_async("steam_guard_export_backup", async {
        let password = password.map(zeroize::Zeroizing::new);
        let locale = state.lock().settings.data().language.resolved();
        let authenticator = state.authenticator();
        tauri::async_runtime::spawn_blocking(move || {
            let mut dialog = app
                .dialog()
                .file()
                .set_title(rust_i18n::t!("backup_export_title", locale = locale))
                .set_file_name(format!("steam-guard.{BACKUP_EXTENSION}"))
                .add_filter(
                    rust_i18n::t!("backup_file_filter", locale = locale),
                    &[BACKUP_EXTENSION],
                );
            if let Some(window) = app.get_webview_window("main") {
                dialog = dialog.set_parent(&window);
            }
            let Some(chosen) = dialog.blocking_save_file() else {
                return Ok(None);
            };
            let path = chosen
                .into_path()
                .map_err(|_| ErrorCode::BackupTargetUnsafe.to_string())?;
            authenticator
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .export_backup(
                    &path,
                    steam_id.as_deref(),
                    password.as_deref().map(|value| value.as_str()),
                )
                .map(Some)
                .map_err(|error| error.to_string())
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())?
    })
    .await
}

#[tauri::command]
pub async fn steam_guard_import_backup(
    state: State<'_, AppState>,
    path: String,
    password: Option<String>,
) -> Result<BackupImportDto, String> {
    crate::logging::run_async("steam_guard_import_backup", async {
        let authenticator = state.authenticator();
        let result = tauri::async_runtime::spawn_blocking(move || {
            let password = password.map(zeroize::Zeroizing::new);
            authenticator
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .import_backup(
                    Path::new(&path),
                    password.as_deref().map(|value| value.as_str()),
                )
                .map_err(|error| error.to_string())
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())??;
        Ok(BackupImportDto {
            imported: result.imported,
            skipped: result.skipped,
            snapshot: vault_snapshot(&state),
        })
    })
    .await
}
