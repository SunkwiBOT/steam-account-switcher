use super::errors::{confirmation_error, steam_error, AuthError};
use super::transport::transport;
use super::types::{ConfirmationItem, ConfirmationKind};
use steamguard::refresher::TokenRefresher;
use steamguard::steamapi::AuthenticationClient;
use steamguard::transport::TransportError;
use steamguard::{Confirmation, Confirmer, ConfirmerError, SteamGuardAccount};
use zeroize::Zeroizing;

pub(super) fn with_valid_tokens<T>(
    account: &mut SteamGuardAccount,
    action: impl Fn(&SteamGuardAccount) -> Result<T, ConfirmerError>,
) -> Result<T, AuthError> {
    match action(account) {
        Ok(value) => Ok(value),
        Err(ConfirmerError::InvalidTokens) => {
            refresh_tokens(account)?;
            action(account).map_err(confirmation_error)
        }
        Err(error) => Err(confirmation_error(error)),
    }
}

pub fn confirmations(account: &mut SteamGuardAccount) -> Result<Vec<ConfirmationItem>, AuthError> {
    let transport = transport();
    let list = with_valid_tokens(account, |account| {
        Confirmer::new(transport.clone(), account).get_confirmations()
    })?;
    Ok(list.iter().map(confirmation_item).collect())
}

pub fn confirmation_action(
    account: &mut SteamGuardAccount,
    id: &str,
    nonce: &str,
    accept: bool,
) -> Result<(), AuthError> {
    let transport = transport();
    let confirmation_id = steamguard::ConfirmationId::new(id, nonce);
    with_valid_tokens(account, |account| {
        let confirmer = Confirmer::new(transport.clone(), account);
        if accept {
            confirmer.accept_confirmation(confirmation_id)
        } else {
            confirmer.deny_confirmation(confirmation_id)
        }
    })
}

pub fn remove_authenticator(
    account: &mut SteamGuardAccount,
    revocation_code: Option<String>,
) -> Result<(), AuthError> {
    use steamguard::ExposeSecret;
    let code = Zeroizing::new(
        revocation_code
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| account.revocation_code.expose_secret().to_string()),
    );
    let transport = transport();
    let result = account.remove_authenticator(transport.clone(), Some(&code));
    let result = match result {
        Err(steamguard::accountlinker::RemoveAuthenticatorError::TransportError(
            TransportError::Unauthorized,
        )) => {
            refresh_tokens(account)?;
            account.remove_authenticator(transport, Some(&code))
        }
        other => other,
    };
    result.map_err(steam_error)
}

pub(super) fn refresh_tokens(account: &mut SteamGuardAccount) -> Result<(), AuthError> {
    let tokens = account.tokens.clone().ok_or(AuthError::NeedsLogin)?;
    let mut refresher = TokenRefresher::new(AuthenticationClient::new(transport()));
    let jwt = refresher
        .refresh(account.steam_id, &tokens)
        .map_err(steam_error)?;
    let mut updated = tokens;
    updated.set_access_token(jwt);
    account.tokens = Some(updated);
    Ok(())
}

pub(super) fn confirmation_item(confirmation: &Confirmation) -> ConfirmationItem {
    let kind = match confirmation.conf_type {
        steamguard::ConfirmationType::Trade => ConfirmationKind::Trade,
        steamguard::ConfirmationType::MarketSell => ConfirmationKind::Market,
        steamguard::ConfirmationType::PhoneNumberChange => ConfirmationKind::Phone,
        steamguard::ConfirmationType::AccountRecovery => ConfirmationKind::Recovery,
        steamguard::ConfirmationType::ApiKeyCreation => ConfirmationKind::ApiKey,
        steamguard::ConfirmationType::FeatureOptOut => ConfirmationKind::Feature,
        steamguard::ConfirmationType::JoinSteamFamily => ConfirmationKind::Family,
        steamguard::ConfirmationType::Test => ConfirmationKind::Test,
        steamguard::ConfirmationType::Unknown(_) => ConfirmationKind::Other,
    };
    ConfirmationItem {
        id: confirmation.id.clone(),
        nonce: confirmation.nonce.clone(),
        kind,
        headline: confirmation.headline.clone(),
        summary: confirmation.summary.join("\n"),
        creator_id: confirmation.creator_id.clone(),
        creation_time: confirmation.creation_time,
        icon: confirmation.icon.clone(),
        multi: confirmation.multi,
    }
}
