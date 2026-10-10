use std::collections::BTreeMap;

use crate::error::Result;
use crate::model::SteamAccount;
use crate::ranks::{self, DEFAULT_TEMPLATE, UNRANKED};
use crate::storage::Database;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankChoice {
    pub template: String,
    pub value: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Entry {
    template: String,
    rank: String,
}

#[derive(Debug, Clone)]
pub struct RankStore {
    database: Database,
    entries: BTreeMap<String, Entry>,
}

impl RankStore {
    /// Loads every stored choice. A database that cannot be read starts empty:
    /// the account then shows as unranked instead of blocking the application.
    pub fn load(database: Database) -> Self {
        let entries = database
            .read("Read rank choices", |connection| {
                let mut statement =
                    connection.prepare("SELECT steam_id, template, rank FROM rank_choice")?;
                let rows = statement.query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        Entry {
                            template: row.get(1)?,
                            rank: row.get(2)?,
                        },
                    ))
                })?;
                rows.collect::<rusqlite::Result<BTreeMap<_, _>>>()
            })
            .unwrap_or_default();
        Self { database, entries }
    }

    pub fn choice_for(&self, account: &SteamAccount) -> RankChoice {
        let Some(entry) = self.entries.get(&account_key(account)) else {
            return RankChoice {
                template: DEFAULT_TEMPLATE.to_string(),
                value: UNRANKED.to_string(),
            };
        };

        let template = if ranks::template(&entry.template).is_some() {
            entry.template.clone()
        } else {
            DEFAULT_TEMPLATE.to_string()
        };

        RankChoice {
            value: ranks::normalize(&template, &entry.rank),
            template,
        }
    }

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
        self.database
            .transaction("Save rank choice", |transaction| {
                transaction.execute(
                    "INSERT INTO rank_choice (steam_id, account_name, persona_name, template, rank)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(steam_id) DO UPDATE SET
                    account_name = excluded.account_name,
                    persona_name = excluded.persona_name,
                    template = excluded.template,
                    rank = excluded.rank",
                    (
                        &key,
                        &account.account_name,
                        &account.persona_name,
                        &template,
                        &value,
                    ),
                )?;
                Ok(())
            })?;
        self.entries.insert(
            key,
            Entry {
                template: template.clone(),
                rank: value.clone(),
            },
        );
        Ok(RankChoice { template, value })
    }
}

fn account_key(account: &SteamAccount) -> String {
    if account.steam_id.is_empty() {
        account.account_name.clone()
    } else {
        account.steam_id.clone()
    }
}
