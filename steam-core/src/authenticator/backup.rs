use super::vault::{
    base64_decode, base64_encode, derive_key, deserialize_accounts, header_aad,
    kdf_params_in_range, open, seal, KDF_ITERATIONS, KDF_ITERATIONS_V1, KDF_MEMORY_KIB,
    KDF_MEMORY_KIB_V1, KDF_PARALLELISM, KDF_PARALLELISM_V1, NONCE_LENGTH, SALT_LENGTH,
};
use super::{io_error, AuthError, Authenticator};
use crate::error_code::ErrorCode;
use aes_gcm::aead::{rand_core::RngCore, OsRng};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;
use steamguard::SteamGuardAccount;
use zeroize::Zeroizing;

// ! Exports always write `BACKUP_FORMAT`; imports also accept `BACKUP_FORMAT_ALIAS` since both apps share the same payload shape.
const BACKUP_FORMAT: &str = "steam-account-switcher-authenticators";
const BACKUP_FORMAT_ALIAS: &str = "steam-guard-desktop-authenticators";
// ! Bumped alongside the vault format; version 1 (fixed cost, no associated data) is still read, exports always write this version.
const BACKUP_VERSION: u32 = 2;
const MAX_BACKUP_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackupPayload {
    #[serde(deserialize_with = "deserialize_accounts")]
    accounts: BTreeMap<String, SteamGuardAccount>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "protection", rename_all = "snake_case", deny_unknown_fields)]
enum BackupData {
    None {
        payload: BackupPayload,
    },
    Password {
        // ! Present from version 2 onward: the Argon2id cost this file was sealed at, so raising the default cost never breaks an older backup.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kdf_memory_kib: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kdf_iterations: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kdf_parallelism: Option<u32>,
        salt: String,
        nonce: String,
        ciphertext: String,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackupFile {
    format: String,
    version: u32,
    data: BackupData,
}

#[derive(Serialize)]
pub struct BackupImport {
    pub imported: usize,
    pub skipped: usize,
}

fn read_backup(path: &Path, password: Option<&str>) -> Result<BackupPayload, AuthError> {
    let mut bytes = Zeroizing::new(Vec::new());
    std::fs::File::open(path)
        .map_err(io_error)?
        .take(MAX_BACKUP_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(io_error)?;
    if bytes.len() as u64 > MAX_BACKUP_BYTES {
        return Err(AuthError::InvalidBackup);
    }
    let file: BackupFile = serde_json::from_slice(&bytes).map_err(|_| AuthError::InvalidBackup)?;
    if !matches!(file.format.as_str(), BACKUP_FORMAT | BACKUP_FORMAT_ALIAS) {
        return Err(AuthError::Code(ErrorCode::UnsupportedBackupFormat));
    }
    if file.version > BACKUP_VERSION || file.version < 1 {
        return Err(AuthError::Code(ErrorCode::UnsupportedBackupFormat));
    }
    let mut payload = match file.data {
        BackupData::None { payload } => payload,
        BackupData::Password {
            kdf_memory_kib,
            kdf_iterations,
            kdf_parallelism,
            salt,
            nonce,
            ciphertext,
        } => {
            let password = password
                .filter(|value| !value.is_empty())
                .ok_or(AuthError::Code(ErrorCode::BackupPasswordRequired))?;
            let salt = base64_decode(&salt).map_err(|_| AuthError::InvalidBackup)?;
            let nonce = base64_decode(&nonce).map_err(|_| AuthError::InvalidBackup)?;
            let ciphertext = base64_decode(&ciphertext).map_err(|_| AuthError::InvalidBackup)?;
            if salt.len() != SALT_LENGTH || nonce.len() != NONCE_LENGTH || ciphertext.len() < 16 {
                return Err(AuthError::InvalidBackup);
            }
            let (memory_kib, iterations, parallelism) =
                match (kdf_memory_kib, kdf_iterations, kdf_parallelism) {
                    (Some(memory_kib), Some(iterations), Some(parallelism)) => {
                        (memory_kib, iterations, parallelism)
                    }
                    // ! Version 1 never stored its cost: it was always this fixed one.
                    _ => (KDF_MEMORY_KIB_V1, KDF_ITERATIONS_V1, KDF_PARALLELISM_V1),
                };
            if file.version >= 2
                && (kdf_memory_kib.is_none()
                    || kdf_iterations.is_none()
                    || kdf_parallelism.is_none())
            {
                return Err(AuthError::InvalidBackup);
            }
            if !kdf_params_in_range(memory_kib, iterations, parallelism) {
                return Err(AuthError::InvalidBackup);
            }
            let key = derive_key(password, &salt, memory_kib, iterations, parallelism)?;
            let aad = if file.version >= 2 {
                header_aad(
                    "steam-backup",
                    file.version,
                    memory_kib,
                    iterations,
                    parallelism,
                )
            } else {
                Vec::new()
            };
            let plaintext = Zeroizing::new(
                open(&key, &nonce, &ciphertext, &aad)
                    .map_err(|_| AuthError::BackupDecryptionFailed)?,
            );
            serde_json::from_slice(&plaintext).map_err(|_| AuthError::InvalidBackup)?
        }
    };
    if payload.accounts.is_empty() {
        return Err(AuthError::Code(ErrorCode::BackupEmpty));
    }
    for account in payload.accounts.values_mut() {
        if account.device_id.trim().is_empty() {
            return Err(AuthError::Code(ErrorCode::BackupDeviceMissing));
        }
        account.tokens = None;
    }
    Ok(payload)
}

impl Authenticator {
    pub fn export_backup(
        &self,
        path: &Path,
        steam_id: Option<&str>,
        password: Option<&str>,
    ) -> Result<usize, AuthError> {
        self.vault.require_available()?;
        let database = self.vault.database.path();
        if path == database
            || std::fs::canonicalize(path)
                .ok()
                .zip(std::fs::canonicalize(database).ok())
                .is_some_and(|(target, vault)| target == vault)
        {
            return Err(AuthError::Code(ErrorCode::BackupTargetUnsafe));
        }
        if let Some(id) = steam_id {
            self.vault.get(id)?;
            if self.vault.pending.contains(id) {
                return Err(AuthError::Code(ErrorCode::ActivationRequired));
            }
        }
        let accounts: BTreeMap<_, _> = self
            .vault
            .accounts
            .iter()
            .filter(|(id, _)| {
                !self.vault.pending.contains(*id) && steam_id.is_none_or(|selected| selected == *id)
            })
            .map(|(id, account)| {
                let mut account = account.clone();
                account.tokens = None;
                (id.clone(), account)
            })
            .collect();
        let count = accounts.len();
        if count == 0 {
            return Err(AuthError::Code(ErrorCode::BackupEmpty));
        }
        let payload = BackupPayload { accounts };
        let data = match password {
            Some(password) => {
                if password.trim().is_empty() {
                    return Err(AuthError::Code(ErrorCode::BackupPasswordRequired));
                }
                let mut salt = [0u8; SALT_LENGTH];
                OsRng.fill_bytes(&mut salt);
                let key = derive_key(
                    password,
                    &salt,
                    KDF_MEMORY_KIB,
                    KDF_ITERATIONS,
                    KDF_PARALLELISM,
                )?;
                let plaintext = Zeroizing::new(
                    serde_json::to_vec(&payload)
                        .map_err(|_| AuthError::Code(ErrorCode::BackupExportFailed))?,
                );
                let aad = header_aad(
                    "steam-backup",
                    BACKUP_VERSION,
                    KDF_MEMORY_KIB,
                    KDF_ITERATIONS,
                    KDF_PARALLELISM,
                );
                let (nonce, ciphertext) = seal(&key, &plaintext, &aad)?;
                BackupData::Password {
                    kdf_memory_kib: Some(KDF_MEMORY_KIB),
                    kdf_iterations: Some(KDF_ITERATIONS),
                    kdf_parallelism: Some(KDF_PARALLELISM),
                    salt: base64_encode(&salt),
                    nonce: base64_encode(&nonce),
                    ciphertext: base64_encode(&ciphertext),
                }
            }
            None => BackupData::None { payload },
        };
        let text = Zeroizing::new(
            serde_json::to_string_pretty(&BackupFile {
                format: BACKUP_FORMAT.to_string(),
                version: BACKUP_VERSION,
                data,
            })
            .map_err(|_| AuthError::Code(ErrorCode::BackupExportFailed))?,
        );
        if text.len() as u64 > MAX_BACKUP_BYTES {
            return Err(AuthError::Code(ErrorCode::BackupExportFailed));
        }
        write_private_file(path, &text)?;
        Ok(count)
    }

    pub fn import_backup(
        &mut self,
        path: &Path,
        password: Option<&str>,
    ) -> Result<BackupImport, AuthError> {
        self.vault.require_available()?;
        if !self.active_links.is_empty() {
            return Err(AuthError::Busy);
        }
        let payload = read_backup(path, password)?;
        let mut inserted = Vec::new();
        let mut skipped = 0;
        for (id, account) in payload.accounts {
            if self.vault.accounts.contains_key(&id) {
                skipped += 1;
            } else {
                self.vault.accounts.insert(id.clone(), account);
                inserted.push(id);
            }
        }
        if !inserted.is_empty() {
            if let Err(error) = self.vault.save() {
                for id in &inserted {
                    self.vault.accounts.remove(id);
                }
                return Err(error);
            }
        }
        Ok(BackupImport {
            imported: inserted.len(),
            skipped,
        })
    }
}

// ! Backup files hold secrets even when encrypted; created owner-only and synced before replacing an older export.
fn write_private_file(path: &Path, text: &str) -> Result<(), AuthError> {
    crate::util::write_atomic(path, text, crate::util::WriteMode::Private(0o600))
        .map_err(|error| AuthError::Io(error.to_string()))
}
