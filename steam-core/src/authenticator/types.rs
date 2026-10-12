use serde::{Deserialize, Serialize};
use steamguard::protobufs::steammessages_auth_steamclient::EAuthSessionGuardType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultStatus {
    pub path: String,
    pub encrypted: bool,
    pub locked: bool,
    pub indexed: bool,
    pub count: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredAuthenticator {
    pub steam_id: String,
    pub account_name: String,
    pub serial_number: String,
    pub device_id: String,
    pub has_tokens: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeInfo {
    pub valid_until: u64,
    pub code: String,
    pub seconds_remaining: u64,
    pub period: u64,
}

/// ! Wire names match the interface's `ConfirmationKind` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationKind {
    Trade,
    Market,
    Phone,
    Recovery,
    ApiKey,
    Feature,
    Family,
    Test,
    Other,
}

/// ! Wire names match the interface's `LinkStatus` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkStatus {
    /// Secrets already saved for this account: the link resumes at the confirmation step.
    Resume,
    Linked,
    AuthenticatorPresent,
    NeedsPhone,
    NeedsEmail,
}

/// ! Wire names match the interface's `LinkConfirmKind` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkConfirmKind {
    Sms,
    Email,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationItem {
    pub id: String,
    pub nonce: String,
    pub kind: ConfirmationKind,
    pub headline: String,
    pub summary: String,
    pub creator_id: String,
    pub creation_time: u64,
    pub icon: Option<String>,
    pub multi: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardMethod {
    pub kind: GuardKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuardKind {
    DeviceCode,
    EmailCode,
    DeviceConfirmation,
    EmailConfirmation,
    None,
    Unknown,
}

impl GuardKind {
    pub fn needs_code(self) -> bool {
        matches!(self, GuardKind::DeviceCode | GuardKind::EmailCode)
    }

    pub(super) fn from_proto(value: EAuthSessionGuardType) -> Self {
        match value {
            EAuthSessionGuardType::k_EAuthSessionGuardType_DeviceCode => GuardKind::DeviceCode,
            EAuthSessionGuardType::k_EAuthSessionGuardType_EmailCode => GuardKind::EmailCode,
            EAuthSessionGuardType::k_EAuthSessionGuardType_DeviceConfirmation => {
                GuardKind::DeviceConfirmation
            }
            EAuthSessionGuardType::k_EAuthSessionGuardType_EmailConfirmation => {
                GuardKind::EmailConfirmation
            }
            EAuthSessionGuardType::k_EAuthSessionGuardType_None => GuardKind::None,
            _ => GuardKind::Unknown,
        }
    }

    pub(super) fn to_proto(self) -> EAuthSessionGuardType {
        match self {
            GuardKind::DeviceCode => EAuthSessionGuardType::k_EAuthSessionGuardType_DeviceCode,
            GuardKind::EmailCode => EAuthSessionGuardType::k_EAuthSessionGuardType_EmailCode,
            GuardKind::DeviceConfirmation => {
                EAuthSessionGuardType::k_EAuthSessionGuardType_DeviceConfirmation
            }
            GuardKind::EmailConfirmation => {
                EAuthSessionGuardType::k_EAuthSessionGuardType_EmailConfirmation
            }
            GuardKind::None => EAuthSessionGuardType::k_EAuthSessionGuardType_None,
            GuardKind::Unknown => EAuthSessionGuardType::k_EAuthSessionGuardType_Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginBegin {
    pub session_id: String,
    pub methods: Vec<GuardMethod>,
    pub qr_svg: Option<String>,
    pub challenge_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResult {
    pub session_id: String,
    pub steam_id: String,
    pub account_name: String,
    pub stored: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkStart {
    pub session_id: String,
    pub status: LinkStatus,
    pub steam_id: String,
    pub account_name: String,
    pub revocation_code: Option<String>,
    pub phone_hint: Option<String>,
    pub confirm_kind: Option<LinkConfirmKind>,
}
