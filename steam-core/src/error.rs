use crate::error_code::ErrorCode;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum SteamError {
    NotFound(String),
    Vdf {
        path: String,
        message: String,
    },
    Io {
        context: String,
        source: io::Error,
    },
    Config(ErrorCode),
    SchemaVersion {
        context: String,
        found: i64,
        expected: i64,
    },
}

impl SteamError {
    pub fn io(context: impl Into<String>, source: io::Error) -> Self {
        SteamError::Io {
            context: context.into(),
            source,
        }
    }

    pub fn config(message: ErrorCode) -> Self {
        SteamError::Config(message)
    }

    pub fn schema_version(context: impl Into<String>, found: i64, expected: i64) -> Self {
        SteamError::SchemaVersion {
            context: context.into(),
            found,
            expected,
        }
    }

    pub fn code(&self) -> ErrorCode {
        match self {
            SteamError::NotFound(_) => ErrorCode::NoSteamInstallation,
            SteamError::Vdf { .. } => ErrorCode::InvalidSteamFile,
            SteamError::Io { source, .. } if source.kind() == io::ErrorKind::PermissionDenied => {
                ErrorCode::FilePermission
            }
            SteamError::Io { .. } => ErrorCode::IoError,
            SteamError::Config(code) => *code,
            SteamError::SchemaVersion { .. } => ErrorCode::DatabaseVersionMismatch,
        }
    }

    // ! Bounded, safe facts only: an `io::ErrorKind` name or schema version numbers. Never a path, URL, token or raw message, any of which could leak user data into the log.
    pub fn detail(&self) -> Option<String> {
        match self {
            SteamError::Io { source, .. } => Some(format!("kind={:?}", source.kind())),
            SteamError::SchemaVersion {
                found, expected, ..
            } => Some(format!("found={found} expected={expected}")),
            SteamError::NotFound(_) | SteamError::Vdf { .. } | SteamError::Config(_) => None,
        }
    }
}

impl fmt::Display for SteamError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code().as_str())
    }
}

impl std::error::Error for SteamError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SteamError::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, SteamError>;
