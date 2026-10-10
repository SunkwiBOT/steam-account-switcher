use super::*;

pub fn begin_password_login(
    login: &mut Login,
    account_name: &str,
    password: &str,
) -> Result<Vec<GuardMethod>, AuthError> {
    let methods = login
        .begin_auth_via_credentials(account_name, password)
        .map_err(login_error)?;
    Ok(methods
        .iter()
        .map(|method| GuardMethod {
            kind: GuardKind::from_proto(method.confirmation_type),
            message: method.associated_messsage.clone(),
        })
        .collect())
}

pub fn begin_qr_login(login: &mut Login) -> Result<(String, Vec<GuardMethod>), AuthError> {
    let response = login.begin_auth_via_qr().map_err(login_error)?;
    let methods = response
        .confirmation_methods()
        .iter()
        .map(|method| GuardMethod {
            kind: GuardKind::from_proto(method.confirmation_type),
            message: method.associated_messsage.clone(),
        })
        .collect();
    Ok((response.challenge_url().clone(), methods))
}

pub fn submit_login_code(login: &mut Login, kind: GuardKind, code: &str) -> Result<(), AuthError> {
    login
        .submit_steam_guard_code(kind.to_proto(), code.to_string())
        .map_err(login_code_error)?;
    Ok(())
}

/// Waits for Steam to hand out the tokens. Blocks until the phone approves or
/// the code is accepted.
pub fn finish_login(login: &mut Login) -> Result<Tokens, AuthError> {
    login.poll_until_tokens().map_err(steam_error)
}

pub fn token_steam_id(tokens: &Tokens) -> Result<u64, AuthError> {
    for token in [tokens.access_token(), tokens.refresh_token()] {
        if let Some(payload) = token.expose_secret().split('.').nth(1) {
            let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(payload);
            if let Ok(bytes) = decoded {
                if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                    if let Some(id) = value
                        .get("sub")
                        .and_then(|value| value.as_str())
                        .and_then(|value| value.parse::<u64>().ok())
                        .filter(|id| *id != 0)
                    {
                        return Ok(id);
                    }
                }
            }
        }
    }
    Err(AuthError::Code(ErrorCode::InvalidSteamId))
}

pub fn qr_svg(data: &str) -> Result<String, AuthError> {
    let code = qrcode::QrCode::new(data.as_bytes())
        .map_err(|_| AuthError::Code(ErrorCode::QrCodeFailed))?;
    Ok(code
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(232, 232)
        .dark_color(qrcode::render::svg::Color("#0f172a"))
        .light_color(qrcode::render::svg::Color("#ffffff"))
        .build())
}
