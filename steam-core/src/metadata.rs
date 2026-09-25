//! Non-secret metadata Steam does not store, such as the rank badge.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::error::{Result, SteamError};
use crate::model::{SteamAccount, SteamInstallation};
use crate::ranks::{self, DEFAULT_TEMPLATE, UNRANKED};
use crate::util::write_atomic;

const METADATA_VERSION: u32 = 1;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountMetadata {
    #[serde(default)]
    pub account_name: String,
    #[serde(default)]
    pub persona_name: String,
    /// Rank ladder chosen for this account ("marvel-rivals", "cs2", ...).
    #[serde(default)]
    pub template: String,
    #[serde(default)]
    pub rank: String,
}

/// The ladder and the value stored for one account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankChoice {
    pub template: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataFile {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub accounts: BTreeMap<String, AccountMetadata>,
    #[serde(flatten, default)]
    pub extra: Map<String, Value>,
}

fn default_version() -> u32 {
    METADATA_VERSION
}

impl Default for MetadataFile {
    fn default() -> Self {
        Self {
            version: METADATA_VERSION,
            accounts: BTreeMap::new(),
            extra: Map::new(),
        }
    }
}

/// Stores the rank of each account next to the installation it belongs to.
#[derive(Debug, Clone)]
pub struct RankStore {
    path: PathBuf,
    data: MetadataFile,
}

impl RankStore {
    pub fn for_installation(installation: &SteamInstallation) -> Self {
        Self::load(
            installation
                .config_dir
                .join("steam-account-switcher-metadata.json"),
        )
    }

    pub fn load(path: PathBuf) -> Self {
        let data = std::fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<MetadataFile>(&text).ok())
            .unwrap_or_default();
        Self { path, data }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Ladder and value of an account.
    ///
    /// Accounts stored before templates existed have no `template` field and a
    /// Marvel Rivals rank, so that is the fallback.
    pub fn choice_for(&self, account: &SteamAccount) -> RankChoice {
        let Some(metadata) = self.data.accounts.get(&account_key(account)) else {
            return RankChoice {
                template: DEFAULT_TEMPLATE.to_string(),
                value: UNRANKED.to_string(),
            };
        };

        let template = if ranks::template(&metadata.template).is_some() {
            metadata.template.clone()
        } else {
            DEFAULT_TEMPLATE.to_string()
        };

        RankChoice {
            value: ranks::normalize(&template, &metadata.rank),
            template,
        }
    }

    /// Stores a rank, validating it against the chosen ladder.
    pub fn set_rank(
        &mut self,
        account: &SteamAccount,
        template: &str,
        value: &str,
    ) -> Result<RankChoice> {
        let template = if ranks::template(template).is_some() {
            template.to_string()
        } else {
            DEFAULT_TEMPLATE.to_string()
        };
        let value = ranks::normalize(&template, value);
        let key = account_key(account);
        let entry = self.data.accounts.entry(key).or_default();
        entry.template = template.clone();
        entry.rank = value.clone();
        entry.account_name = account.account_name.clone();
        entry.persona_name = account.persona_name.clone();
        self.save()?;
        Ok(RankChoice { template, value })
    }

    fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.data)
            .map_err(|error| SteamError::config(format!("Unable to encode rank data: {error}")))?;
        write_atomic(&self.path, &json).map_err(|error| {
            SteamError::io(format!("Unable to write {}", self.path.display()), error)
        })
    }
}

fn account_key(account: &SteamAccount) -> String {
    if account.steam_id.is_empty() {
        account.account_name.clone()
    } else {
        account.steam_id.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(steam_id: &str, name: &str) -> SteamAccount {
        SteamAccount {
            steam_id: steam_id.to_string(),
            account_name: name.to_string(),
            persona_name: name.to_string(),
            avatar_path: None,
            most_recent: false,
            remember_password: false,
            allow_auto_login: false,
            wants_offline_mode: false,
            skip_offline_mode_warning: false,
            timestamp: None,
        }
    }

    #[test]
    fn ranks_round_trip_through_disk() {
        let path = temp_path("round-trip");
        let first = account("76561198000000001", "first");

        let mut store = RankStore::load(path.clone());
        assert_eq!(store.choice_for(&first).value, UNRANKED);
        assert_eq!(store.choice_for(&first).template, DEFAULT_TEMPLATE);

        store
            .set_rank(&first, "marvel-rivals", "Diamond 2")
            .expect("set rank");
        let reloaded = RankStore::load(path.clone());
        let choice = reloaded.choice_for(&first);
        assert_eq!(choice.template, "marvel-rivals");
        assert_eq!(choice.value, "Diamond 2");
        assert_eq!(
            reloaded
                .choice_for(&account("76561198000000009", "other"))
                .value,
            UNRANKED
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn each_template_keeps_its_own_ladder() {
        let path = temp_path("templates");
        let account = account("76561198000000001", "player");
        let mut store = RankStore::load(path.clone());

        store
            .set_rank(&account, "overwatch", "Emerald 2")
            .expect("overwatch rank");
        assert_eq!(store.choice_for(&account).value, "Emerald 2");

        store
            .set_rank(&account, "cs2", "Global Elite")
            .expect("cs2 rank");
        let choice = store.choice_for(&account);
        assert_eq!(choice.template, "cs2");
        assert_eq!(choice.value, "Global Elite");

        // A rank from another ladder is not valid here.
        store
            .set_rank(&account, "cs2", "Diamond 3")
            .expect("invalid rank");
        assert_eq!(store.choice_for(&account).value, UNRANKED);

        store
            .set_rank(&account, "cs2-premier", "21 340")
            .expect("rating");
        assert_eq!(store.choice_for(&account).value, "21340");

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn ranks_stored_before_templates_still_load() {
        let path = temp_path("legacy");
        std::fs::write(
            &path,
            r#"{ "version": 1, "accounts": { "76561198000000001": { "rank": "Celestial 3" } } }"#,
        )
        .expect("legacy file");

        let store = RankStore::load(path.clone());
        let choice = store.choice_for(&account("76561198000000001", "player"));
        assert_eq!(choice.template, DEFAULT_TEMPLATE);
        assert_eq!(choice.value, "Celestial 3");

        let _ = std::fs::remove_file(path);
    }

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "steam-core-metadata-{name}-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or(0)
        ))
    }
}
