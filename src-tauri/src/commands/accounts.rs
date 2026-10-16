use super::{build_snapshot, dto::SnapshotDto};
use crate::state::AppState;
use steam_core::error_code::ErrorCode;
use steam_core::model::SteamAccount;
use tauri::State;

pub(crate) fn login_name(state: &AppState, steam_id: &str) -> Result<String, String> {
    login_name_for(&state.display_accounts(), steam_id)
}

fn login_name_for(accounts: &[SteamAccount], steam_id: &str) -> Result<String, String> {
    accounts
        .iter()
        .find(|account| account.steam_id == steam_id)
        .map(|account| account.account_name.clone())
        .filter(|name| !name.is_empty())
        .ok_or_else(|| ErrorCode::AccountUnavailable.as_str().to_string())
}
#[tauri::command(async)]
pub fn account_login_name(state: State<'_, AppState>, steam_id: String) -> Result<String, String> {
    crate::logging::read("account_login_name", || login_name(&state, &steam_id))
}

#[tauri::command]
pub async fn add_steam_account(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<SnapshotDto, String> {
    crate::logging::run_async("add_steam_account", async {
        let operation = state.begin_steam_operation()?;
        let manager = state
            .lock()
            .manager
            .clone()
            .ok_or_else(|| ErrorCode::NoSteamInstallation.as_str().to_string())?;

        let result = tauri::async_runtime::spawn_blocking(move || {
            let _operation = operation;
            manager.add_account()
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())?;

        match result {
            Ok(()) => {
                state.refresh_accounts();
                crate::tray::refresh_menu(&app);
                Ok(build_snapshot(&state))
            }
            Err(error) => Err(error.to_string()),
        }
    })
    .await
}

#[tauri::command]
pub async fn running_steam_games(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    crate::logging::run_async("running_steam_games", async {
        let installations = {
            let inner = state.lock();
            let manager = inner
                .manager
                .as_ref()
                .ok_or_else(|| ErrorCode::NoSteamInstallation.to_string())?;
            inner
                .detected
                .iter()
                .chain(&inner.custom)
                .chain(std::iter::once(&manager.installation))
                .map(|installation| (crate::state::root_key(installation), installation.clone()))
                .collect::<std::collections::BTreeMap<_, _>>()
        };
        tauri::async_runtime::spawn_blocking(move || {
            let mut monitor = steam_core::processes::ProcessMonitor::new();
            monitor.refresh();
            let monitor = std::sync::Mutex::new(monitor);
            let mut games = std::collections::BTreeMap::new();
            for installation in installations.into_values() {
                let events = steam_core::playtime::running_game_events(
                    &installation.root.join("logs/gameprocess_log.txt"),
                    &|pid| {
                        monitor
                            .lock()
                            .unwrap_or_else(|error| error.into_inner())
                            .process_facts(pid)
                    },
                )
                .map_err(|error| error.to_string())?;
                let catalog = steam_core::games::SteamGameCatalog::new(&installation);
                for event in events {
                    games
                        .entry(event.app_id.clone())
                        .or_insert_with(|| catalog.display_name_for(&event.app_id));
                }
            }
            Ok(games.into_values().collect())
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())?
    })
    .await
}

#[tauri::command]
pub async fn switch_account(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    steam_id: String,
) -> Result<SnapshotDto, String> {
    crate::logging::run_async("switch_account", async {
        let operation = state.begin_steam_operation()?;
        let manager = state
            .lock()
            .manager
            .clone()
            .ok_or_else(|| ErrorCode::NoSteamInstallation.as_str().to_string())?;

        let running = manager.is_steam_running();
        crate::logging::write(&format!("action=switch_account steam_running={running}"));

        let identifier = steam_id.clone();
        let result = tauri::async_runtime::spawn_blocking(move || {
            let _operation = operation;
            if manager.find_account(&identifier)?.is_none() {
                return Err(steam_core::error::SteamError::config(
                    ErrorCode::AccountNotLocal,
                ));
            }
            manager.switch_account(&identifier, true)
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())?;

        match result {
            Ok(outcome) => {
                crate::logging::write(&format!(
                    "action=switch_account already_active={} started={}",
                    outcome.already_active(),
                    outcome.started()
                ));
                state.refresh_accounts();
                crate::tray::refresh_menu(&app);
                Ok(build_snapshot(&state))
            }
            Err(error) => Err(error.to_string()),
        }
    })
    .await
}

#[tauri::command]
pub async fn remove_account(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    steam_id: String,
    cleanup_userdata: bool,
    confirmed: bool,
) -> Result<SnapshotDto, String> {
    let snapshot = remove_account_for(&state, &steam_id, cleanup_userdata, confirmed).await?;
    crate::tray::refresh_menu(&app);
    Ok(snapshot)
}

async fn remove_account_for(
    state: &AppState,
    steam_id: &str,
    cleanup_userdata: bool,
    confirmed: bool,
) -> Result<SnapshotDto, String> {
    crate::logging::run_async("remove_account", async {
        if !confirmed {
            return Err(ErrorCode::ConfirmationRequired.as_str().to_string());
        }
        let operation = state.begin_steam_operation()?;
        state.display_accounts();
        let manager = state
            .lock()
            .manager
            .clone()
            .ok_or_else(|| ErrorCode::NoSteamInstallation.as_str().to_string())?;

        let identifier = steam_id.to_string();
        let result = tauri::async_runtime::spawn_blocking(move || {
            let _operation = operation;
            manager.remove_account(&identifier, cleanup_userdata)
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())?;

        match result {
            Ok(()) => {
                state.refresh_accounts();
                Ok(build_snapshot(state))
            }
            Err(error) => Err(error.to_string()),
        }
    })
    .await
}

#[tauri::command]
pub async fn forget_saved_account(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    steam_id: String,
    confirmed: bool,
) -> Result<SnapshotDto, String> {
    let snapshot = forget_saved_account_for(&state, &steam_id, confirmed).await?;
    crate::tray::refresh_menu(&app);
    Ok(snapshot)
}

async fn forget_saved_account_for(
    state: &AppState,
    steam_id: &str,
    confirmed: bool,
) -> Result<SnapshotDto, String> {
    crate::logging::run_async("forget_saved_account", async {
        if !confirmed {
            return Err(ErrorCode::ConfirmationRequired.as_str().to_string());
        }
        let operation = state.begin_steam_operation()?;
        let manager = {
            let inner = state.lock();
            if inner
                .accounts
                .iter()
                .any(|account| account.steam_id == steam_id)
            {
                return Err(ErrorCode::AccountLocal.as_str().to_string());
            }
            inner.manager.clone()
        };
        let authenticator = state.authenticator();
        let id = steam_id.to_string();
        tauri::async_runtime::spawn_blocking(move || {
            let _operation = operation;
            if let Some(manager) = manager {
                if manager
                    .find_account(&id)
                    .map_err(|error| error.to_string())?
                    .is_some()
                {
                    return Err(ErrorCode::AccountLocal.as_str().to_string());
                }
            }
            let mut guard = authenticator
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            guard.forget_local(&id).map_err(|error| error.to_string())
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())??;
        {
            let mut inner = state.lock();
            let previous = inner.settings.data().clone();
            let settings = inner.settings.data_mut();
            settings.guard_account_profiles.remove(steam_id);
            settings.account_folders.remove(steam_id);
            settings.account_colors.remove(steam_id);
            settings.account_notes.remove(steam_id);
            if let Err(error) = inner.settings.save() {
                *inner.settings.data_mut() = previous;
                crate::logging::write(&format!(
                    "action=forget_saved_profile status=failed error={}",
                    crate::logging::error_category(&error)
                ));
            }
        }
        Ok(build_snapshot(state))
    })
    .await
}

#[tauri::command]
pub async fn restart_steam(state: State<'_, AppState>) -> Result<SnapshotDto, String> {
    crate::logging::run_async("restart_steam", async {
        let operation = state.begin_steam_operation()?;
        let manager = state
            .lock()
            .manager
            .clone()
            .ok_or_else(|| ErrorCode::NoSteamInstallation.as_str().to_string())?;

        let result = tauri::async_runtime::spawn_blocking(move || {
            let _operation = operation;
            manager.restart_steam()
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())?;

        match result {
            Ok(()) => Ok(build_snapshot(&state)),
            Err(error) => Err(error.to_string()),
        }
    })
    .await
}

#[tauri::command(async)]
pub fn set_rank(
    state: State<'_, AppState>,
    steam_id: String,
    template: String,
    rank: String,
) -> Result<SnapshotDto, String> {
    crate::logging::run("set_rank", || {
        let (account, ranks) = {
            let inner = state.lock();
            let account = inner
                .accounts
                .iter()
                .find(|account| account.steam_id == steam_id)
                .cloned()
                .ok_or_else(|| ErrorCode::AccountUnavailable.to_string())?;
            let ranks = inner
                .ranks
                .clone()
                .ok_or_else(|| ErrorCode::RankStoreUnavailable.as_str().to_string())?;
            (account, ranks)
        };

        let mut ranks = ranks;
        ranks
            .set_rank(&account, &template, &rank)
            .map_err(|error| error.to_string())?;
        {
            let mut inner = state.lock();
            inner.ranks = Some(ranks);
        }
        Ok(build_snapshot(&state))
    })
}
