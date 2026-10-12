use steam_core::authenticator::{AuthError, Authenticator, StoredAuthenticator, VaultStatus};
use steam_core::error_code::ErrorCode;
use tauri::State;

use crate::commands::{build_snapshot, SnapshotDto};
use crate::state::AppState;

pub(super) fn vault_snapshot(state: &AppState) -> SnapshotDto {
    let snapshot = build_snapshot(state);
    let vault = &snapshot.vault;
    crate::logging::write(&format!(
        "vault: encrypted={} locked={} indexed={} accounts={}",
        vault.status.encrypted, vault.status.locked, vault.status.indexed, vault.status.count
    ));
    snapshot
}

#[tauri::command(async)]
pub fn authenticator_status(state: State<'_, AppState>) -> VaultStatus {
    let authenticator = state.authenticator();
    let guard = authenticator
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    guard.status()
}

#[tauri::command(async)]
pub fn authenticator_accounts(state: State<'_, AppState>) -> Vec<StoredAuthenticator> {
    let authenticator = state.authenticator();
    let guard = authenticator
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    guard.accounts()
}

async fn update_vault(
    state: &AppState,
    action: impl FnOnce(&mut Authenticator) -> Result<(), AuthError> + Send + 'static,
) -> Result<SnapshotDto, String> {
    let authenticator = state.authenticator();
    tauri::async_runtime::spawn_blocking(move || {
        let mut guard = authenticator
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        action(&mut guard).map_err(|error| error.to_string())
    })
    .await
    .map_err(|_| ErrorCode::WorkerFailed.to_string())??;
    // ! vault_snapshot locks the authenticator again: keep it outside the blocking closure.
    let snapshot = vault_snapshot(state);
    if snapshot.vault.status.encrypted && !snapshot.vault.status.locked {
        state.note_vault_unlocked();
    } else {
        state.note_vault_locked();
    }
    Ok(snapshot)
}

#[tauri::command]
pub async fn authenticator_unlock(
    state: State<'_, AppState>,
    passphrase: String,
) -> Result<SnapshotDto, String> {
    crate::logging::run_async("authenticator_unlock", async {
        update_vault(&state, move |guard| {
            let passphrase = zeroize::Zeroizing::new(passphrase);
            guard.unlock(&passphrase).map(|_| ())
        })
        .await
    })
    .await
}

#[tauri::command]
pub async fn authenticator_lock(state: State<'_, AppState>) -> Result<SnapshotDto, String> {
    crate::logging::run_async("authenticator_lock", async {
        update_vault(&state, |guard| guard.lock()).await
    })
    .await
}

#[tauri::command]
pub async fn authenticator_set_passphrase(
    state: State<'_, AppState>,
    passphrase: Option<String>,
    current_password: Option<String>,
) -> Result<SnapshotDto, String> {
    crate::logging::run_async("authenticator_set_passphrase", async {
        update_vault(&state, move |guard| {
            guard
                .set_passphrase(passphrase, current_password)
                .map(|_| ())
        })
        .await
    })
    .await
}
