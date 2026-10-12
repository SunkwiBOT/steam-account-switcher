use steam_core::authenticator::{
    begin_password_login, begin_qr_login, finish_login, qr_svg, submit_login_code, GuardKind,
    LoginBegin, LoginResult,
};
use steam_core::error_code::ErrorCode;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn steam_guard_login_password(
    state: State<'_, AppState>,
    steam_id: String,
    password: String,
) -> Result<LoginBegin, String> {
    crate::logging::run_async("steam_guard_login_password", async {
        let account_name = crate::commands::login_name(&state, &steam_id)?;
        let authenticator = state.authenticator();
        tauri::async_runtime::spawn_blocking(move || {
            let password = zeroize::Zeroizing::new(password);
            let (session_id, mut login) = authenticator
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .prepare_login()
                .map_err(|error| error.to_string())?;
            let outcome = begin_password_login(&mut login, &account_name, &password);
            let mut guard = authenticator
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            match outcome {
                Ok(methods) => {
                    guard
                        .store_login(&session_id, login)
                        .map_err(|error| error.to_string())?;
                    Ok(LoginBegin {
                        session_id,
                        methods,
                        qr_svg: None,
                        challenge_url: None,
                    })
                }
                Err(error) => {
                    guard.cancel_session(&session_id);
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
pub async fn steam_guard_login_qr(state: State<'_, AppState>) -> Result<LoginBegin, String> {
    crate::logging::run_async("steam_guard_login_qr", async {
        let authenticator = state.authenticator();
        tauri::async_runtime::spawn_blocking(move || {
            let (session_id, mut login) = authenticator
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .prepare_login()
                .map_err(|error| error.to_string())?;
            let outcome = begin_qr_login(&mut login)
                .and_then(|(url, methods)| Ok((qr_svg(&url)?, url, methods)));
            let mut guard = authenticator
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            match outcome {
                Ok((svg, url, methods)) => {
                    guard
                        .store_login(&session_id, login)
                        .map_err(|error| error.to_string())?;
                    Ok(LoginBegin {
                        session_id,
                        methods,
                        qr_svg: Some(svg),
                        challenge_url: Some(url),
                    })
                }
                Err(error) => {
                    guard.cancel_session(&session_id);
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
pub async fn steam_guard_login_finish(
    state: State<'_, AppState>,
    session_id: String,
    guard_kind: Option<GuardKind>,
    code: Option<String>,
    steam_id: String,
) -> Result<LoginResult, String> {
    crate::logging::run_async("steam_guard_login_finish", async {
        let account_name = crate::commands::login_name(&state, &steam_id)?;
        let authenticator = state.authenticator();
        let mut login = {
            let mut guard = authenticator
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            guard
                .take_login(&session_id)
                .map_err(|error| error.to_string())?
        };

        let session = session_id.clone();
        let (login, outcome) = tauri::async_runtime::spawn_blocking(move || {
            let result = (|| {
                if let (Some(kind), Some(code)) = (guard_kind, code.as_deref()) {
                    if kind.needs_code() {
                        submit_login_code(&mut login, kind, code)?;
                    }
                }
                finish_login(&mut login)
            })();
            (login, result)
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())?;

        match outcome {
            Ok(tokens) => {
                let mut guard = authenticator
                    .lock()
                    .unwrap_or_else(|error| error.into_inner());
                if steam_core::authenticator::token_steam_id(&tokens)
                    .map_err(|error| error.to_string())?
                    .to_string()
                    != steam_id
                {
                    guard.cancel_session(&session);
                    return Err(ErrorCode::AccountMismatch.as_str().to_string());
                }
                guard
                    .absorb_login(&session, tokens, &account_name)
                    .map_err(|error| error.to_string())
            }
            Err(error) => {
                let mut guard = authenticator
                    .lock()
                    .unwrap_or_else(|error| error.into_inner());
                guard.put_login(&session, login);
                Err(error.to_string())
            }
        }
    })
    .await
}

#[tauri::command(async)]
pub fn steam_guard_login_cancel(state: State<'_, AppState>, session_id: String) {
    crate::logging::write("action=steam_guard_login_cancel status=cancelled");
    let authenticator = state.authenticator();
    let mut guard = authenticator
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    guard.cancel_session(&session_id);
}
