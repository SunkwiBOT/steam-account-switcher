use std::path::Path;

use steam_core::authenticator::{
    confirmation_action, confirmations, remove_authenticator, CodeInfo, ConfirmationItem,
};
use steam_core::error_code::ErrorCode;
use tauri::State;

use super::vault::vault_snapshot;
use crate::commands::SnapshotDto;
use crate::state::AppState;

#[tauri::command(async)]
pub fn steam_guard_code(state: State<'_, AppState>, steam_id: String) -> Result<CodeInfo, String> {
    crate::logging::read("steam_guard_code", || {
        let authenticator = state.authenticator();
        let guard = authenticator
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        guard.code(&steam_id).map_err(|error| error.to_string())
    })
}

#[tauri::command(async)]
pub fn steam_guard_revocation_code(
    state: State<'_, AppState>,
    steam_id: String,
) -> Result<String, String> {
    crate::logging::read("steam_guard_revocation_code", || {
        let authenticator = state.authenticator();
        let guard = authenticator
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        guard
            .revocation_code(&steam_id)
            .map_err(|error| error.to_string())
    })
}

#[tauri::command]
pub async fn steam_guard_import(
    state: State<'_, AppState>,
    steam_id: Option<String>,
    path: String,
    passkey: Option<String>,
) -> Result<SnapshotDto, String> {
    crate::logging::run_async("steam_guard_import", async {
        let authenticator = state.authenticator();
        let result = tauri::async_runtime::spawn_blocking(move || {
            let passkey = passkey.map(zeroize::Zeroizing::new);
            let mut guard = authenticator
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            guard
                .import(
                    Path::new(&path),
                    passkey.as_deref().map(|value| value.as_str()),
                    steam_id.as_deref(),
                )
                .map_err(|error| error.to_string())?;

            Ok::<(), String>(())
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())?;
        result?;
        Ok(vault_snapshot(&state))
    })
    .await
}

#[tauri::command]
pub async fn steam_guard_confirmations(
    state: State<'_, AppState>,
    steam_id: String,
) -> Result<Vec<ConfirmationItem>, String> {
    crate::logging::run_async("steam_guard_confirmations", async {
        let authenticator = state.authenticator();
        tauri::async_runtime::spawn_blocking(move || {
            let mut account = {
                let guard = authenticator
                    .lock()
                    .unwrap_or_else(|error| error.into_inner());
                guard
                    .confirmation_account(&steam_id)
                    .map_err(|error| error.to_string())?
            };
            let result = confirmations(&mut account).map_err(|error| error.to_string())?;
            authenticator
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .remember_tokens(&account)
                .map_err(|error| error.to_string())?;
            Ok(result)
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())?
    })
    .await
}

#[tauri::command]
pub async fn steam_guard_confirmation_action(
    state: State<'_, AppState>,
    steam_id: String,
    id: String,
    nonce: String,
    accept: bool,
) -> Result<(), String> {
    crate::logging::run_async("steam_guard_confirmation_action", async {
        let authenticator = state.authenticator();
        tauri::async_runtime::spawn_blocking(move || {
            let mut account = {
                let guard = authenticator
                    .lock()
                    .unwrap_or_else(|error| error.into_inner());
                guard
                    .confirmation_account(&steam_id)
                    .map_err(|error| error.to_string())?
            };
            confirmation_action(&mut account, &id, &nonce, accept)
                .map_err(|error| error.to_string())?;
            // An accepted action must not be reported as failed just because
            // refreshing the list or saving refreshed tokens failed afterwards.
            if let Err(error) = authenticator
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .remember_tokens(&account)
            {
                crate::logging::write(&format!(
                    "action=steam_guard_confirmation_action stage=save_session status=failed error={}",
                    crate::logging::error_category(&error.to_string())
                ));
            }
            Ok::<(), String>(())
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())?
    })
    .await
}

#[tauri::command]
pub async fn steam_guard_remove(
    state: State<'_, AppState>,
    steam_id: String,
    revocation_code: Option<String>,
) -> Result<SnapshotDto, String> {
    crate::logging::run_async("steam_guard_remove", async {
        let authenticator = state.authenticator();
        let result = tauri::async_runtime::spawn_blocking(move || {
            let mut account = {
                let guard = authenticator
                    .lock()
                    .unwrap_or_else(|error| error.into_inner());
                guard
                    .confirmation_account(&steam_id)
                    .map_err(|error| error.to_string())?
            };
            remove_authenticator(&mut account, revocation_code)
                .map_err(|error| error.to_string())?;
            authenticator
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .forget_revoked(&steam_id)
                .map_err(|error| error.to_string())
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())?;
        result?;
        Ok(vault_snapshot(&state))
    })
    .await
}
