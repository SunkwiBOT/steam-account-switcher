use super::{build_snapshot, dto::SnapshotDto};
use crate::state::AppState;
use serde::Serialize;
use steam_core::error_code::ErrorCode;
use tauri::State;

const REPOSITORY_URL: &str = "https://github.com/SunkwiBOT/steam-account-switcher";

#[tauri::command(async)]
pub fn set_close_to_tray(
    state: State<'_, AppState>,
    close_to_tray: bool,
) -> Result<SnapshotDto, String> {
    crate::logging::run("set_close_to_tray", || {
        {
            let mut inner = state.lock();
            let previous = inner.settings.data().close_to_tray;
            inner.settings.data_mut().close_to_tray = close_to_tray;
            if let Err(error) = inner.settings.save() {
                inner.settings.data_mut().close_to_tray = previous;
                return Err(error);
            }
        }
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn set_update_interval(
    state: State<'_, AppState>,
    interval: crate::settings::UpdateInterval,
) -> Result<SnapshotDto, String> {
    crate::logging::run("set_update_interval", || {
        {
            let mut inner = state.lock();
            let previous = inner.settings.data().update_interval;
            inner.settings.data_mut().update_interval = interval;
            if let Err(error) = inner.settings.save() {
                inner.settings.data_mut().update_interval = previous;
                return Err(error);
            }
        }
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn set_vault_lock_interval(
    state: State<'_, AppState>,
    interval: crate::settings::VaultLockInterval,
) -> Result<SnapshotDto, String> {
    crate::logging::run("set_vault_lock_interval", || {
        {
            let mut inner = state.lock();
            let previous = inner.settings.data().vault_lock_interval;
            inner.settings.data_mut().vault_lock_interval = interval;
            if let Err(error) = inner.settings.save() {
                inner.settings.data_mut().vault_lock_interval = previous;
                return Err(error);
            }
        }
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn set_search_scope(
    state: State<'_, AppState>,
    scope: crate::settings::SearchScope,
) -> Result<SnapshotDto, String> {
    crate::logging::run("set_search_scope", || {
        {
            let mut inner = state.lock();
            let previous = inner.settings.data().search_scope;
            inner.settings.data_mut().search_scope = scope;
            if let Err(error) = inner.settings.save() {
                inner.settings.data_mut().search_scope = previous;
                return Err(error);
            }
        }
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn set_language(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    language: crate::locale::Language,
) -> Result<SnapshotDto, String> {
    crate::logging::run("set_language", || {
        {
            let mut inner = state.lock();
            let previous = inner.settings.data().language;
            inner.settings.data_mut().language = language;
            if let Err(error) = inner.settings.save() {
                inner.settings.data_mut().language = previous;
                return Err(error);
            }
        }
        crate::tray::refresh_menu(&app);
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn select_installation(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    index: usize,
) -> Result<SnapshotDto, String> {
    crate::logging::run("select_installation", || {
        state.select_installation(index)?;
        crate::tray::refresh_menu(&app);
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn set_masked(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    masked: bool,
) -> Result<SnapshotDto, String> {
    crate::logging::run("set_masked", || {
        state.set_privacy(Some(masked), None)?;
        crate::tray::refresh_menu(&app);
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn set_steam_ids_visible(
    state: State<'_, AppState>,
    visible: bool,
) -> Result<SnapshotDto, String> {
    crate::logging::run("set_steam_ids_visible", || {
        state.set_privacy(None, Some(visible))?;
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn update_settings(
    state: State<'_, AppState>,
    show_installation_selector: bool,
) -> Result<SnapshotDto, String> {
    crate::logging::run("update_settings", || {
        state.set_ui_settings(show_installation_selector)?;
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn set_installation_hidden(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    root: String,
    hidden: bool,
) -> Result<SnapshotDto, String> {
    crate::logging::run("set_installation_hidden", || {
        state.set_installation_hidden(&root, hidden)?;
        crate::tray::refresh_menu(&app);
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn add_custom_installation(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<SnapshotDto, String> {
    crate::logging::run("add_custom_installation", || {
        state.add_custom_installation(&path)?;
        crate::tray::refresh_menu(&app);
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn remove_custom_installation(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    root: String,
) -> Result<SnapshotDto, String> {
    crate::logging::run("remove_custom_installation", || {
        state.remove_custom_installation(&root)?;
        crate::tray::refresh_menu(&app);
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn create_folder(
    state: State<'_, AppState>,
    name: String,
    color: Option<String>,
    icon: Option<String>,
) -> Result<SnapshotDto, String> {
    crate::logging::run("create_folder", || {
        state.create_folder(&name, color.as_deref(), icon.as_deref())?;
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn update_folder(
    state: State<'_, AppState>,
    id: String,
    name: String,
    color: Option<String>,
    icon: Option<String>,
) -> Result<SnapshotDto, String> {
    crate::logging::run("update_folder", || {
        state.update_folder(&id, &name, color.as_deref(), icon.as_deref())?;
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn delete_folder(state: State<'_, AppState>, id: String) -> Result<SnapshotDto, String> {
    crate::logging::run("delete_folder", || {
        state.delete_folder(&id)?;
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn assign_folder(
    state: State<'_, AppState>,
    steam_id: String,
    folder_id: Option<String>,
) -> Result<SnapshotDto, String> {
    crate::logging::run("assign_folder", || {
        state.set_account_folder(&steam_id, folder_id.as_deref())?;
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn set_account_color(
    state: State<'_, AppState>,
    steam_id: String,
    color: Option<String>,
) -> Result<SnapshotDto, String> {
    crate::logging::run("set_account_color", || {
        state.set_account_color(&steam_id, color.as_deref())?;
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn set_folder_color(
    state: State<'_, AppState>,
    id: String,
    color: Option<String>,
) -> Result<SnapshotDto, String> {
    crate::logging::run("set_folder_color", || {
        state.set_folder_color(&id, color.as_deref())?;
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn set_folder_icon(
    state: State<'_, AppState>,
    id: String,
    icon: Option<String>,
) -> Result<SnapshotDto, String> {
    crate::logging::run("set_folder_icon", || {
        state.set_folder_icon(&id, icon.as_deref())?;
        Ok(build_snapshot(&state))
    })
}
#[tauri::command(async)]
pub fn set_window_active(state: State<'_, AppState>, active: bool) {
    state.set_window_active(active);
}

#[derive(Serialize)]
pub struct AppInfoDto {
    pub name: String,
    pub version: String,
    pub repository: String,
}
#[tauri::command(async)]
pub fn app_info(app: tauri::AppHandle) -> AppInfoDto {
    AppInfoDto {
        name: app.package_info().name.clone(),
        version: app.package_info().version.to_string(),
        repository: REPOSITORY_URL.to_string(),
    }
}

#[tauri::command(async)]
pub fn autostart_status(app: tauri::AppHandle) -> Result<bool, String> {
    crate::logging::read("autostart_status", || {
        use tauri_plugin_autostart::ManagerExt;
        app.autolaunch()
            .is_enabled()
            .map_err(|_| ErrorCode::AutostartFailed.to_string())
    })
}

#[tauri::command(async)]
pub fn set_autostart(app: tauri::AppHandle, enabled: bool) -> Result<bool, String> {
    crate::logging::run("set_autostart", || {
        use tauri_plugin_autostart::ManagerExt;
        let manager = app.autolaunch();
        let result = if enabled {
            manager.enable()
        } else {
            manager.disable()
        };
        result.map_err(|_| ErrorCode::AutostartFailed.to_string())?;
        manager
            .is_enabled()
            .map_err(|_| ErrorCode::AutostartFailed.to_string())
    })
}
