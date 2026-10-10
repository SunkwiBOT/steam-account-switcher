use super::*;

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

pub(super) fn io_error(error: std::io::Error) -> AuthError {
    AuthError::Io(error.to_string())
}

pub(super) fn steam_error(_error: impl std::fmt::Display) -> AuthError {
    AuthError::Code(ErrorCode::SteamError)
}

pub(super) fn transport_code(error: &TransportError) -> ErrorCode {
    match error {
        TransportError::Unauthorized => ErrorCode::SignInRequired,
        TransportError::NetworkFailure(_) => ErrorCode::NetworkError,
        _ => ErrorCode::SteamError,
    }
}

pub(super) fn login_error(error: steamguard::userlogin::LoginError) -> AuthError {
    use steamguard::userlogin::LoginError;
    AuthError::Code(match error {
        LoginError::BadCredentials => ErrorCode::WrongSteamPassword,
        LoginError::TooManyAttempts => ErrorCode::SteamRateLimited,
        LoginError::SessionExpired => ErrorCode::SessionExpired,
        LoginError::TransportError(error) => transport_code(&error),
        LoginError::NetworkFailure(_) => ErrorCode::NetworkError,
        _ => ErrorCode::SteamError,
    })
}

pub(super) fn login_code_error(error: steamguard::userlogin::UpdateAuthSessionError) -> AuthError {
    use steamguard::userlogin::UpdateAuthSessionError;
    AuthError::Code(match error {
        UpdateAuthSessionError::IncorrectSteamGuardCode => ErrorCode::WrongGuardCode,
        UpdateAuthSessionError::TooManyAttempts => ErrorCode::SteamRateLimited,
        UpdateAuthSessionError::SessionExpired | UpdateAuthSessionError::SessionNotStarted => {
            ErrorCode::SessionExpired
        }
        UpdateAuthSessionError::TransportError(error) => transport_code(&error),
        UpdateAuthSessionError::NetworkFailure(_) => ErrorCode::NetworkError,
        _ => ErrorCode::SteamError,
    })
}

pub(super) fn confirmation_error(error: ConfirmerError) -> AuthError {
    AuthError::Code(match error {
        ConfirmerError::InvalidTokens => ErrorCode::SignInRequired,
        ConfirmerError::NetworkFailure(_) => ErrorCode::NetworkError,
        _ => ErrorCode::SteamError,
    })
}
