// ! Durable write-ahead record for one link/finalize/transfer step: without it, a crash after Steam accepts the request would lose the only copy of its secret.

use super::vault::{base64_decode, base64_encode, open, seal, Vault, KEY_LENGTH};
use super::AuthError;
use crate::error_code::ErrorCode;
use crate::storage::Database;
use rusqlite::OptionalExtension;
use steamguard::SteamGuardAccount;
use zeroize::Zeroizing;

pub(super) struct WalContext {
    database: Database,
    key: Option<Zeroizing<[u8; KEY_LENGTH]>>,
    encrypted: bool,
}

struct Row {
    encrypted: bool,
    nonce: Option<String>,
    body: String,
    accepted: bool,
}

fn storage_error(error: crate::error::SteamError) -> AuthError {
    AuthError::Io(error.to_string())
}

fn write_row(
    database: &Database,
    steam_id: u64,
    encrypted: bool,
    nonce: Option<&[u8]>,
    body: &str,
    accepted: bool,
) -> Result<(), AuthError> {
    database
        .transaction("Save link write-ahead record", |transaction| {
            transaction.execute(
                "INSERT INTO link_wal (steam_id, encrypted, nonce, body, accepted)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(steam_id) DO UPDATE SET
                    encrypted = excluded.encrypted,
                    nonce = excluded.nonce,
                    body = excluded.body,
                    accepted = excluded.accepted",
                rusqlite::params![
                    steam_id.to_string(),
                    encrypted,
                    nonce.map(base64_encode),
                    body,
                    accepted,
                ],
            )?;
            Ok(())
        })
        .map_err(storage_error)
}

fn read_row(database: &Database, steam_id: u64) -> Result<Option<Row>, AuthError> {
    database
        .read("Read link write-ahead record", |connection| {
            connection
                .query_row(
                    "SELECT encrypted, nonce, body, accepted FROM link_wal WHERE steam_id = ?1",
                    [steam_id.to_string()],
                    |row| {
                        Ok(Row {
                            encrypted: row.get(0)?,
                            nonce: row.get(1)?,
                            body: row.get(2)?,
                            accepted: row.get(3)?,
                        })
                    },
                )
                .optional()
        })
        .map_err(storage_error)
}

fn read_all(database: &Database) -> Result<Vec<(u64, Row)>, AuthError> {
    database
        .read("Read link write-ahead records", |connection| {
            connection
                .prepare("SELECT steam_id, encrypted, nonce, body, accepted FROM link_wal")?
                .query_map([], |row| {
                    let steam_id: String = row.get(0)?;
                    Ok((
                        steam_id,
                        Row {
                            encrypted: row.get(1)?,
                            nonce: row.get(2)?,
                            body: row.get(3)?,
                            accepted: row.get(4)?,
                        },
                    ))
                })?
                .collect::<rusqlite::Result<Vec<_>>>()
        })
        .map_err(storage_error)
        .map(|rows| {
            rows.into_iter()
                .filter_map(|(id, row)| id.parse().ok().map(|id| (id, row)))
                .collect()
        })
}

fn decode(context: &WalContext, row: Row) -> Result<SteamGuardAccount, AuthError> {
    let plain = if row.encrypted {
        let key = context.key.as_ref().ok_or(AuthError::Locked)?;
        let nonce = base64_decode(row.nonce.as_deref().unwrap_or_default())?;
        let ciphertext = base64_decode(&row.body)?;
        open(key, &nonce, &ciphertext, b"")?
    } else {
        row.body.into_bytes()
    };
    serde_json::from_slice(&plain).map_err(|_| AuthError::InvalidData)
}

impl WalContext {
    pub(super) fn capture(vault: &Vault) -> Self {
        Self {
            database: vault.database.clone(),
            key: vault.key.clone(),
            encrypted: vault.encrypted,
        }
    }

    fn seal_account(
        &self,
        account: &SteamGuardAccount,
    ) -> Result<(bool, Option<Vec<u8>>, String), AuthError> {
        let plain = serde_json::to_vec(account)
            .map_err(|_| AuthError::Code(ErrorCode::InvalidAuthenticatorData))?;
        if self.encrypted {
            let key = self.key.as_ref().ok_or(AuthError::Locked)?;
            let (nonce, ciphertext) = seal(key, &plain, b"")?;
            Ok((true, Some(nonce), base64_encode(&ciphertext)))
        } else {
            Ok((
                false,
                None,
                String::from_utf8(plain).map_err(|_| AuthError::InvalidData)?,
            ))
        }
    }

    pub(super) fn before(
        &self,
        steam_id: u64,
        account: &SteamGuardAccount,
    ) -> Result<(), AuthError> {
        let (encrypted, nonce, body) = self.seal_account(account)?;
        write_row(
            &self.database,
            steam_id,
            encrypted,
            nonce.as_deref(),
            &body,
            false,
        )
    }

    pub(super) fn after_accepted(
        &self,
        steam_id: u64,
        account: &SteamGuardAccount,
    ) -> Result<(), AuthError> {
        let (encrypted, nonce, body) = self.seal_account(account)?;
        write_row(
            &self.database,
            steam_id,
            encrypted,
            nonce.as_deref(),
            &body,
            true,
        )
    }

    pub(super) fn is_accepted(&self, steam_id: u64) -> Result<bool, AuthError> {
        Ok(read_row(&self.database, steam_id)?.is_some_and(|row| row.accepted))
    }

    // ! The account from an `accepted` row, for a step retried after Steam already confirmed it in this process.
    pub(super) fn load_accepted(
        &self,
        steam_id: u64,
    ) -> Result<Option<SteamGuardAccount>, AuthError> {
        match read_row(&self.database, steam_id)? {
            Some(row) if row.accepted => Ok(Some(decode(self, row)?)),
            _ => Ok(None),
        }
    }

    // ! The account is durably saved in `vault` now; its write-ahead copy is redundant.
    pub(super) fn clear(&self, steam_id: u64) -> Result<(), AuthError> {
        clear(&self.database, steam_id)
    }
}

fn clear(database: &Database, steam_id: u64) -> Result<(), AuthError> {
    database
        .transaction("Clear link write-ahead record", |transaction| {
            transaction.execute(
                "DELETE FROM link_wal WHERE steam_id = ?1",
                [steam_id.to_string()],
            )?;
            Ok(())
        })
        .map_err(storage_error)
}

pub(super) fn recover(vault: &mut Vault) {
    if vault.locked {
        return;
    }
    let context = WalContext::capture(vault);
    let rows = match read_all(&vault.database) {
        Ok(rows) => rows,
        Err(_) => return,
    };
    for (steam_id, row) in rows {
        if !row.accepted {
            let _ = clear(&vault.database, steam_id);
            continue;
        }
        let Ok(account) = decode(&context, row) else {
            let _ = clear(&vault.database, steam_id);
            continue;
        };
        if vault.commit_link(account).is_ok() {
            let _ = clear(&vault.database, steam_id);
        }
    }
}
