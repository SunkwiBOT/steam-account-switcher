use super::dto::*;
use crate::{base64, state::AppState};
use std::sync::Arc;
use steam_core::error_code::ErrorCode;
use steam_core::metadata::RankStore;
use steam_core::model::SteamAccount;
use steam_core::playtime::{
    AccountPlaytime, DailyPlaytime, GamePlaytime, MachineGamePlaytime, PlaytimeTracker,
    SessionPlaytime,
};
use steam_core::ranks;
use tauri::State;

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
        } else if self.masked {
            "******".to_string()
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
        if self.masked && persona.is_empty() {
            return "*".to_string();
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

fn report_for_account(
    tracker: &PlaytimeTracker,
    account: &SteamAccount,
    ranks: Option<&RankStore>,
    catalog: &Catalog,
) -> ReportDto {
    let games = tracker.account_games(&account.steam_id);
    let session_count: u64 = games.iter().map(|game| game.sessions).sum();
    let statistics = tracker.history_statistics(Some(&account.steam_id));
    let choice = ranks.map(|store| store.choice_for(account));

    let daily7 = tracker.daily_playtime(7, Some(&account.steam_id));
    let daily30 = tracker.daily_playtime(30, Some(&account.steam_id));
    let daily90 = tracker.daily_playtime(90, Some(&account.steam_id));
    let sessions = tracker.session_history(Some(&account.steam_id), true);
    let game_count = games.len();

    ReportDto {
        account_id: Some(account.steam_id.clone()),
        total_seconds: tracker.account_seconds(&account.steam_id, true),
        game_count,
        account_count: 1,
        session_count,
        rank: choice
            .as_ref()
            .map(|choice| choice.value.clone())
            .unwrap_or_else(|| ranks::UNRANKED.to_string()),
        rank_template: choice
            .map(|choice| choice.template)
            .unwrap_or_else(|| ranks::DEFAULT_TEMPLATE.to_string()),
        statistics: statistics.into(),
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
        status: if game.active {
            "running".into()
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
        status: if game.active {
            "running".into()
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
        status: if account.active {
            "running".into()
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
        started_at: session.started_at,
        ended_at: session.ended_at,
        launch_timestamp: session.launch_timestamp,
        process_create_time: session.process_create_time,
        root_pid: session.root_pid,
        coverage: session.tracking_mode.clone(),
        end_reason: session.end_reason.clone(),
        exit_code: session.exit_code,
        status: if session.active {
            "running".into()
        } else {
            String::new()
        },
        seconds: session.duration_seconds,
    }
}

#[tauri::command(async)]
pub fn account_report(state: State<'_, AppState>, steam_id: String) -> Result<ReportDto, String> {
    crate::logging::read("account_report", || {
        let (tracker, account, ranks, catalog) = report_context(&state, Some(&steam_id))?;
        let Some(account) = account else {
            return Err(ErrorCode::AccountUnavailable.to_string());
        };

        let tracker = tracker.ok_or_else(|| ErrorCode::PlaytimeUnavailable.as_str().to_string())?;
        let tracker = tracker
            .lock()
            .map_err(|_| ErrorCode::PlaytimeBusy.to_string())?;

        Ok(report_for_account(
            &tracker,
            &account,
            ranks.as_ref(),
            &catalog,
        ))
    })
}

#[tauri::command(async)]
pub fn machine_report(state: State<'_, AppState>) -> Result<ReportDto, String> {
    crate::logging::read("machine_report", || {
        let (tracker, _account, _ranks, catalog) = report_context(&state, None)?;
        let tracker = tracker.ok_or_else(|| ErrorCode::PlaytimeUnavailable.as_str().to_string())?;
        let tracker = tracker
            .lock()
            .map_err(|_| ErrorCode::PlaytimeBusy.to_string())?;

        let games = tracker.machine_games();
        let accounts = tracker.account_playtimes();
        let tracked = accounts
            .iter()
            .filter(|account| account.seconds > 0.0 || account.active)
            .count();
        let statistics = tracker.history_statistics(None);

        Ok(ReportDto {
            account_id: None,
            total_seconds: tracker.global_seconds(true),
            game_count: games.len(),
            account_count: tracked,
            session_count: statistics.session_count as u64,
            rank: String::new(),
            rank_template: String::new(),
            statistics: statistics.into(),
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
    })
}

type ReportContext = (
    Option<Arc<std::sync::Mutex<PlaytimeTracker>>>,
    Option<SteamAccount>,
    Option<RankStore>,
    Catalog,
);

fn report_context(state: &AppState, steam_id: Option<&str>) -> Result<ReportContext, String> {
    let accounts = state.display_accounts();
    let inner = state.lock();
    let account = steam_id.and_then(|steam_id| {
        accounts
            .iter()
            .find(|account| account.steam_id == steam_id)
            .cloned()
    });
    let catalog = Catalog {
        accounts,
        masked: inner.masked,
    };
    Ok((inner.tracker.clone(), account, inner.ranks.clone(), catalog))
}
