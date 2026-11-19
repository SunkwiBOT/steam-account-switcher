use std::path::{Path, PathBuf};

use steam_account_switcher_lib::commands::dto;
use steam_account_switcher_lib::locale::Language;
use steam_account_switcher_lib::logging::LogSnapshot;
use steam_account_switcher_lib::settings::{SearchScope, UpdateInterval, VaultLockInterval};
use ts_rs::{Config, TS};

fn config(out_dir: &Path) -> Config {
    Config::new().with_out_dir(out_dir)
}

fn export<T: TS + 'static>(cfg: &Config) {
    T::export_all(cfg).expect("bindings written");
}

fn main() {
    let dir = PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("usage: export-bindings <output directory>"),
    );

    if dir.exists() {
        for entry in std::fs::read_dir(&dir).expect("readable output directory") {
            let path = entry.expect("directory entry").path();
            if path.extension().is_some_and(|extension| extension == "ts") {
                std::fs::remove_file(path).expect("stale binding removed");
            }
        }
    }

    let cfg = config(&dir);

    export::<Language>(&cfg);
    export::<UpdateInterval>(&cfg);
    export::<SearchScope>(&cfg);
    export::<VaultLockInterval>(&cfg);

    export::<dto::InstallationDto>(&cfg);
    export::<dto::InstallationSettingDto>(&cfg);
    export::<dto::SettingsDto>(&cfg);
    export::<dto::TrackingHistoryDto>(&cfg);
    export::<dto::TrackingInstallationDto>(&cfg);
    export::<dto::AccountDto>(&cfg);
    export::<dto::VaultStatusDto>(&cfg);
    export::<dto::FolderDto>(&cfg);
    export::<dto::ActiveGameDto>(&cfg);
    export::<dto::SnapshotDto>(&cfg);
    export::<dto::SwitchResultDto>(&cfg);
    export::<steam_core::accounts::SwitchOutcomeKind>(&cfg);
    export::<dto::AccountPlaytimeCellDto>(&cfg);
    export::<dto::PlaytimeSnapshotDto>(&cfg);
    export::<dto::GameDto>(&cfg);
    export::<dto::MachineGameDto>(&cfg);
    export::<dto::AccountPlaytimeDto>(&cfg);
    export::<dto::SessionDto>(&cfg);
    export::<dto::DailyPointDto>(&cfg);
    export::<dto::StatisticsDto>(&cfg);
    export::<dto::ReportDto>(&cfg);
    export::<dto::RankTemplateDto>(&cfg);
    export::<dto::RankEntryDto>(&cfg);
    export::<dto::AppInfoDto>(&cfg);
    export::<dto::UpdateInfoDto>(&cfg);
    export::<dto::UpdateProgressDto>(&cfg);
    export::<dto::BackupImportDto>(&cfg);
    export::<steam_core::model::InstallKind>(&cfg);
    export::<steam_core::ranks::RankKind>(&cfg);
    export::<LogSnapshot>(&cfg);

    export::<steam_core::error_code::ErrorCode>(&cfg);
    export::<steam_core::authenticator::VaultStatus>(&cfg);
    export::<steam_core::authenticator::StoredAuthenticator>(&cfg);
    export::<steam_core::authenticator::CodeInfo>(&cfg);
    export::<steam_core::authenticator::ConfirmationKind>(&cfg);
    export::<steam_core::authenticator::LinkStatus>(&cfg);
    export::<steam_core::authenticator::LinkConfirmKind>(&cfg);
    export::<steam_core::authenticator::ConfirmationItem>(&cfg);
    export::<steam_core::authenticator::GuardMethod>(&cfg);
    export::<steam_core::authenticator::GuardKind>(&cfg);
    export::<steam_core::authenticator::LoginBegin>(&cfg);
    export::<steam_core::authenticator::LoginResult>(&cfg);
    export::<steam_core::authenticator::LinkStart>(&cfg);
}
