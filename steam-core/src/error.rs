use crate::error_code::ErrorCode;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum SteamError {
    NotFound(String),
    Vdf { path: String, message: String },
    Io { context: String, source: io::Error },
    Config(ErrorCode),
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
}

impl fmt::Display for SteamError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match self {
            SteamError::NotFound(_) => ErrorCode::NoSteamInstallation,
            SteamError::Vdf { .. } => ErrorCode::InvalidSteamFile,
            SteamError::Io { source, .. } if source.kind() == io::ErrorKind::PermissionDenied => {
                ErrorCode::FilePermission
            }
            SteamError::Io { .. } => ErrorCode::IoError,
            SteamError::Config(code) => *code,
        };
        formatter.write_str(code.as_str())
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
