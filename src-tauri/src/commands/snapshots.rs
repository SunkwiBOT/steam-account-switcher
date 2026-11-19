use super::dto::*;
use crate::{avatars, state::AppState};
use steam_core::metadata::RankChoice;
use steam_core::model::{is_selected_account, masked_identity, MASKED_LABEL};
use steam_core::playtime::ActiveGame;
use steam_core::ranks;
use tauri::State;

fn rank_templates() -> Vec<RankTemplateDto> {
    ranks::templates()
        .iter()
        .map(|template| RankTemplateDto {
            id: template.id.to_string(),
            label: template.label.to_string(),
            kind: template.kind,
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

fn active_game_dto(game: ActiveGame) -> ActiveGameDto {
    ActiveGameDto {
        app_id: game.app_id,
        name: game.name,
        account_id: game.account_id,
        elapsed_seconds: game.elapsed_seconds,
    }
}

fn masked_account_name(account: &steam_core::model::SteamAccount, masked: bool) -> String {
    if masked {
        MASKED_LABEL.to_string()
    } else {
        account.account_name.clone()
    }
}

pub(crate) fn build_snapshot(state: &AppState) -> SnapshotDto {
    let accounts = state.display_accounts();
    let inner = state.lock();
    let revision = state.next_snapshot_revision();
    let tracker = inner.tracker.clone();
    let ranks = inner.ranks.clone();
    let masked = inner.masked;
    let accounts_error = inner.accounts_error.map(|code| code.as_str().to_string());
    let steam_ids_visible = inner.steam_ids_visible;
    let local_ids: Vec<String> = inner
        .accounts
        .iter()
        .map(|account| account.steam_id.clone())
        .collect();
    let auto_login_user = inner.auto_login_user.clone();
    let installations = inner.installations.clone();
    let detected = inner.detected.clone();
    let custom = inner.custom.clone();
    let settings_data = inner.settings.data().clone();
    let database_path = state.database.path().display().to_string();
    let selected = inner.selected;
    let vault_lock_in_seconds = inner.unlocked_at.and_then(|unlocked_at| {
        settings_data
            .vault_lock_interval
            .seconds()
            .map(|seconds| seconds.saturating_sub(unlocked_at.elapsed().as_secs()))
    });
    let manager = inner.manager.clone();
    drop(inner);
    // ! Enumerating every process takes a while, so it runs after releasing the state lock to avoid blocking the tray and window close handler.
    let steam_running = manager
        .as_ref()
        .map(|manager| manager.is_steam_running())
        .unwrap_or(false);
    let (authenticators, configured_ids, vault_status) = {
        let authenticator = state.authenticator();
        let guard = authenticator
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        (guard.accounts(), guard.configured_ids(), guard.status())
    };

    let (machine_seconds, active_games, tracker_ready, per_account) = match &tracker {
        Some(tracker) => match tracker.lock() {
            Ok(tracker) => {
                let per_account: Vec<(String, f64, usize)> = accounts
                    .iter()
                    .map(|account| {
                        (
                            account.steam_id.clone(),
                            tracker.account_seconds(&account.steam_id, true),
                            tracker.session_count(Some(&account.steam_id)),
                        )
                    })
                    .collect();
                let active = tracker
                    .active_games()
                    .into_iter()
                    .map(active_game_dto)
                    .collect();
                let machine = tracker.global_seconds(true);
                let ready = tracker.enabled() && tracker.log_path().exists();
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
                .map(|template| template.kind)
                .unwrap_or(ranks::RankKind::Tiered);
            let family = ranks::family(&choice.template, &choice.value);
            let playtime = per_account
                .iter()
                .find(|(steam_id, _, _)| steam_id == &account.steam_id)
                .map(|(_, seconds, sessions)| (*seconds, *sessions))
                .unwrap_or((0.0, 0));

            let is_local = local_ids.contains(&account.steam_id);
            let (display_name, initial) = masked_identity(account, masked);
            AccountDto {
                steam_id: account.steam_id.clone(),
                account_name: masked_account_name(account, masked),
                persona_name: account.persona_name.clone(),
                display_name,
                initial,
                avatar: account.avatar_path.as_deref().and_then(avatars::data_url),
                rank: choice.value.clone(),
                rank_template: choice.template.clone(),
                rank_kind: kind,
                rank_family: family,
                last_login_seconds: account.timestamp.unwrap_or(0),
                is_current: is_local && is_selected_account(account, &auto_login_user),
                is_local,
                session_count: playtime.1,
                color: settings_data
                    .account_color(&account.steam_id)
                    .map(str::to_string),
                note: settings_data
                    .account_note(&account.steam_id)
                    .map(str::to_string),
                folder_id: if is_local {
                    settings_data
                        .account_folder(&account.steam_id)
                        .filter(|id| *id != crate::retained_accounts::SAVED_GUARD_FOLDER_ID)
                        .map(str::to_string)
                } else {
                    Some(crate::retained_accounts::SAVED_GUARD_FOLDER_ID.to_string())
                },
                has_authenticator: configured_ids.contains(&account.steam_id),
                has_authenticator_session: authenticators
                    .iter()
                    .any(|entry| entry.steam_id == account.steam_id && entry.has_tokens),
                playtime_seconds: playtime.0,
            }
        })
        .collect();

    let mut folders: Vec<FolderDto> = settings_data
        .folders
        .iter()
        .filter(|folder| folder.id != crate::retained_accounts::SAVED_GUARD_FOLDER_ID)
        .map(|folder| FolderDto {
            id: folder.id.clone(),
            name: folder.name.clone(),
            system: false,
            color: folder.color.clone(),
            icon: folder.icon.clone(),
            account_count: account_dtos
                .iter()
                .filter(|account| account.folder_id.as_deref() == Some(folder.id.as_str()))
                .count(),
        })
        .collect();

    let retained_count = account_dtos
        .iter()
        .filter(|account| !account.is_local)
        .count();
    if retained_count > 0 {
        folders.insert(
            0,
            FolderDto {
                id: crate::retained_accounts::SAVED_GUARD_FOLDER_ID.to_string(),
                name: crate::retained_accounts::SAVED_GUARD_FOLDER_NAME.to_string(),
                account_count: retained_count,
                system: true,
                color: None,
                icon: None,
            },
        );
    }

    SnapshotDto {
        folders,
        settings: SettingsDto {
            language: settings_data.language,
            resolved_language: settings_data.language.resolved().to_string(),
            show_installation_selector: settings_data.show_installation_selector,
            tracking_enabled: settings_data.tracking_enabled,
            tracking_history: super::tracking::tracking_history(
                &state.database,
                &detected.iter().chain(&custom).cloned().collect::<Vec<_>>(),
            ),
            close_to_tray: settings_data.close_to_tray,
            update_interval: settings_data.update_interval,
            search_scope: settings_data.search_scope,
            vault_lock_interval: settings_data.vault_lock_interval,
            last_update_check_at: settings_data.last_update_check_at,
            database_path,
            entries: detected
                .iter()
                .chain(custom.iter())
                .map(|installation| {
                    let root = crate::state::root_key(installation);
                    InstallationSettingDto {
                        kind: installation.kind,
                        custom: settings_data.is_custom(&root),
                        hidden: settings_data.is_hidden(&root),
                        root,
                    }
                })
                .collect(),
            vault_offer_dismissed: settings_data.vault_offer_dismissed,
        },
        revision,
        installations: installations
            .iter()
            .enumerate()
            .map(|(index, installation)| InstallationDto {
                index,
                kind: installation.kind,
                root: installation.root.display().to_string(),
            })
            .collect(),
        selected,
        accounts: account_dtos,
        accounts_error,
        masked,
        steam_ids_visible,
        machine_seconds,
        active_games,
        tracker_ready,
        steam_running,
        rank_templates: rank_templates(),
        vault: VaultStatusDto {
            status: vault_status,
            lock_in_seconds: vault_lock_in_seconds,
        },
    }
}

#[tauri::command(async)]
pub fn snapshot(state: State<'_, AppState>) -> SnapshotDto {
    build_snapshot(&state)
}
#[tauri::command(async)]
pub fn playtime_snapshot(state: State<'_, AppState>) -> PlaytimeSnapshotDto {
    let (accounts, tracker) = {
        let inner = state.lock();
        (inner.accounts.clone(), inner.tracker.clone())
    };

    let Some(tracker) = tracker else {
        return PlaytimeSnapshotDto {
            machine_seconds: 0.0,
            tracker_ready: false,
            accounts: Vec::new(),
            active_games: Vec::new(),
        };
    };

    let Ok(tracker) = tracker.lock() else {
        return PlaytimeSnapshotDto {
            machine_seconds: 0.0,
            tracker_ready: false,
            accounts: Vec::new(),
            active_games: Vec::new(),
        };
    };

    let cells: Vec<AccountPlaytimeCellDto> = accounts
        .iter()
        .map(|account| {
            let seconds = tracker.account_seconds(&account.steam_id, true);
            let session_count = tracker.session_count(Some(&account.steam_id));
            AccountPlaytimeCellDto {
                steam_id: account.steam_id.clone(),
                seconds,
                session_count,
            }
        })
        .collect();

    let machine_seconds = tracker.global_seconds(true);
    PlaytimeSnapshotDto {
        machine_seconds,
        tracker_ready: tracker.enabled() && tracker.log_path().exists(),
        accounts: cells,
        active_games: tracker
            .active_games()
            .into_iter()
            .map(active_game_dto)
            .collect(),
    }
}

#[tauri::command(async)]
pub fn refresh(app: tauri::AppHandle, state: State<'_, AppState>) -> SnapshotDto {
    crate::logging::write("action=refresh status=requested");
    state.refresh_accounts();
    crate::tray::refresh_menu(&app);
    build_snapshot(&state)
}
