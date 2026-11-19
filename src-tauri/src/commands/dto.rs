use serde::Serialize;
use ts_rs::TS;

#[derive(Serialize, TS)]
pub struct InstallationDto {
    pub index: usize,
    pub kind: steam_core::model::InstallKind,
    pub root: String,
}

#[derive(Serialize, TS)]
pub struct InstallationSettingDto {
    pub root: String,
    pub kind: steam_core::model::InstallKind,
    pub custom: bool,
    pub hidden: bool,
}

#[derive(Serialize, TS)]
pub struct SettingsDto {
    pub language: crate::locale::Language,
    pub resolved_language: String,
    pub show_installation_selector: bool,
    pub tracking_enabled: bool,
    pub tracking_history: Vec<TrackingHistoryDto>,
    pub close_to_tray: bool,
    pub update_interval: crate::settings::UpdateInterval,
    pub search_scope: crate::settings::SearchScope,
    pub vault_lock_interval: crate::settings::VaultLockInterval,
    #[ts(type = "number | null")]
    pub last_update_check_at: Option<u64>,
    pub entries: Vec<InstallationSettingDto>,
    pub database_path: String,
    pub vault_offer_dismissed: bool,
}

#[derive(Serialize, TS)]
pub struct TrackingHistoryDto {
    pub installation: Option<TrackingInstallationDto>,
    pub account_count: usize,
}

#[derive(Serialize, TS)]
pub struct TrackingInstallationDto {
    pub kind: steam_core::model::InstallKind,
    pub root: String,
}

#[derive(Serialize, TS)]
pub struct AccountDto {
    pub steam_id: String,
    pub account_name: String,
    pub persona_name: String,
    pub display_name: String,
    pub initial: String,
    pub avatar: Option<String>,
    pub rank: String,
    pub rank_template: String,
    pub rank_kind: steam_core::ranks::RankKind,
    pub rank_family: String,
    pub playtime_seconds: f64,
    #[ts(type = "number")]
    pub last_login_seconds: i64,
    pub is_current: bool,
    pub is_local: bool,
    pub session_count: usize,
    pub color: Option<String>,
    pub note: Option<String>,
    pub folder_id: Option<String>,
    pub has_authenticator: bool,
    pub has_authenticator_session: bool,
}

#[derive(Serialize, TS)]
pub struct VaultStatusDto {
    #[serde(flatten)]
    #[ts(flatten)]
    pub status: steam_core::authenticator::VaultStatus,
    #[ts(type = "number | null")]
    pub lock_in_seconds: Option<u64>,
}

#[derive(Serialize, TS)]
pub struct FolderDto {
    pub id: String,
    pub name: String,
    pub account_count: usize,
    pub system: bool,
    pub color: Option<String>,
    pub icon: Option<String>,
}

#[derive(Serialize, TS)]
pub struct ActiveGameDto {
    pub app_id: String,
    pub name: String,
    pub account_id: String,
    pub elapsed_seconds: f64,
}

#[derive(Serialize, TS)]
pub struct SnapshotDto {
    #[ts(type = "number")]
    pub revision: u64,
    pub installations: Vec<InstallationDto>,
    pub selected: usize,
    pub accounts: Vec<AccountDto>,
    pub accounts_error: Option<String>,
    pub masked: bool,
    pub steam_ids_visible: bool,
    pub machine_seconds: f64,
    pub active_games: Vec<ActiveGameDto>,
    pub tracker_ready: bool,
    pub steam_running: bool,
    pub rank_templates: Vec<RankTemplateDto>,
    pub settings: SettingsDto,
    pub folders: Vec<FolderDto>,
    pub vault: VaultStatusDto,
}

#[derive(Serialize, TS)]
pub struct SwitchResultDto {
    pub snapshot: SnapshotDto,
    pub outcome: steam_core::accounts::SwitchOutcomeKind,
}

#[derive(Serialize, TS)]
pub struct AccountPlaytimeCellDto {
    pub steam_id: String,
    pub seconds: f64,
    pub session_count: usize,
}
#[derive(Serialize, TS)]
pub struct PlaytimeSnapshotDto {
    pub machine_seconds: f64,
    pub tracker_ready: bool,
    pub accounts: Vec<AccountPlaytimeCellDto>,
    pub active_games: Vec<ActiveGameDto>,
}

#[derive(Serialize, TS)]
pub struct GameDto {
    pub app_id: String,
    pub name: String,
    #[ts(type = "number")]
    pub sessions: u64,
    pub seconds: f64,
    pub status: steam_core::playtime::PlaytimeStatus,
}

#[derive(Serialize, TS)]
pub struct MachineGameDto {
    pub app_id: String,
    pub name: String,
    pub accounts: usize,
    #[ts(type = "number")]
    pub sessions: u64,
    pub seconds: f64,
    pub status: steam_core::playtime::PlaytimeStatus,
}

#[derive(Serialize, TS)]
pub struct AccountPlaytimeDto {
    pub account_id: String,
    pub display_name: String,
    pub account_name: String,
    pub initial: String,
    pub avatar: Option<String>,
    pub games: usize,
    pub seconds: f64,
    pub status: steam_core::playtime::PlaytimeStatus,
}

#[derive(Serialize, TS)]
pub struct SessionDto {
    pub session_id: String,
    pub account_id: String,
    pub account_name: String,
    pub display_name: String,
    pub initial: String,
    pub avatar: Option<String>,
    pub game_name: String,
    pub app_id: String,
    pub started_at: f64,
    pub ended_at: Option<f64>,
    pub launch_timestamp: f64,
    pub process_create_time: Option<f64>,
    pub root_pid: Option<u32>,
    pub coverage: steam_core::playtime::TrackingMode,
    pub end_reason: Option<steam_core::playtime::EndReason>,
    pub exit_code: Option<i32>,
    pub status: steam_core::playtime::PlaytimeStatus,
    pub seconds: f64,
}

#[derive(Serialize, TS)]
pub struct DailyPointDto {
    pub day: String,
    pub seconds: f64,
    #[ts(type = "number")]
    pub sessions: u64,
}

#[derive(Serialize, TS)]
pub struct StatisticsDto {
    pub history_started_at: f64,
    pub session_count: usize,
    pub average_session_seconds: f64,
    pub longest_session_seconds: f64,
}

impl From<steam_core::playtime::PlaytimeStatistics> for StatisticsDto {
    fn from(value: steam_core::playtime::PlaytimeStatistics) -> Self {
        Self {
            history_started_at: value.history_started_at,
            session_count: value.session_count,
            average_session_seconds: value.average_session_seconds,
            longest_session_seconds: value.longest_session_seconds,
        }
    }
}

#[derive(Serialize, TS)]
pub struct ReportDto {
    pub account_id: Option<String>,
    pub total_seconds: f64,
    pub game_count: usize,
    pub account_count: usize,
    #[ts(type = "number")]
    pub session_count: u64,
    pub rank: String,
    pub rank_template: String,
    pub statistics: StatisticsDto,
    pub games: Vec<GameDto>,
    pub machine_games: Vec<MachineGameDto>,
    pub accounts: Vec<AccountPlaytimeDto>,
    pub daily7: Vec<DailyPointDto>,
    pub daily30: Vec<DailyPointDto>,
    pub daily90: Vec<DailyPointDto>,
    pub sessions: Vec<SessionDto>,
    pub machine: bool,
}

#[derive(Serialize, TS)]
pub struct RankTemplateDto {
    pub id: String,
    pub label: String,
    pub kind: steam_core::ranks::RankKind,
    pub max_rating: Option<u32>,
    pub ranks: Vec<RankEntryDto>,
}

#[derive(Serialize, TS)]
pub struct RankEntryDto {
    pub value: String,
    pub family: String,
}

#[derive(Serialize, TS)]
pub struct AppInfoDto {
    pub name: String,
    pub version: String,
    pub repository: String,
    pub backup_extension: String,
}

#[derive(Serialize, TS)]
pub struct UpdateInfoDto {
    pub version: String,
    pub current_version: String,
    pub notes: Option<String>,
}

#[derive(Clone, Serialize, TS)]
pub struct UpdateProgressDto {
    #[ts(type = "number")]
    pub downloaded: u64,
    #[ts(type = "number | null")]
    pub total: Option<u64>,
}

#[derive(Serialize, TS)]
pub struct BackupImportDto {
    pub imported: usize,
    pub skipped: usize,
    pub snapshot: SnapshotDto,
}
