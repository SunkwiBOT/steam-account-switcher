use steam_core::authenticator::{AuthError, LinkStart};
use steam_core::error_code::ErrorCode;
use tauri::State;

use super::vault::vault_snapshot;
use crate::commands::SnapshotDto;
use crate::state::AppState;

#[tauri::command]
pub async fn steam_guard_link_start(
    state: State<'_, AppState>,
    steam_id: String,
) -> Result<LinkStart, String> {
    crate::logging::run_async("steam_guard_link_start", async {
        let numeric: u64 = steam_id
            .parse()
            .map_err(|_| ErrorCode::InvalidSteamId.to_string())?;
        let account_name = {
            let inner = state.lock();
            inner
                .accounts
                .iter()
                .find(|account| account.steam_id == steam_id)
                .map(|account| account.account_name.clone())
                .unwrap_or_else(|| steam_id.clone())
        };

        let authenticator = state.authenticator();
        tauri::async_runtime::spawn_blocking(move || {
            let (session_id, mut pending) = authenticator
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .prepare_link(numeric, &account_name)
                .map_err(|error| error.to_string())?;
            let outcome = pending.start(&session_id);
            let mut guard = authenticator
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            match outcome {
                Ok(result) => {
                    guard
                        .store_link(&session_id, pending)
                        .map_err(|error| error.to_string())?;
                    Ok(result)
                }
                Err(error) => {
                    guard.finish_link_operation(&session_id);
                    Err(error.to_string())
                }
            }
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())?
    })
    .await
}

#[tauri::command]
pub async fn steam_guard_link_finalize(
    state: State<'_, AppState>,
    session_id: String,
    confirm_code: String,
) -> Result<SnapshotDto, String> {
    crate::logging::run_async("steam_guard_link_finalize", async {
        let authenticator = state.authenticator();
        let mut pending = {
            let mut guard = authenticator
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            guard
                .take_linker(&session_id)
                .map_err(|error| error.to_string())?
        };

        let session = session_id.clone();
        let (pending, outcome) = tauri::async_runtime::spawn_blocking(move || {
            let result = (|| {
                pending.finalize(&confirm_code)?;
                pending.verify()?;
                pending
                    .take_account()
                    .ok_or(AuthError::Code(ErrorCode::AuthenticatorMissing))
            })();
            (pending, result)
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())?;

        match outcome {
            Ok(account) => {
                let mut guard = authenticator
                    .lock()
                    .unwrap_or_else(|error| error.into_inner());
                guard
                    .absorb_link(&session, account)
                    .map_err(|error| error.to_string())?;
                drop(guard);
                Ok(vault_snapshot(&state))
            }
            Err(error) => {
                let mut guard = authenticator
                    .lock()
                    .unwrap_or_else(|error| error.into_inner());
                guard.put_linker(&session, pending);
                Err(error.to_string())
            }
        }
    })
    .await
}

#[tauri::command]
pub async fn steam_guard_link_transfer_start(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    crate::logging::run_async("steam_guard_link_transfer_start", async {
        let authenticator = state.authenticator();
        let mut pending = {
            let mut guard = authenticator
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            guard
                .take_linker(&session_id)
                .map_err(|error| error.to_string())?
        };

        let session = session_id.clone();
        let (pending, outcome) = tauri::async_runtime::spawn_blocking(move || {
            let result = pending.transfer_start();
            (pending, result)
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())?;

        let mut guard = authenticator
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        guard.put_linker(&session, pending);
        outcome.map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub async fn steam_guard_link_transfer_finish(
    state: State<'_, AppState>,
    session_id: String,
    sms_code: String,
) -> Result<SnapshotDto, String> {
    crate::logging::run_async("steam_guard_link_transfer_finish", async {
        let authenticator = state.authenticator();
        let mut pending = {
            let mut guard = authenticator
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            guard
                .take_linker(&session_id)
                .map_err(|error| error.to_string())?
        };

        let session = session_id.clone();
        let (pending, outcome) = tauri::async_runtime::spawn_blocking(move || {
            let result = pending.transfer_finish(&sms_code);
            (pending, result)
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())?;

        match outcome {
            Ok(account) => {
                let mut guard = authenticator
                    .lock()
                    .unwrap_or_else(|error| error.into_inner());
                guard
                    .absorb_link(&session, account)
                    .map_err(|error| error.to_string())?;
                drop(guard);
                Ok(vault_snapshot(&state))
            }
            Err(error) => {
                let mut guard = authenticator
                    .lock()
                    .unwrap_or_else(|error| error.into_inner());
                guard.put_linker(&session, pending);
                Err(error.to_string())
            }
        }
    })
    .await
}

#[tauri::command]
pub async fn steam_guard_link_revoke(
    state: State<'_, AppState>,
    session_id: String,
    revocation_code: String,
) -> Result<(), String> {
    crate::logging::run_async("steam_guard_link_revoke", async {
        let authenticator = state.authenticator();
        let mut pending = {
            let mut guard = authenticator
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            guard
                .take_linker(&session_id)
                .map_err(|error| error.to_string())?
        };

        let session = session_id.clone();
        let (pending, outcome) = tauri::async_runtime::spawn_blocking(move || {
            let result = pending.revoke(&revocation_code);
            (pending, result)
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())?;

        if outcome.is_ok() {
            let mut guard = authenticator
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            guard.finish_link_operation(&session);
            guard.cancel_session(&session);
        } else {
            let mut guard = authenticator
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            guard.put_linker(&session, pending);
        }
        outcome.map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command(async)]
pub fn steam_guard_link_cancel(state: State<'_, AppState>, session_id: String) {
    crate::logging::write("action=steam_guard_link_cancel status=cancelled");
    let authenticator = state.authenticator();
    let mut guard = authenticator
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    guard.cancel_session(&session_id);
}
