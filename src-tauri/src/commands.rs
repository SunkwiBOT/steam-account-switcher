//! Commands exposed to the web UI.

use std::sync::{Arc, Mutex};

use serde::Serialize;
use steam_core::metadata::{RankChoice, RankStore};
use steam_core::model::SteamAccount;
use steam_core::playtime::{
    AccountPlaytime, DailyPlaytime, GamePlaytime, MachineGamePlaytime, PlaytimeStatistics,
    PlaytimeTracker, SessionPlaytime,
};
use steam_core::ranks;
use steam_core::util::{absolute_time, format_playtime, relative_time};
use tauri::{Emitter, State};
use tauri_plugin_updater::Update;

use crate::base64;
use crate::state::AppState;

/// Where the "open the project" links point. Kept here so there is exactly one
/// place to change when the repository moves.
pub const REPOSITORY_URL: &str = "https://github.com/SunkwiBOT/steam-account-switcher";

// ----------------------------------------------------------------- DTO types

#[derive(Serialize)]
pub struct InstallationDto {
    pub index: usize,
    pub label: String,
    pub kind: String,
    pub root: String,
}

#[derive(Serialize)]
pub struct InstallationSettingDto {
    pub root: String,
    pub label: String,
    pub kind: String,
    pub custom: bool,
    pub hidden: bool,
}

#[derive(Serialize)]
pub struct SettingsDto {
    pub show_installation_selector: bool,
    pub show_machine_playtime: bool,
    pub entries: Vec<InstallationSettingDto>,
    pub settings_path: String,
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
    /// Ladder this rank belongs to ("marvel-rivals", "cs2-premier", ...).
    pub rank_template: String,
    /// "tiered" or "rating": the interface picks a list or a number field.
    pub rank_kind: String,
    /// Icon file stem inside the template folder, empty when there is none.
    pub rank_family: String,
    pub playtime_seconds: f64,
    pub playtime_text: String,
    pub playtime_tooltip: String,
    pub last_login_text: String,
    pub last_login_absolute: String,
    pub last_login_seconds: i64,
    pub is_current: bool,
    pub session_count: usize,
    pub color: Option<String>,
    pub folder_id: Option<String>,
}

#[derive(Serialize)]
pub struct FolderDto {
    pub id: String,
    pub name: String,
    pub account_count: usize,
}

#[derive(Serialize)]
pub struct ActiveGameDto {
    pub app_id: String,
    pub name: String,
    pub account_id: String,
    pub elapsed_text: String,
}

#[derive(Serialize)]
pub struct SnapshotDto {
    pub installations: Vec<InstallationDto>,
    pub selected: usize,
    pub accounts: Vec<AccountDto>,
    pub auto_login_user: String,
    pub masked: bool,
    pub steam_ids_visible: bool,
    pub machine_seconds: f64,
    pub machine_text: String,
    pub active_games: Vec<ActiveGameDto>,
    pub tracker_ready: bool,
    pub status: String,
    /// Whether the Steam client is currently running, so the switch dialog can
    /// say "start Steam" instead of "restart Steam".
    pub steam_running: bool,
    pub rank_templates: Vec<RankTemplateDto>,
    pub settings: SettingsDto,
    pub folders: Vec<FolderDto>,
}

#[derive(Serialize)]
pub struct AccountPlaytimeCellDto {
    pub steam_id: String,
    pub seconds: f64,
    pub text: String,
    pub session_count: usize,
}

/// Light payload emitted once per second while the tracker runs.
#[derive(Serialize)]
pub struct PlaytimeSnapshotDto {
    pub machine_seconds: f64,
    pub machine_text: String,
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
    pub seconds_text: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct MachineGameDto {
    pub app_id: String,
    pub name: String,
    pub accounts: usize,
    pub sessions: u64,
    pub seconds: f64,
    pub seconds_text: String,
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
    pub seconds_text: String,
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
    pub started_text: String,
    pub ended_text: String,
    pub duration_text: String,
    pub coverage: String,
    pub end_reason: String,
    pub status: String,
    pub seconds: f64,
    pub tooltip: String,
}

#[derive(Serialize)]
pub struct DailyPointDto {
    pub day: String,
    pub seconds: f64,
    pub sessions: u64,
}

#[derive(Serialize)]
pub struct ReportDto {
    pub account_id: Option<String>,
    pub title: String,
    pub subtitle: String,
    pub summary_text: String,
    pub statistics_text: String,
    pub notice_text: String,
    pub games: Vec<GameDto>,
    pub machine_games: Vec<MachineGameDto>,
    pub accounts: Vec<AccountPlaytimeDto>,
    pub daily7: Vec<DailyPointDto>,
    pub daily30: Vec<DailyPointDto>,
    pub daily90: Vec<DailyPointDto>,
    pub sessions: Vec<SessionDto>,
    pub machine: bool,
}

// -------------------------------------------------------------- helpers/text

fn plural(count: usize, singular: &str) -> String {
    if count == 1 {
        singular.to_string()
    } else {
        format!("{singular}s")
    }
}

fn end_reason_label(reason: &str) -> String {
    match reason {
        "steam_running_list" => "Steam stopped game".to_string(),
        "root_process_exit_log" => "Root process exited".to_string(),
        "process_exit" => "Process exited".to_string(),
        "process_missing_after_restart" => "Process missing after restart".to_string(),
        "replaced_by_new_launch" => "New launch detected".to_string(),
        "" | "unknown" => "Unknown".to_string(),
        other => {
            let mut label = other.replace('_', " ");
            if let Some(first) = label.get_mut(0..1) {
                first.make_ascii_uppercase();
            }
            label
        }
    }
}

fn statistics_text(statistics: &PlaytimeStatistics) -> String {
    format!(
        "Since: {}  |  Detailed sessions: {}  |  Average: {}  |  Longest: {}",
        absolute_time(Some(statistics.history_started_at as i64)),
        statistics.session_count,
        format_playtime(statistics.average_session_seconds),
        format_playtime(statistics.longest_session_seconds)
    )
}

fn notice_text(statistics: &PlaytimeStatistics) -> String {
    let started = absolute_time(Some(statistics.history_started_at as i64));
    if statistics.legacy_seconds > 0.0 {
        format!(
            "Detailed history starts {started}. An earlier {} aggregate remains in the totals without invented dates.",
            format_playtime(statistics.legacy_seconds)
        )
    } else {
        format!("Detailed history starts {started}.")
    }
}

fn session_tooltip(session: &SessionPlaytime, masked: bool) -> String {
    let account = if masked {
        "hidden".to_string()
    } else {
        session.account_name.clone()
    };
    let process_created = session
        .process_create_time
        .map(|value| absolute_time(Some(value as i64)))
        .unwrap_or_else(|| "Unavailable".to_string());

    [
        format!("Session ID: {}", session.session_id),
        format!("Account: {account}"),
        format!(
            "Steam launch detected: {}",
            absolute_time(Some(session.launch_timestamp as i64))
        ),
        format!(
            "Playtime counted from: {}",
            absolute_time(Some(session.started_at as i64))
        ),
        format!(
            "Root PID: {}",
            session
                .root_pid
                .map(|pid| pid.to_string())
                .unwrap_or_else(|| "Unavailable".to_string())
        ),
        format!("Process created: {process_created}"),
        format!(
            "Coverage: {}",
            if session.tracking_mode == "full" {
                "Full"
            } else {
                "Partial"
            }
        ),
        format!("End reason: {}", end_reason_label(&session.end_reason)),
    ]
    .join("\n")
}

/// A rank ladder, sent once per snapshot so the interface can build its
/// template menu and rank lists from the same source as the validation.
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

fn rank_templates() -> Vec<RankTemplateDto> {
    ranks::templates()
        .iter()
        .map(|template| RankTemplateDto {
            id: template.id.to_string(),
            label: template.label.to_string(),
            kind: template.kind.as_str().to_string(),
            max_rating: template.max_rating,
            ranks: template
                .ranks
                .iter()
                .map(|rank| RankEntryDto {
                    value: rank.value.clone(),
                    family: rank.family.clone(),
                })
                .collect(),
        })
        .collect()
}

fn daily_points(entries: Vec<DailyPlaytime>) -> Vec<DailyPointDto> {
    entries
        .into_iter()
        .map(|entry| DailyPointDto {
            day: format!(
                "{:04}-{:02}-{:02}",
                entry.day.year(),
                entry.day.month() as u8,
                entry.day.day()
            ),
            seconds: entry.seconds,
            sessions: entry.sessions,
        })
        .collect()
}

struct Catalog {
    accounts: Vec<SteamAccount>,
    masked: bool,
}

impl Catalog {
    fn account(&self, steam_id: &str) -> Option<&SteamAccount> {
        self.accounts
            .iter()
            .find(|account| account.steam_id == steam_id)
    }

    fn avatar(&self, steam_id: &str) -> Option<String> {
        self.account(steam_id)
            .and_then(|account| account.avatar_path.as_deref())
            .and_then(base64::data_url)
    }

    fn display_name(&self, steam_id: &str, fallback: &str, persona: &str) -> String {
        if let Some(account) = self.account(steam_id) {
            return account.display_name().to_string();
        }
        if !persona.is_empty() {
            persona.to_string()
        } else if !fallback.is_empty() {
            fallback.to_string()
        } else {
            steam_id.to_string()
        }
    }

    fn initial(&self, steam_id: &str, fallback: &str, persona: &str) -> String {
        if let Some(account) = self.account(steam_id) {
            return account.avatar_initial();
        }
        let label = if !persona.is_empty() {
            persona
        } else {
            fallback
        };
        label
            .chars()
            .next()
            .map(|character| character.to_uppercase().to_string())
            .unwrap_or_else(|| "?".to_string())
    }

    fn account_name(&self, account_id: &str) -> String {
        self.account(account_id)
            .map(|account| account.account_name.clone())
            .unwrap_or_default()
    }
}

/// Persona name of an account, falling back to its login name.
fn display_name_for(state: &State<'_, AppState>, steam_id: &str, fallback: &str) -> String {
    let inner = state.lock();
    let account = inner
        .accounts
        .iter()
        .find(|account| account.steam_id == steam_id)
        .or_else(|| {
            inner
                .accounts
                .iter()
                .find(|account| account.account_name.eq_ignore_ascii_case(fallback))
        });

    match account {
        Some(account) => account.display_name().to_string(),
        None => fallback.to_string(),
    }
}

fn report_for_account(
    tracker: &PlaytimeTracker,
    account: &SteamAccount,
    ranks: Option<&RankStore>,
    catalog: &Catalog,
) -> ReportDto {
    let games = tracker.account_games(&account.steam_id);
    let session_count: u64 = games.iter().map(|game| game.sessions).sum();
    let statistics = tracker.history_statistics(Some(&account.steam_id));
    let rank = ranks
        .map(|store| store.choice_for(account).value)
        .unwrap_or_else(|| ranks::UNRANKED.to_string());

    let daily7 = tracker.daily_playtime(7, Some(&account.steam_id));
    let daily30 = tracker.daily_playtime(30, Some(&account.steam_id));
    let daily90 = tracker.daily_playtime(90, Some(&account.steam_id));
    let sessions = tracker.session_history(Some(&account.steam_id), true);
    let game_count = games.len();

    ReportDto {
        account_id: Some(account.steam_id.clone()),
        title: format!("{} - Local Playtime", account.display_name()),
        subtitle: "Games detected for this account on this PC only.".to_string(),
        summary_text: format!(
            "Total: {}  |  {} {}  |  {} {}",
            format_playtime(tracker.account_seconds(&account.steam_id, true)),
            game_count,
            plural(game_count, "game"),
            session_count,
            plural(session_count as usize, "session")
        ),
        statistics_text: statistics_text(&statistics),
        notice_text: format!("{}  |  Rank: {rank}", notice_text(&statistics)),
        games: games.into_iter().map(game_dto).collect(),
        machine_games: Vec::new(),
        accounts: Vec::new(),
        daily7: daily_points(daily7),
        daily30: daily_points(daily30),
        daily90: daily_points(daily90),
        sessions: sessions
            .iter()
            .map(|session| session_dto(session, catalog))
            .collect(),
        machine: false,
    }
}

fn game_dto(game: GamePlaytime) -> GameDto {
    GameDto {
        app_id: game.app_id,
        name: game.name,
        sessions: game.sessions,
        seconds: game.seconds,
        seconds_text: format_playtime(game.seconds),
        status: if game.active {
            "Running".into()
        } else {
            String::new()
        },
    }
}

fn machine_game_dto(game: MachineGamePlaytime) -> MachineGameDto {
    MachineGameDto {
        app_id: game.app_id,
        name: game.name,
        accounts: game.account_count,
        sessions: game.sessions,
        seconds: game.seconds,
        seconds_text: format_playtime(game.seconds),
        status: if game.active {
            "Running".into()
        } else {
            String::new()
        },
    }
}

fn account_playtime_dto(account: AccountPlaytime, catalog: &Catalog) -> AccountPlaytimeDto {
    AccountPlaytimeDto {
        display_name: catalog.display_name(
            &account.account_id,
            &account.account_name,
            &account.persona_name,
        ),
        account_name: if catalog.masked {
            "******".to_string()
        } else {
            catalog.account_name(&account.account_id)
        },
        initial: catalog.initial(
            &account.account_id,
            &account.account_name,
            &account.persona_name,
        ),
        avatar: catalog.avatar(&account.account_id),
        games: account.game_count,
        seconds: account.seconds,
        seconds_text: format_playtime(account.seconds),
        status: if account.active {
            "Running".into()
        } else {
            String::new()
        },
        account_id: account.account_id,
    }
}

fn session_dto(session: &SessionPlaytime, catalog: &Catalog) -> SessionDto {
    SessionDto {
        session_id: session.session_id.clone(),
        account_id: session.account_id.clone(),
        account_name: if catalog.masked {
            "******".to_string()
        } else {
            session.account_name.clone()
        },
        display_name: catalog.display_name(
            &session.account_id,
            &session.account_name,
            &session.persona_name,
        ),
        initial: catalog.initial(
            &session.account_id,
            &session.account_name,
            &session.persona_name,
        ),
        avatar: catalog.avatar(&session.account_id),
        game_name: session.game_name.clone(),
        app_id: session.app_id.clone(),
        started_text: absolute_time(Some(session.started_at as i64)),
        ended_text: session
            .ended_at
            .map(|value| absolute_time(Some(value as i64)))
            .unwrap_or_else(|| "Running".to_string()),
        duration_text: format_playtime(session.duration_seconds),
        coverage: if session.tracking_mode == "full" {
            "Full".into()
        } else {
            "Partial".into()
        },
        end_reason: if session.active {
            "Running".to_string()
        } else if session.exit_code.is_some() {
            format!(
                "{} (code {})",
                end_reason_label(&session.end_reason),
                session.exit_code.unwrap_or_default()
            )
        } else {
            end_reason_label(&session.end_reason)
        },
        status: if session.active {
            "Running".into()
        } else {
            String::new()
        },
        seconds: session.duration_seconds,
        tooltip: session_tooltip(session, catalog.masked),
    }
}

// --------------------------------------------------------------- command set

fn build_snapshot(state: &AppState) -> SnapshotDto {
    let inner = state.lock();
    let tracker = inner.tracker.clone();
    let ranks = inner.ranks.clone();
    let masked = inner.masked;
    let steam_ids_visible = inner.steam_ids_visible;
    let accounts = inner.accounts.clone();
    let auto_login_user = inner.auto_login_user.clone();
    let installations = inner.installations.clone();
    let detected = inner.detected.clone();
    let custom = inner.custom.clone();
    let settings_data = inner.settings.data().clone();
    let settings_path = inner.settings.path().display().to_string();
    let selected = inner.selected;
    let message = inner.message.clone();
    let steam_running = inner
        .manager
        .as_ref()
        .map(|manager| manager.is_steam_running())
        .unwrap_or(false);
    drop(inner);

    let (machine_seconds, active_games, tracker_ready, per_account) = match &tracker {
        Some(tracker) => match tracker.lock() {
            Ok(tracker) => {
                let per_account: Vec<(String, f64, usize)> = accounts
                    .iter()
                    .map(|account| {
                        (
                            account.steam_id.clone(),
                            tracker.account_seconds(&account.steam_id, true),
                            tracker
                                .history_statistics(Some(&account.steam_id))
                                .session_count,
                        )
                    })
                    .collect();
                let active = tracker
                    .active_games()
                    .into_iter()
                    .map(|game| ActiveGameDto {
                        app_id: game.app_id,
                        name: game.name,
                        account_id: game.account_id,
                        elapsed_text: format_playtime(game.elapsed_seconds),
                    })
                    .collect();
                let machine = tracker.global_seconds(true);
                let ready = tracker.log_path().exists();
                (machine, active, ready, per_account)
            }
            Err(_) => (0.0, Vec::new(), false, Vec::new()),
        },
        None => (0.0, Vec::new(), false, Vec::new()),
    };

    let account_dtos: Vec<AccountDto> = accounts
        .iter()
        .map(|account| {
            let choice = ranks
                .as_ref()
                .map(|store| store.choice_for(account))
                .unwrap_or(RankChoice {
                    template: ranks::DEFAULT_TEMPLATE.to_string(),
                    value: ranks::UNRANKED.to_string(),
                });
            let kind = ranks::template(&choice.template)
                .map(|template| template.kind.as_str())
                .unwrap_or(ranks::RankKind::Tiered.as_str());
            let family = ranks::family(&choice.template, &choice.value);
            let playtime = per_account
                .iter()
                .find(|(steam_id, _, _)| steam_id == &account.steam_id)
                .map(|(_, seconds, sessions)| (*seconds, *sessions))
                .unwrap_or((0.0, 0));

            AccountDto {
                steam_id: account.steam_id.clone(),
                account_name: if masked {
                    "******".to_string()
                } else {
                    account.account_name.clone()
                },
                persona_name: account.persona_name.clone(),
                display_name: account.display_name().to_string(),
                initial: account.avatar_initial(),
                avatar: account.avatar_path.as_deref().and_then(base64::data_url),
                rank: choice.value.clone(),
                rank_template: choice.template.clone(),
                rank_kind: kind.to_string(),
                rank_family: family,
                last_login_text: relative_time(account.timestamp),
                last_login_absolute: absolute_time(account.timestamp),
                last_login_seconds: account.timestamp.unwrap_or(0),
                is_current: account.account_name.eq_ignore_ascii_case(&auto_login_user),
                session_count: playtime.1,
                color: settings_data
                    .account_color(&account.steam_id)
                    .map(str::to_string),
                folder_id: settings_data
                    .account_folder(&account.steam_id)
                    .map(str::to_string),
                playtime_seconds: playtime.0,
                playtime_text: format_playtime(playtime.0),
                playtime_tooltip: if playtime.1 == 0 {
                    "No local session detected yet.".to_string()
                } else {
                    format!(
                        "{} in {} session(s). Open the card details for the per-game breakdown.",
                        format_playtime(playtime.0),
                        playtime.1
                    )
                },
            }
        })
        .collect();

    let status = if message.is_empty() {
        let count = accounts.len();
        format!("{count} account{}", if count == 1 { "" } else { "s" })
    } else {
        message
    };

    SnapshotDto {
        folders: settings_data
            .folders
            .iter()
            .map(|folder| FolderDto {
                id: folder.id.clone(),
                name: folder.name.clone(),
                account_count: accounts
                    .iter()
                    .filter(|account| {
                        settings_data.account_folder(&account.steam_id) == Some(folder.id.as_str())
                    })
                    .count(),
            })
            .collect(),
        settings: SettingsDto {
            show_installation_selector: settings_data.show_installation_selector,
            show_machine_playtime: settings_data.show_machine_playtime,
            settings_path,
            entries: detected
                .iter()
                .chain(custom.iter())
                .map(|installation| {
                    let root = crate::state::root_key(installation);
                    InstallationSettingDto {
                        label: installation.label(),
                        kind: installation.kind.as_str().to_string(),
                        custom: settings_data.is_custom(&root),
                        hidden: settings_data.is_hidden(&root),
                        root,
                    }
                })
                .collect(),
        },
        installations: installations
            .iter()
            .enumerate()
            .map(|(index, installation)| InstallationDto {
                index,
                label: installation.label(),
                kind: installation.kind.as_str().to_string(),
                root: installation.root.display().to_string(),
            })
            .collect(),
        selected,
        accounts: account_dtos,
        auto_login_user,
        masked,
        steam_ids_visible,
        machine_seconds,
        machine_text: format_playtime(machine_seconds),
        active_games,
        tracker_ready,
        status,
        steam_running,
        rank_templates: rank_templates(),
    }
}

#[tauri::command]
pub fn snapshot(state: State<'_, AppState>) -> SnapshotDto {
    build_snapshot(&state)
}

/// Lightweight payload for the once-per-second refresh: playtime only, no
/// avatars and no re-reading of `loginusers.vdf`.
#[tauri::command]
pub fn playtime_snapshot(state: State<'_, AppState>) -> PlaytimeSnapshotDto {
    let (accounts, tracker) = {
        let inner = state.lock();
        (inner.accounts.clone(), inner.tracker.clone())
    };

    let Some(tracker) = tracker else {
        return PlaytimeSnapshotDto {
            machine_seconds: 0.0,
            machine_text: format_playtime(0.0),
            tracker_ready: false,
            accounts: Vec::new(),
            active_games: Vec::new(),
        };
    };

    let Ok(tracker) = tracker.lock() else {
        return PlaytimeSnapshotDto {
            machine_seconds: 0.0,
            machine_text: format_playtime(0.0),
            tracker_ready: false,
            accounts: Vec::new(),
            active_games: Vec::new(),
        };
    };

    let cells: Vec<AccountPlaytimeCellDto> = accounts
        .iter()
        .map(|account| {
            let seconds = tracker.account_seconds(&account.steam_id, true);
            let session_count = tracker
                .history_statistics(Some(&account.steam_id))
                .session_count;
            AccountPlaytimeCellDto {
                steam_id: account.steam_id.clone(),
                seconds,
                text: format_playtime(seconds),
                session_count,
            }
        })
        .collect();

    let machine_seconds = tracker.global_seconds(true);
    PlaytimeSnapshotDto {
        machine_seconds,
        machine_text: format_playtime(machine_seconds),
        tracker_ready: tracker.log_path().exists(),
        accounts: cells,
        active_games: tracker
            .active_games()
            .into_iter()
            .map(|game| ActiveGameDto {
                app_id: game.app_id,
                name: game.name,
                account_id: game.account_id,
                elapsed_text: format_playtime(game.elapsed_seconds),
            })
            .collect(),
    }
}

#[tauri::command]
pub fn refresh(state: State<'_, AppState>) -> SnapshotDto {
    state.refresh_accounts();
    build_snapshot(&state)
}

#[tauri::command]
pub fn reload_installations(state: State<'_, AppState>) -> SnapshotDto {
    state.reload_installations();
    build_snapshot(&state)
}

#[tauri::command]
pub fn select_installation(
    state: State<'_, AppState>,
    index: usize,
) -> Result<SnapshotDto, String> {
    state.select_installation(index)?;
    Ok(build_snapshot(&state))
}

#[tauri::command]
pub fn set_masked(state: State<'_, AppState>, masked: bool) -> SnapshotDto {
    state.lock().masked = masked;
    build_snapshot(&state)
}

#[tauri::command]
pub fn set_steam_ids_visible(state: State<'_, AppState>, visible: bool) -> SnapshotDto {
    state.lock().steam_ids_visible = visible;
    build_snapshot(&state)
}

#[tauri::command]
pub fn update_settings(
    state: State<'_, AppState>,
    show_installation_selector: bool,
    show_machine_playtime: bool,
) -> Result<SnapshotDto, String> {
    state.set_ui_settings(show_installation_selector, show_machine_playtime)?;
    Ok(build_snapshot(&state))
}

#[tauri::command]
pub fn set_installation_hidden(
    state: State<'_, AppState>,
    root: String,
    hidden: bool,
) -> Result<SnapshotDto, String> {
    state.set_installation_hidden(&root, hidden)?;
    Ok(build_snapshot(&state))
}

#[tauri::command]
pub fn add_custom_installation(
    state: State<'_, AppState>,
    path: String,
) -> Result<SnapshotDto, String> {
    let root = state.add_custom_installation(&path)?;
    state.lock().message = format!("Added {root}");
    Ok(build_snapshot(&state))
}

#[tauri::command]
pub fn remove_custom_installation(
    state: State<'_, AppState>,
    root: String,
) -> Result<SnapshotDto, String> {
    state.remove_custom_installation(&root)?;
    Ok(build_snapshot(&state))
}

#[tauri::command]
pub fn create_folder(state: State<'_, AppState>, name: String) -> Result<SnapshotDto, String> {
    state.create_folder(&name)?;
    Ok(build_snapshot(&state))
}

#[tauri::command]
pub fn rename_folder(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> Result<SnapshotDto, String> {
    state.rename_folder(&id, &name)?;
    Ok(build_snapshot(&state))
}

#[tauri::command]
pub fn delete_folder(state: State<'_, AppState>, id: String) -> Result<SnapshotDto, String> {
    state.delete_folder(&id)?;
    Ok(build_snapshot(&state))
}

#[tauri::command]
pub fn assign_folder(
    state: State<'_, AppState>,
    steam_id: String,
    folder_id: Option<String>,
) -> Result<SnapshotDto, String> {
    state.set_account_folder(&steam_id, folder_id.as_deref())?;
    Ok(build_snapshot(&state))
}

#[tauri::command]
pub fn set_account_color(
    state: State<'_, AppState>,
    steam_id: String,
    color: Option<String>,
) -> Result<SnapshotDto, String> {
    state.set_account_color(&steam_id, color.as_deref())?;
    Ok(build_snapshot(&state))
}

/// Lets the interface tell the backend whether it is in the foreground, so the
/// tracking loop can slow down while the window is in the background.
#[tauri::command]
pub fn set_window_active(state: State<'_, AppState>, active: bool) {
    state.set_window_active(active);
}

#[derive(Serialize)]
pub struct UpdateInfoDto {
    pub version: String,
    pub current_version: String,
    pub notes: Option<String>,
}

/// Asks the release feed whether a newer build exists.
///
/// The feed is the `latest.json` published next to every GitHub release, so a
/// pushed tag is what the user sees here.
#[tauri::command]
pub async fn check_for_update(app: tauri::AppHandle) -> Result<Option<UpdateInfoDto>, String> {
    use tauri_plugin_updater::UpdaterExt;

    let updater = app.updater().map_err(|error| error.to_string())?;
    let update = updater.check().await.map_err(|error| error.to_string())?;

    Ok(update.map(|update| UpdateInfoDto {
        version: update.version.clone(),
        current_version: update.current_version.clone(),
        notes: update.body.clone(),
    }))
}

#[derive(Clone, Serialize)]
pub struct UpdateProgressDto {
    pub downloaded: u64,
    pub total: Option<u64>,
}

/// A release that was downloaded, verified and is waiting for the user to
/// decide when the application restarts to finish installing it.
#[derive(Default)]
pub struct PendingUpdate(Mutex<Option<DownloadedUpdate>>);

impl PendingUpdate {
    pub fn new() -> Self {
        Self::default()
    }
}

struct DownloadedUpdate {
    update: Update,
    bytes: Vec<u8>,
    version: String,
}

#[tauri::command]
pub async fn download_update(
    app: tauri::AppHandle,
    pending: State<'_, PendingUpdate>,
) -> Result<String, String> {
    use tauri_plugin_updater::UpdaterExt;

    let updater = app.updater().map_err(|error| error.to_string())?;
    let update = updater
        .check()
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "No update available.".to_string())?;
    let version = update.version.clone();

    let emitter = app.clone();
    let mut downloaded: u64 = 0;
    let bytes = update
        .download(
            move |chunk_length, total| {
                downloaded += chunk_length as u64;
                let _ = emitter.emit("update-progress", UpdateProgressDto { downloaded, total });
            },
            || {},
        )
        .await
        .map_err(|error| error.to_string())?;

    let mut slot = pending.0.lock().map_err(|error| error.to_string())?;
    *slot = Some(DownloadedUpdate {
        update,
        bytes,
        version: version.clone(),
    });
    drop(slot);

    crate::logging::write(&format!("update {version}: downloaded"));
    let _ = app.emit("update-downloaded", version.clone());
    Ok(version)
}

/// Installs the downloaded release and restarts.
#[tauri::command]
pub async fn apply_update(
    app: tauri::AppHandle,
    pending: State<'_, PendingUpdate>,
) -> Result<(), String> {
    let downloaded = {
        let mut slot = pending.0.lock().map_err(|error| error.to_string())?;
        slot.take()
    }
    .ok_or_else(|| "No downloaded update. Check for updates again.".to_string())?;

    crate::logging::write(&format!("update {}: installing", downloaded.version));
    downloaded
        .update
        .install(&downloaded.bytes)
        .map_err(|error| {
            crate::logging::write(&format!("update {}: failed: {error}", downloaded.version));
            error.to_string()
        })?;

    app.restart();
}

#[derive(Serialize)]
pub struct AppInfoDto {
    pub name: String,
    pub version: String,
    pub repository: String,
}

/// Name, version and project link, shown in the header, the footer and the
/// settings panel.
#[tauri::command]
pub fn app_info(app: tauri::AppHandle) -> AppInfoDto {
    AppInfoDto {
        name: app.package_info().name.clone(),
        version: app.package_info().version.to_string(),
        repository: REPOSITORY_URL.to_string(),
    }
}

/// Opens the project page in the default browser.
#[tauri::command]
pub fn open_repository(app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url(REPOSITORY_URL, None::<&str>)
        .map_err(|error| error.to_string())
}

/// Restarts Steam at its own login window so the user can sign in to an
/// account this application has never seen. Nothing is stored here: Steam
/// performs the sign-in and writes `loginusers.vdf` itself.
#[tauri::command]
pub async fn add_steam_account(state: State<'_, AppState>) -> Result<SnapshotDto, String> {
    let operation = state.begin_steam_operation()?;
    let manager = state
        .lock()
        .manager
        .clone()
        .ok_or_else(|| "No Steam installation selected.".to_string())?;

    let result = tauri::async_runtime::spawn_blocking(move || {
        let _operation = operation;
        manager.add_account()
    })
    .await
    .map_err(|error| error.to_string())?;

    match result {
        Ok(()) => {
            state.refresh_accounts();
            state.lock().message =
                "Steam restarted at the login window. Sign in with the account to add.".to_string();
            Ok(build_snapshot(&state))
        }
        Err(error) => {
            state.lock().message = error.to_string();
            Err(error.to_string())
        }
    }
}

#[tauri::command]
pub async fn switch_account(
    state: State<'_, AppState>,
    steam_id: String,
) -> Result<SnapshotDto, String> {
    let operation = state.begin_steam_operation()?;
    let manager = state
        .lock()
        .manager
        .clone()
        .ok_or_else(|| "No Steam installation selected.".to_string())?;

    let running = manager.is_steam_running();
    crate::logging::write(&format!(
        "switch {steam_id}: requested (steam running: {running})"
    ));
    let started = std::time::Instant::now();

    let identifier = steam_id.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let _operation = operation;
        manager.switch_account(&identifier, true)
    })
    .await
    .map_err(|error| error.to_string())?;

    let elapsed = started.elapsed();
    match result {
        Ok(outcome) => {
            crate::logging::write(&format!(
                "switch {steam_id}: {} in {:.1}s",
                outcome.account_name(),
                elapsed.as_secs_f32()
            ));
            state.refresh_accounts();
            let account_name = outcome.account_name().to_string();
            let display = display_name_for(&state, &steam_id, &account_name);
            let mut inner = state.lock();
            inner.message = if outcome.already_active() {
                format!("{display} is already the active Steam account.")
            } else if outcome.started() {
                format!("Steam started on {display}.")
            } else {
                format!("Switched to {display}.")
            };
            drop(inner);
            Ok(build_snapshot(&state))
        }
        Err(error) => {
            crate::logging::write(&format!(
                "switch {steam_id}: failed in {:.1}s: {error}",
                elapsed.as_secs_f32()
            ));
            state.lock().message = error.to_string();
            Err(error.to_string())
        }
    }
}

#[tauri::command]
pub async fn remove_account(
    state: State<'_, AppState>,
    steam_id: String,
    cleanup_userdata: bool,
) -> Result<SnapshotDto, String> {
    let operation = state.begin_steam_operation()?;
    let manager = state
        .lock()
        .manager
        .clone()
        .ok_or_else(|| "No Steam installation selected.".to_string())?;

    let identifier = steam_id.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let _operation = operation;
        manager.remove_account(&identifier, cleanup_userdata)
    })
    .await
    .map_err(|error| error.to_string())?;

    match result {
        Ok(()) => {
            state.refresh_accounts();
            state.lock().message = "Account removed.".to_string();
            Ok(build_snapshot(&state))
        }
        Err(error) => {
            state.lock().message = error.to_string();
            Err(error.to_string())
        }
    }
}

#[tauri::command]
pub async fn restart_steam(state: State<'_, AppState>) -> Result<SnapshotDto, String> {
    let operation = state.begin_steam_operation()?;
    let manager = state
        .lock()
        .manager
        .clone()
        .ok_or_else(|| "No Steam installation selected.".to_string())?;

    let result = tauri::async_runtime::spawn_blocking(move || {
        let _operation = operation;
        manager.restart_steam()
    })
    .await
    .map_err(|error| error.to_string())?;

    match result {
        Ok(()) => {
            state.lock().message = "Steam restarted.".to_string();
            Ok(build_snapshot(&state))
        }
        Err(error) => {
            state.lock().message = error.to_string();
            Err(error.to_string())
        }
    }
}

#[tauri::command]
pub fn set_rank(
    state: State<'_, AppState>,
    steam_id: String,
    template: String,
    rank: String,
) -> Result<SnapshotDto, String> {
    let (account, ranks) = {
        let inner = state.lock();
        let account = inner
            .accounts
            .iter()
            .find(|account| account.steam_id == steam_id)
            .cloned()
            .ok_or_else(|| format!("Unknown Steam account: {steam_id}"))?;
        let ranks = inner
            .ranks
            .clone()
            .ok_or_else(|| "No rank store.".to_string())?;
        (account, ranks)
    };

    let mut ranks = ranks;
    ranks
        .set_rank(&account, &template, &rank)
        .map_err(|error| error.to_string())?;
    {
        let mut inner = state.lock();
        inner.ranks = Some(ranks);
    }
    Ok(build_snapshot(&state))
}

#[tauri::command]
pub fn account_report(state: State<'_, AppState>, steam_id: String) -> Result<ReportDto, String> {
    let (tracker, account, ranks, catalog) = report_context(&state, Some(&steam_id))?;
    let Some(account) = account else {
        return Err(format!("Unknown Steam account: {steam_id}"));
    };

    let tracker = tracker.ok_or_else(|| "Local playtime tracking is not available.".to_string())?;
    let tracker = tracker
        .lock()
        .map_err(|_| "Local playtime tracking is busy.".to_string())?;

    Ok(report_for_account(
        &tracker,
        &account,
        ranks.as_ref(),
        &catalog,
    ))
}

#[tauri::command]
pub fn machine_report(state: State<'_, AppState>) -> Result<ReportDto, String> {
    let (tracker, _account, _ranks, catalog) = report_context(&state, None)?;
    let tracker = tracker.ok_or_else(|| "Local playtime tracking is not available.".to_string())?;
    let tracker = tracker
        .lock()
        .map_err(|_| "Local playtime tracking is busy.".to_string())?;

    let games = tracker.machine_games();
    let accounts = tracker.account_playtimes();
    let tracked = accounts
        .iter()
        .filter(|account| account.seconds > 0.0 || account.active)
        .count();
    let statistics = tracker.history_statistics(None);

    Ok(ReportDto {
        account_id: None,
        title: "This PC - Local Playtime".to_string(),
        subtitle: "Local sessions detected by Steam Account Switcher.".to_string(),
        summary_text: format!(
            "Total: {}  |  {} {}  |  {} {}",
            format_playtime(tracker.global_seconds(true)),
            games.len(),
            plural(games.len(), "game"),
            tracked,
            plural(tracked, "account")
        ),
        statistics_text: statistics_text(&statistics),
        notice_text: notice_text(&statistics),
        games: Vec::new(),
        machine_games: games.into_iter().map(machine_game_dto).collect(),
        accounts: accounts
            .into_iter()
            .map(|account| account_playtime_dto(account, &catalog))
            .collect(),
        daily7: daily_points(tracker.daily_playtime(7, None)),
        daily30: daily_points(tracker.daily_playtime(30, None)),
        daily90: daily_points(tracker.daily_playtime(90, None)),
        sessions: tracker
            .session_history(None, true)
            .iter()
            .map(|session| session_dto(session, &catalog))
            .collect(),
        machine: true,
    })
}

#[tauri::command]
pub fn autostart_status(app: tauri::AppHandle) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch()
        .is_enabled()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_autostart(app: tauri::AppHandle, enabled: bool) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    let manager = app.autolaunch();
    let result = if enabled {
        manager.enable()
    } else {
        manager.disable()
    };
    result.map_err(|error| error.to_string())?;
    manager.is_enabled().map_err(|error| error.to_string())
}

/// Everything a playtime report needs: the tracker, the optional account, the
/// rank store and the account catalog used to render names and avatars.
type ReportContext = (
    Option<Arc<std::sync::Mutex<PlaytimeTracker>>>,
    Option<SteamAccount>,
    Option<RankStore>,
    Catalog,
);

fn report_context(state: &AppState, steam_id: Option<&str>) -> Result<ReportContext, String> {
    let inner = state.lock();
    let account = steam_id.and_then(|steam_id| {
        inner
            .accounts
            .iter()
            .find(|account| account.steam_id == steam_id)
            .cloned()
    });
    let catalog = Catalog {
        accounts: inner.accounts.clone(),
        masked: inner.masked,
    };
    Ok((inner.tracker.clone(), account, inner.ranks.clone(), catalog))
}
