use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum UpdateInterval {
    #[serde(rename = "never")]
    Never,
    #[serde(rename = "startup")]
    Startup,
    #[serde(rename = "15m")]
    FifteenMinutes,
    #[serde(rename = "30m")]
    ThirtyMinutes,
    #[serde(rename = "1h")]
    OneHour,
    #[default]
    #[serde(rename = "6h")]
    SixHours,
    #[serde(rename = "1d")]
    OneDay,
    #[serde(rename = "1w")]
    OneWeek,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum SearchScope {
    /// Matches the displayed name only, never the account identifier.
    #[serde(rename = "display_name")]
    DisplayName,
    /// Matches the Steam login name and the Steam ID only.
    #[serde(rename = "identifier")]
    Identifier,
    #[default]
    #[serde(rename = "both")]
    Both,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum VaultLockInterval {
    /// The vault is only locked at startup, so it stays unlocked while the app runs.
    #[default]
    #[serde(rename = "startup")]
    Startup,
    #[serde(rename = "5m")]
    FiveMinutes,
    #[serde(rename = "10m")]
    TenMinutes,
    #[serde(rename = "15m")]
    FifteenMinutes,
    #[serde(rename = "20m")]
    TwentyMinutes,
    #[serde(rename = "30m")]
    ThirtyMinutes,
    #[serde(rename = "1h")]
    OneHour,
    #[serde(rename = "2h")]
    TwoHours,
    #[serde(rename = "4h")]
    FourHours,
}

impl VaultLockInterval {
    /// Seconds unlocked before the vault locks again; `None` never locks while running.
    pub const fn seconds(self) -> Option<u64> {
        match self {
            Self::Startup => None,
            Self::FiveMinutes => Some(5 * 60),
            Self::TenMinutes => Some(10 * 60),
            Self::FifteenMinutes => Some(15 * 60),
            Self::TwentyMinutes => Some(20 * 60),
            Self::ThirtyMinutes => Some(30 * 60),
            Self::OneHour => Some(60 * 60),
            Self::TwoHours => Some(2 * 60 * 60),
            Self::FourHours => Some(4 * 60 * 60),
        }
    }

    pub fn locks_after(self, elapsed_seconds: u64) -> bool {
        matches!(self.seconds(), Some(seconds) if elapsed_seconds >= seconds)
    }
}
