use super::{AuthError, VaultStatus};
use crate::error_code::ErrorCode;
use crate::storage::Database;
use aes_gcm::aead::{rand_core::RngCore, Aead, OsRng};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::Engine as _;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use steamguard::SteamGuardAccount;
use zeroize::Zeroizing;

pub(super) const VAULT_VERSION: u32 = 1;
pub(super) const KDF_MEMORY_KIB: u32 = 19 * 1024;
pub(super) const KDF_ITERATIONS: u32 = 2;
pub(super) const KDF_PARALLELISM: u32 = 1;
pub(super) const KEY_LENGTH: usize = 32;
pub(super) const NONCE_LENGTH: usize = 12;
pub(super) const SALT_LENGTH: usize = 16;

pub(super) fn derive_key(
    passphrase: &str,
    salt: &[u8],
) -> Result<Zeroizing<[u8; KEY_LENGTH]>, AuthError> {
    let params = Params::new(
        KDF_MEMORY_KIB,
        KDF_ITERATIONS,
        KDF_PARALLELISM,
        Some(KEY_LENGTH),
    )
    .map_err(|_| AuthError::Code(ErrorCode::VaultEncryptionFailed))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = Zeroizing::new([0u8; KEY_LENGTH]);
    argon
        .hash_password_into(passphrase.as_bytes(), salt, key.as_mut())
        .map_err(|_| AuthError::Code(ErrorCode::VaultEncryptionFailed))?;
    Ok(key)
}

pub(super) fn seal(
    key: &[u8; KEY_LENGTH],
    plaintext: &[u8],
) -> Result<(Vec<u8>, Vec<u8>), AuthError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| AuthError::Code(ErrorCode::VaultEncryptionFailed))?;
    let mut nonce = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce);
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext)
        .map_err(|_| AuthError::Code(ErrorCode::VaultEncryptionFailed))?;
    Ok((nonce.to_vec(), ciphertext))
}

pub(super) fn open(
    key: &[u8; KEY_LENGTH],
    nonce: &[u8],
    ciphertext: &[u8],
) -> Result<Vec<u8>, AuthError> {
    if nonce.len() != NONCE_LENGTH {
        return Err(AuthError::InvalidVault);
    }
    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|_| AuthError::Code(ErrorCode::VaultOpenFailed))?;
    cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|_| AuthError::WrongPassphrase)
}

pub(super) fn base64_encode(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

pub(super) fn base64_decode(value: &str) -> Result<Vec<u8>, AuthError> {
    base64::engine::general_purpose::STANDARD
        .decode(value)
        .map_err(|_| AuthError::InvalidData)
}

#[derive(Debug, Serialize, Deserialize)]
struct PlainVault {
    #[serde(deserialize_with = "deserialize_accounts")]
    pub(super) accounts: BTreeMap<String, SteamGuardAccount>,
    #[serde(default)]
    pub(super) pending: BTreeSet<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct VaultFile {
    pub(super) version: u32,
    #[serde(default)]
    pub(super) encrypted: bool,
    #[serde(default)]
    pub(super) passwordless: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) kdf: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) salt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) nonce: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) ciphertext: Option<String>,
    /// Account IDs are public metadata. They never authorize vault operations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) account_ids: Option<BTreeSet<String>>,
    #[serde(default, deserialize_with = "deserialize_accounts")]
    pub(super) accounts: BTreeMap<String, SteamGuardAccount>,
    #[serde(default)]
    pub(super) pending: BTreeSet<String>,
}

impl Default for VaultFile {
    fn default() -> Self {
        Self {
            version: VAULT_VERSION,
            encrypted: false,
            passwordless: false,
            kdf: None,
            salt: None,
            nonce: None,
            ciphertext: None,
            account_ids: None,
            accounts: BTreeMap::new(),
            pending: BTreeSet::new(),
        }
    }
}

pub(super) struct Vault {
    pub(super) database: Database,
    pub(super) accounts: BTreeMap<String, SteamGuardAccount>,
    pub(super) encrypted: bool,
    pub(super) locked: bool,
    pub(super) key: Option<Zeroizing<[u8; KEY_LENGTH]>>,
    pub(super) salt: Option<[u8; SALT_LENGTH]>,
    pub(super) pending: BTreeSet<String>,
    pub(super) load_error: Option<String>,
    known_ids: BTreeSet<String>,
    indexed: bool,
}

/// steamguard's shared-secret deserializer panics on invalid lengths. Validate
/// external data before calling it so malformed backups cannot crash the app.
pub(super) fn deserialize_accounts<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, SteamGuardAccount>, D::Error> {
    let values = BTreeMap::<String, serde_json::Value>::deserialize(deserializer)?;
    values
        .into_iter()
        .map(|(id, value)| {
            let secret = value
                .get("shared_secret")
                .and_then(|value| value.as_str())
                .unwrap_or_default();
            validate_secret(secret).map_err(serde::de::Error::custom)?;
            let identity = value
                .get("identity_secret")
                .and_then(|value| value.as_str())
                .unwrap_or_default();
            validate_secret(identity).map_err(serde::de::Error::custom)?;
            let account: SteamGuardAccount =
                serde_json::from_value(value).map_err(serde::de::Error::custom)?;
            if id != account.steam_id.to_string() || account.steam_id == 0 {
                return Err(serde::de::Error::custom(
                    "Vault account ID does not match its entry.",
                ));
            }
            Ok((id, account))
        })
        .collect()
}

pub(super) fn validate_secret(secret: &str) -> Result<Vec<u8>, AuthError> {
    let bytes = base64_decode(secret)?;
    if bytes.len() != 20 {
        return Err(AuthError::InvalidData);
    }
    Ok(bytes)
}

fn storage_error(error: crate::error::SteamError) -> AuthError {
    AuthError::Io(error.to_string())
}

fn read_vault(database: &Database) -> Result<Option<VaultFile>, AuthError> {
    let text: Option<String> = database
        .read("Read vault", |connection| {
            connection
                .query_row("SELECT body FROM vault WHERE id = 1", [], |row| row.get(0))
                .optional()
        })
        .map_err(storage_error)?;
    let Some(text) = text.map(Zeroizing::new) else {
        return Ok(None);
    };
    let file: VaultFile = serde_json::from_str(&text).map_err(|_| AuthError::InvalidVault)?;
    if file.version != VAULT_VERSION {
        return Err(AuthError::Code(ErrorCode::UnsupportedVaultFormat));
    }
    if (file.encrypted && file.passwordless)
        || (!file.encrypted
            && (file.kdf.is_some()
                || file.salt.is_some()
                || file.nonce.is_some()
                || file.ciphertext.is_some()))
    {
        return Err(AuthError::InvalidVault);
    }
    if file.encrypted && file.kdf.as_deref() != Some("argon2id") {
        return Err(AuthError::Code(ErrorCode::UnsupportedVaultProtection));
    }
    Ok(Some(file))
}

pub(super) fn write_vault(database: &Database, text: &str) -> Result<(), AuthError> {
    database
        .transaction("Save vault", |transaction| {
            transaction.execute(
                "INSERT INTO vault (id, body) VALUES (1, ?1)
                 ON CONFLICT(id) DO UPDATE SET body = excluded.body",
                [text],
            )?;
            Ok(())
        })
        .map_err(storage_error)
}

impl Vault {
    pub(super) fn load(database: Database) -> Self {
        let mut vault = Self {
            database,
            accounts: BTreeMap::new(),
            encrypted: false,
            locked: false,
            key: None,
            salt: None,
            pending: BTreeSet::new(),
            load_error: None,
            known_ids: BTreeSet::new(),
            indexed: true,
        };

        let file = match read_vault(&vault.database) {
            Ok(Some(file)) => file,
            Ok(None) => return vault,
            Err(error) => {
                vault.load_error = Some(error.to_string());
                vault.locked = true;
                vault.indexed = false;
                return vault;
            }
        };

        vault.encrypted = file.encrypted;
        if file.encrypted {
            vault.locked = true;
            vault.indexed = file.account_ids.is_some();
            vault.known_ids = file.account_ids.unwrap_or_default();
        } else {
            vault.accounts = file.accounts;
            vault.pending = file.pending;
        }
        vault
    }

    pub(super) fn status(&self) -> VaultStatus {
        VaultStatus {
            path: self.database.path().display().to_string(),
            encrypted: self.encrypted,
            locked: self.locked,
            indexed: self.indexed,
            count: self.configured_ids().len(),
            error: self.load_error.clone(),
        }
    }

    pub(super) fn configured_ids(&self) -> BTreeSet<String> {
        if self.locked {
            return self.known_ids.clone();
        }
        self.accounts
            .keys()
            .filter(|id| !self.pending.contains(*id))
            .cloned()
            .collect()
    }

    pub(super) fn require_unlocked(&self) -> Result<(), AuthError> {
        if let Some(error) = &self.load_error {
            Err(AuthError::Code(
                ErrorCode::from_code(error).unwrap_or(ErrorCode::InvalidVault),
            ))
        } else if self.locked {
            Err(AuthError::Locked)
        } else {
            Ok(())
        }
    }

    pub(super) fn unlock(&mut self, passphrase: &str) -> Result<(), AuthError> {
        let file = read_vault(&self.database)?.ok_or(AuthError::InvalidVault)?;

        if !file.encrypted {
            self.accounts = file.accounts;
            self.encrypted = false;
            self.locked = false;
            self.pending = file.pending;
            self.key = None;
            self.salt = None;
            self.known_ids.clear();
            self.indexed = true;
            self.load_error = None;
            return Ok(());
        }

        let salt = base64_decode(file.salt.as_deref().unwrap_or_default())
            .map_err(|_| AuthError::InvalidVault)?;
        let nonce = base64_decode(file.nonce.as_deref().unwrap_or_default())
            .map_err(|_| AuthError::InvalidVault)?;
        let ciphertext = base64_decode(file.ciphertext.as_deref().unwrap_or_default())
            .map_err(|_| AuthError::InvalidVault)?;
        if salt.len() != SALT_LENGTH || ciphertext.len() < 16 {
            return Err(AuthError::InvalidVault);
        }
        let key = derive_key(passphrase, &salt)?;
        let plaintext = Zeroizing::new(open(&key, &nonce, &ciphertext)?);
        let plain: PlainVault =
            serde_json::from_slice(&plaintext).map_err(|_| AuthError::InvalidVault)?;

        self.accounts = plain.accounts;
        self.pending = plain.pending;
        self.encrypted = true;
        self.locked = false;
        self.key = Some(key);
        self.salt = salt.try_into().ok();
        self.load_error = None;
        self.known_ids = self.configured_ids();
        self.indexed = true;
        // Repair the public index using the authenticated payload.
        // If the durable write fails, keep secrets locked and the old file intact.
        if file.account_ids.as_ref() != Some(&self.known_ids) {
            if let Err(error) = self.save() {
                self.lock();
                return Err(error);
            }
        }
        Ok(())
    }

    pub(super) fn lock(&mut self) {
        if self.encrypted {
            self.known_ids = self.configured_ids();
            self.indexed = true;
            self.locked = true;
            self.accounts.clear();
            self.key = None;
            self.salt = None;
            self.pending.clear();
        }
    }

    /// None explicitly selects no-password storage; an empty password is invalid.
    /// Changing or removing an existing password requires its current value.
    /// Conversion requires an unlocked vault and rolls back if persistence fails.
    pub(super) fn set_passphrase(
        &mut self,
        passphrase: Option<String>,
        current_password: Option<String>,
    ) -> Result<(), AuthError> {
        self.require_unlocked()?;
        let passphrase = passphrase.map(Zeroizing::new);
        let current_password = current_password.map(Zeroizing::new);
        if self.encrypted {
            let password = current_password
                .as_ref()
                .filter(|value| !value.is_empty())
                .ok_or(AuthError::Code(ErrorCode::CurrentPasswordRequired))?;
            let candidate = derive_key(password, &self.salt.ok_or(AuthError::Locked)?)?;
            // Authenticate the candidate key using GCM's constant-time tag check.
            // Do not reload accounts or mutate the vault during verification.
            let (nonce, proof) = seal(
                self.key.as_ref().ok_or(AuthError::Locked)?,
                b"vault password verification",
            )?;
            open(&candidate, &nonce, &proof)?;
        }
        let (key, salt) = match passphrase.as_ref() {
            Some(value) => {
                if value.trim().is_empty() {
                    return Err(AuthError::Code(ErrorCode::VaultPasswordRequired));
                }
                let mut salt = [0u8; SALT_LENGTH];
                OsRng.fill_bytes(&mut salt);
                (Some(derive_key(value, &salt)?), Some(salt))
            }
            None => (None, None),
        };
        let previous = (self.encrypted, self.key.take(), self.salt);
        self.encrypted = passphrase.is_some();
        self.key = key;
        self.salt = salt;
        if let Err(error) = self.save() {
            (self.encrypted, self.key, self.salt) = previous;
            return Err(error);
        }
        Ok(())
    }

    /// A vault without a password is usable as-is; nothing is stored before the first save.
    pub(super) fn require_available(&self) -> Result<(), AuthError> {
        self.require_unlocked()
    }

    pub(super) fn save(&self) -> Result<(), AuthError> {
        self.require_available()?;
        let file = if self.encrypted {
            let key = self.key.as_ref().ok_or(AuthError::Locked)?;
            let plain = Zeroizing::new(
                serde_json::to_vec(&PlainVault {
                    accounts: self.accounts.clone(),
                    pending: self.pending.clone(),
                })
                .map_err(|_| AuthError::Code(ErrorCode::InvalidAuthenticatorData))?,
            );
            let (nonce, ciphertext) = seal(key, &plain)?;
            VaultFile {
                encrypted: true,
                passwordless: false,
                kdf: Some("argon2id".to_string()),
                salt: Some(base64_encode(&self.salt.ok_or(AuthError::Locked)?)),
                nonce: Some(base64_encode(&nonce)),
                ciphertext: Some(base64_encode(&ciphertext)),
                account_ids: Some(self.configured_ids()),
                ..VaultFile::default()
            }
        } else {
            VaultFile {
                passwordless: true,
                accounts: self.accounts.clone(),
                pending: self.pending.clone(),
                ..VaultFile::default()
            }
        };
        let text = Zeroizing::new(
            serde_json::to_string_pretty(&file)
                .map_err(|_| AuthError::Code(ErrorCode::InvalidAuthenticatorData))?,
        );
        write_vault(&self.database, &text)
    }

    pub(super) fn insert(&mut self, account: SteamGuardAccount) -> Result<(), AuthError> {
        self.require_available()?;
        let key = account.steam_id.to_string();
        let old = self.accounts.insert(key.clone(), account);
        if let Err(error) = self.save() {
            match old {
                Some(account) => {
                    self.accounts.insert(key, account);
                }
                None => {
                    self.accounts.remove(&key);
                }
            }
            return Err(error);
        }
        Ok(())
    }

    pub(super) fn remove(
        &mut self,
        steam_id: &str,
    ) -> Result<Option<SteamGuardAccount>, AuthError> {
        self.require_unlocked()?;
        let removed = self.accounts.remove(steam_id);
        let was_pending = self.pending.remove(steam_id);
        if removed.is_some() {
            if let Err(error) = self.save() {
                if let Some(account) = &removed {
                    self.accounts.insert(steam_id.to_string(), account.clone());
                }
                if was_pending {
                    self.pending.insert(steam_id.to_string());
                }
                return Err(error);
            }
        }
        Ok(removed)
    }

    pub(super) fn get(&self, steam_id: &str) -> Result<&SteamGuardAccount, AuthError> {
        self.require_unlocked()?;
        self.accounts.get(steam_id).ok_or(AuthError::Missing)
    }
}
