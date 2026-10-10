use super::*;

#[derive(serde::Serialize)]
pub struct BackupImportDto {
    pub imported: usize,
    pub skipped: usize,
    pub snapshot: SnapshotDto,
}

#[tauri::command]
pub async fn steam_guard_export_backup(
    state: State<'_, AppState>,
    steam_id: Option<String>,
    path: String,
    password: Option<String>,
) -> Result<usize, String> {
    crate::logging::run_async("steam_guard_export_backup", async {
        let authenticator = state.authenticator();
        tauri::async_runtime::spawn_blocking(move || {
            let password = password.map(zeroize::Zeroizing::new);
            authenticator
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .export_backup(
                    Path::new(&path),
                    steam_id.as_deref(),
                    password.as_deref().map(|value| value.as_str()),
                )
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
