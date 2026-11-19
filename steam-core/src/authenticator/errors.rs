use crate::error_code::ErrorCode;
use steamguard::transport::TransportError;
use steamguard::ConfirmerError;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("{0}")]
    Code(ErrorCode),
    #[error("VAULT_LOCKED")]
    Locked,
    #[error("AUTHENTICATOR_MISSING")]
    Missing,
    #[error("WRONG_VAULT_PASSWORD")]
    WrongPassphrase,
    #[error("INVALID_VAULT")]
    InvalidVault,
    #[error("INVALID_BACKUP")]
    InvalidBackup,
    #[error("INVALID_BACKUP_MANIFEST")]
    InvalidBackupManifest,
    #[error("BACKUP_DECRYPTION_FAILED")]
    BackupDecryptionFailed,
    #[error("INVALID_AUTHENTICATOR_DATA")]
    InvalidData,
    #[error("SESSION_EXPIRED")]
    UnknownSession,
    #[error("SIGN_IN_REQUIRED")]
    NeedsLogin,
    #[error("AUTHENTICATOR_BUSY")]
    Busy,
    #[error("STEAM_ERROR")]
    Steam(String),
    #[error("NETWORK_ERROR")]
    Network(String),
    #[error("IO_ERROR")]
    Io(String),
}

impl AuthError {
    // ! The stable, translatable identifier for this error; crosses IPC to the interface's message tables.
    pub fn code(&self) -> ErrorCode {
        match self {
            AuthError::Code(code) => *code,
            AuthError::Locked => ErrorCode::VaultLocked,
            AuthError::Missing => ErrorCode::AuthenticatorMissing,
            AuthError::WrongPassphrase => ErrorCode::WrongVaultPassword,
            AuthError::InvalidVault => ErrorCode::InvalidVault,
            AuthError::InvalidBackup => ErrorCode::InvalidBackup,
            AuthError::InvalidBackupManifest => ErrorCode::InvalidBackupManifest,
            AuthError::BackupDecryptionFailed => ErrorCode::BackupDecryptionFailed,
            AuthError::InvalidData => ErrorCode::InvalidAuthenticatorData,
            AuthError::UnknownSession => ErrorCode::SessionExpired,
            AuthError::NeedsLogin => ErrorCode::SignInRequired,
            AuthError::Busy => ErrorCode::AuthenticatorBusy,
            AuthError::Steam(_) => ErrorCode::SteamError,
            AuthError::Network(_) => ErrorCode::NetworkError,
            AuthError::Io(_) => ErrorCode::IoError,
        }
    }

    // ! The cause, for logs only, redacted by the logging filter before it is written; the interface only ever shows `code()`'s stable string to the user.
    pub fn detail(&self) -> Option<&str> {
        match self {
            AuthError::Steam(detail) | AuthError::Network(detail) | AuthError::Io(detail) => {
                Some(detail)
            }
            _ => None,
        }
    }
}

pub(super) fn io_error(error: std::io::Error) -> AuthError {
    AuthError::Io(error.to_string())
}

pub(super) fn steam_error(error: impl std::fmt::Display) -> AuthError {
    AuthError::Steam(error.to_string())
}

pub(super) fn transport_code(error: &TransportError) -> ErrorCode {
    match error {
        TransportError::Unauthorized => ErrorCode::SignInRequired,
        TransportError::NetworkFailure(_) => ErrorCode::NetworkError,
        _ => ErrorCode::SteamError,
    }
}

// ! Strips the request URL (which carries `access_token` in its query string) from a `reqwest::Error` before it is ever formatted.
pub(super) fn transport_error(error: TransportError) -> AuthError {
    let code = transport_code(&error);
    let detail = match (&code, error) {
        (ErrorCode::NetworkError, TransportError::NetworkFailure(error)) => {
            error.without_url().to_string()
        }
        (_, error) => error.to_string(),
    };
    match code {
        ErrorCode::NetworkError => AuthError::Network(detail),
        ErrorCode::SignInRequired => AuthError::NeedsLogin,
        _ => AuthError::Steam(detail),
    }
}

pub(super) fn login_error(error: steamguard::userlogin::LoginError) -> AuthError {
    use steamguard::userlogin::LoginError;
    match error {
        LoginError::BadCredentials => AuthError::Code(ErrorCode::WrongSteamPassword),
        LoginError::TooManyAttempts => AuthError::Code(ErrorCode::SteamRateLimited),
        LoginError::SessionExpired => AuthError::Code(ErrorCode::SessionExpired),
        LoginError::TransportError(error) => transport_error(error),
        LoginError::NetworkFailure(error) => AuthError::Network(error.without_url().to_string()),
        other => AuthError::Steam(other.to_string()),
    }
}

pub(super) fn login_code_error(error: steamguard::userlogin::UpdateAuthSessionError) -> AuthError {
    use steamguard::userlogin::UpdateAuthSessionError;
    match error {
        UpdateAuthSessionError::IncorrectSteamGuardCode => {
            AuthError::Code(ErrorCode::WrongGuardCode)
        }
        UpdateAuthSessionError::TooManyAttempts => AuthError::Code(ErrorCode::SteamRateLimited),
        UpdateAuthSessionError::SessionExpired | UpdateAuthSessionError::SessionNotStarted => {
            AuthError::Code(ErrorCode::SessionExpired)
        }
        UpdateAuthSessionError::TransportError(error) => transport_error(error),
        UpdateAuthSessionError::NetworkFailure(error) => {
            AuthError::Network(error.without_url().to_string())
        }
        other => AuthError::Steam(other.to_string()),
    }
}

pub(super) fn confirmation_error(error: ConfirmerError) -> AuthError {
    match error {
        ConfirmerError::InvalidTokens => AuthError::Code(ErrorCode::SignInRequired),
        ConfirmerError::NetworkFailure(error) => {
            AuthError::Network(error.without_url().to_string())
        }
        other => AuthError::Steam(other.to_string()),
    }
}
