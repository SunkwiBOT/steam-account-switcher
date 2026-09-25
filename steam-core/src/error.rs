//! Error type shared by the whole core crate.

use std::fmt;
use std::io;

#[derive(Debug)]
pub enum SteamError {
    /// No supported Steam installation could be detected.
    NotFound(String),
    /// A KeyValues file could not be parsed.
    Vdf { path: String, message: String },
    /// Filesystem failure, with the operation that failed.
    Io { context: String, source: io::Error },
    /// Anything else Steam related (bad state, refusing a dangerous write).
    Config(String),
}

impl SteamError {
    pub fn io(context: impl Into<String>, source: io::Error) -> Self {
        SteamError::Io {
            context: context.into(),
            source,
        }
    }

    pub fn config(message: impl Into<String>) -> Self {
        SteamError::Config(message.into())
    }
}

impl fmt::Display for SteamError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SteamError::NotFound(message) => write!(formatter, "{message}"),
            SteamError::Vdf { path, message } => {
                write!(formatter, "Unable to parse {path}: {message}")
            }
            SteamError::Io { context, source } => write!(formatter, "{context}: {source}"),
            SteamError::Config(message) => write!(formatter, "{message}"),
        }
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
