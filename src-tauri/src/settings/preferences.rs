use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UpdateInterval {
    Never,
    Startup,
    FifteenMinutes,
    ThirtyMinutes,
    OneHour,
    #[default]
    SixHours,
    OneDay,
    OneWeek,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SearchScope {
    DisplayName,
    Identifier,
    #[default]
    Both,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VaultLockInterval {
    // ! The vault is only locked at startup, so it stays unlocked while the app runs.
    #[default]
    Startup,
    FiveMinutes,
    TenMinutes,
    FifteenMinutes,
    TwentyMinutes,
    ThirtyMinutes,
    OneHour,
    TwoHours,
    FourHours,
}

impl VaultLockInterval {
    // ! Seconds unlocked before the vault locks again; `None` never locks while running.
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
