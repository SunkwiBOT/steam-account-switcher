use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use steam_core::authenticator::StoredAuthenticator;
use steam_core::model::SteamAccount;

pub const SAVED_GUARD_FOLDER_ID: &str = "system:steam-guard";
pub const SAVED_GUARD_FOLDER_NAME: &str = "Saved Steam Guard";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedAccountProfile {
    pub account_name: String,
    pub persona_name: String,
    pub avatar_path: Option<PathBuf>,
    pub timestamp: Option<i64>,
}

impl SavedAccountProfile {
    fn from_account(account: &SteamAccount) -> Self {
        Self {
            account_name: account.account_name.clone(),
            persona_name: account.persona_name.clone(),
            avatar_path: account.avatar_path.clone(),
            timestamp: account.timestamp,
        }
    }

    fn to_account(&self, steam_id: &str) -> SteamAccount {
        SteamAccount {
            steam_id: steam_id.to_string(),
            account_name: self.account_name.clone(),
            persona_name: self.persona_name.clone(),
            avatar_path: self.avatar_path.clone(),
            timestamp: self.timestamp,
            most_recent: false,
            remember_password: false,
            allow_auto_login: false,
            wants_offline_mode: false,
            skip_offline_mode_warning: false,
        }
    }
}

pub fn remember_profiles(
    profiles: &mut BTreeMap<String, SavedAccountProfile>,
    local: &[SteamAccount],
    stored: &[StoredAuthenticator],
    configured_ids: &[String],
) -> bool {
    let mut changed = false;
    for id in configured_ids {
        let profile = local
            .iter()
            .find(|account| account.steam_id == *id)
            .map(SavedAccountProfile::from_account)
            .or_else(|| {
                if profiles.contains_key(id) {
                    return None;
                }
                stored
                    .iter()
                    .find(|account| account.steam_id == *id)
                    .map(|account| SavedAccountProfile {
                        account_name: account.account_name.clone(),
                        persona_name: String::new(),
                        avatar_path: None,
                        timestamp: None,
                    })
            });
        if let Some(profile) = profile {
            if profiles.get(id) != Some(&profile) {
                profiles.insert(id.clone(), profile);
                changed = true;
            }
        }
    }
    changed
}

pub fn display_accounts(
    local: &[SteamAccount],
    configured_ids: &[String],
    profiles: &BTreeMap<String, SavedAccountProfile>,
) -> Vec<SteamAccount> {
    let mut accounts = local.to_vec();
    for id in configured_ids {
        if accounts.iter().any(|account| account.steam_id == *id) {
            continue;
        }
        let fallback = SavedAccountProfile {
            account_name: String::new(),
            persona_name: String::new(),
            avatar_path: None,
            timestamp: None,
        };
        accounts.push(profiles.get(id).unwrap_or(&fallback).to_account(id));
    }
    accounts
}
