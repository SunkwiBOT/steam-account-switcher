//! Access to the application's own diagnostics, never arbitrary filesystem paths.

use steam_core::error_code::ErrorCode;
use tauri_plugin_opener::OpenerExt;

#[tauri::command(async)]
pub fn logs_snapshot(previous: bool) -> Result<crate::logging::LogSnapshot, String> {
    crate::logging::read("logs_snapshot", || crate::logging::snapshot(previous))
}

#[tauri::command(async)]
pub fn logs_open_directory(app: tauri::AppHandle) -> Result<(), String> {
    crate::logging::run("logs_open_directory", || {
        let directory = crate::logging::directory()?;
        app.opener()
            .open_path(directory.to_string_lossy(), None::<&str>)
            .map_err(|_| ErrorCode::OpenFolderFailed.to_string())
    })
}
