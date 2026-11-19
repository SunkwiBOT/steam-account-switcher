use steam_core::error_code::ErrorCode;
use tauri::State;

use super::vault::vault_snapshot;
use crate::commands::dto::BackupImportDto;
use crate::state::AppState;

// ! Shared with the interface's file-picker filter: the two must name the same extension.
pub(crate) const BACKUP_EXTENSION: &str = "steamguard";

// ! The destination comes from a native save dialog opened here, never from the page, so a script in the webview can never choose where the secrets are written. `None` means cancelled.
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
                .set_title(crate::catalog::text(locale, "backup_export_title"))
                .set_file_name(format!("authenticators.{BACKUP_EXTENSION}"))
                .add_filter(
                    crate::catalog::text(locale, "backup_file_filter"),
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
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    password: Option<String>,
) -> Result<Option<BackupImportDto>, String> {
    use tauri::Manager;
    use tauri_plugin_dialog::DialogExt;

    crate::logging::run_async("steam_guard_import_backup", async {
        let locale = state.lock().settings.data().language.resolved();
        let authenticator = state.authenticator();
        let chosen = tauri::async_runtime::spawn_blocking(move || {
            let mut dialog = app
                .dialog()
                .file()
                .set_title(crate::catalog::text(locale, "backup_import_title"))
                .add_filter(
                    crate::catalog::text(locale, "backup_file_filter"),
                    &[BACKUP_EXTENSION],
                );
            if let Some(window) = app.get_webview_window("main") {
                dialog = dialog.set_parent(&window);
            }
            dialog.blocking_pick_file()
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())?;

        let Some(chosen) = chosen else {
            return Ok(None);
        };
        let path = chosen
            .into_path()
            .map_err(|_| ErrorCode::BackupTargetUnsafe.to_string())?;

        let result = tauri::async_runtime::spawn_blocking(move || {
            let password = password.map(zeroize::Zeroizing::new);
            authenticator
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .import_backup(&path, password.as_deref().map(|value| value.as_str()))
                .map_err(|error| error.to_string())
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())??;
        Ok(Some(BackupImportDto {
            imported: result.imported,
            skipped: result.skipped,
            snapshot: vault_snapshot(&state),
        }))
    })
    .await
}
