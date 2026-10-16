use serde::Serialize;

#[derive(Serialize)]
pub struct InstallationDto {
    pub index: usize,
    pub kind: String,
    pub root: String,
}

#[derive(Serialize)]
pub struct InstallationSettingDto {
    pub root: String,
    pub kind: String,
    pub custom: bool,
    pub hidden: bool,
}

#[derive(Serialize)]
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
    pub last_update_check_at: Option<u64>,
    pub entries: Vec<InstallationSettingDto>,
    pub database_path: String,
}

#[derive(Serialize)]
pub struct TrackingHistoryDto {
    pub installation: Option<TrackingInstallationDto>,
    pub account_count: usize,
}

#[derive(Serialize)]
pub struct TrackingInstallationDto {
    pub kind: String,
    pub root: String,
}

#[derive(Serialize)]
pub struct AccountDto {
    pub steam_id: String,
    pub account_name: String,
    pub persona_name: String,
    pub display_name: String,
    pub initial: String,
    pub avatar: Option<String>,
    pub rank: String,
    pub rank_template: String,
    pub rank_kind: String,
    pub rank_family: String,
    pub playtime_seconds: f64,
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

#[derive(Serialize)]
pub struct VaultStatusDto {
    #[serde(flatten)]
    pub status: steam_core::authenticator::VaultStatus,
    pub lock_in_seconds: Option<u64>,
}

#[derive(Serialize)]
pub struct FolderDto {
    pub id: String,
    pub name: String,
    pub account_count: usize,
    pub system: bool,
    pub color: Option<String>,
    pub icon: Option<String>,
}

#[derive(Serialize)]
pub struct ActiveGameDto {
    pub app_id: String,
    pub name: String,
    pub account_id: String,
    pub elapsed_seconds: f64,
}

#[derive(Serialize)]
pub struct SnapshotDto {
    pub revision: u64,
    pub installations: Vec<InstallationDto>,
    pub selected: usize,
    pub accounts: Vec<AccountDto>,
    pub auto_login_user: String,
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

#[derive(Serialize)]
pub struct AccountPlaytimeCellDto {
    pub steam_id: String,
    pub seconds: f64,
    pub session_count: usize,
}
#[derive(Serialize)]
pub struct PlaytimeSnapshotDto {
    pub machine_seconds: f64,
    pub tracker_ready: bool,
    pub accounts: Vec<AccountPlaytimeCellDto>,
    pub active_games: Vec<ActiveGameDto>,
}

#[derive(Serialize)]
pub struct GameDto {
    pub app_id: String,
    pub name: String,
    pub sessions: u64,
    pub seconds: f64,
    pub status: String,
}

#[derive(Serialize)]
pub struct MachineGameDto {
    pub app_id: String,
    pub name: String,
    pub accounts: usize,
    pub sessions: u64,
    pub seconds: f64,
    pub status: String,
}

#[derive(Serialize)]
pub struct AccountPlaytimeDto {
    pub account_id: String,
    pub display_name: String,
    pub account_name: String,
    pub initial: String,
    pub avatar: Option<String>,
    pub games: usize,
    pub seconds: f64,
    pub status: String,
}

#[derive(Serialize)]
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
    pub coverage: String,
    pub end_reason: String,
    pub exit_code: Option<i32>,
    pub status: String,
    pub seconds: f64,
}

#[derive(Serialize)]
pub struct DailyPointDto {
    pub day: String,
    pub seconds: f64,
    pub sessions: u64,
}

#[derive(Serialize)]
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

#[derive(Serialize)]
pub struct ReportDto {
    pub account_id: Option<String>,
    pub total_seconds: f64,
    pub game_count: usize,
    pub account_count: usize,
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

#[derive(Serialize)]
pub struct RankTemplateDto {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub max_rating: Option<u32>,
    pub ranks: Vec<RankEntryDto>,
}

#[derive(Serialize)]
pub struct RankEntryDto {
    pub value: String,
    pub family: String,
}
