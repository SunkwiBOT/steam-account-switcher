use crate::error_code::ErrorCode;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::storage::Database;
use aes_gcm::aead::{rand_core::RngCore, OsRng};
use base64::Engine as _;
use steamguard::token::Tokens;
use steamguard::transport::WebApiTransport;
use steamguard::userlogin::UserLogin;
use steamguard::{AccountLinker, SteamGuardAccount};

pub const CODE_PERIOD_SECONDS: u64 = 30;

pub type Login = UserLogin<LoginTransport>;
pub type Linker = AccountLinker<WebApiTransport>;

mod vault;
use vault::Vault;
mod backup;
pub use backup::BackupImport;

mod import;
pub use import::import_mafile;

mod errors;
mod guard;
mod link;
mod login;
mod transport;
mod types;

pub use errors::*;
pub use guard::*;
pub use link::*;
pub use login::*;
pub use transport::*;
pub use types::*;

pub struct Authenticator {
    vault: Vault,
    logins: BTreeMap<String, Login>,
    linkers: BTreeMap<String, PendingLink>,
    remembered: BTreeMap<u64, Tokens>,
    login_controls: BTreeMap<String, Arc<AtomicBool>>,
    active_links: BTreeSet<String>,
}

impl Authenticator {
    pub fn new(database: Database) -> Self {
        Self {
            vault: Vault::load(database),
            logins: BTreeMap::new(),
            linkers: BTreeMap::new(),
            remembered: BTreeMap::new(),
            login_controls: BTreeMap::new(),
            active_links: BTreeSet::new(),
        }
    }

    pub fn status(&self) -> VaultStatus {
        self.vault.status()
    }

    pub fn is_locked(&self) -> bool {
        self.vault.locked
    }

    /// Presentation hints only: secrets and tokens still require unlocking.
    pub fn configured_ids(&self) -> Vec<String> {
        self.vault.configured_ids().into_iter().collect()
    }

    pub fn unlock(&mut self, passphrase: &str) -> Result<VaultStatus, AuthError> {
        self.vault.unlock(passphrase)?;
        Ok(self.vault.status())
    }

    pub fn lock(&mut self) -> Result<(), AuthError> {
        if !self.vault.encrypted {
            return Err(AuthError::Code(ErrorCode::VaultPasswordRequiredToLock));
        }
        if !self.active_links.is_empty() {
            return Err(AuthError::Busy);
        }
        self.vault.lock();
        for control in self.login_controls.values() {
            control.store(true, Ordering::SeqCst);
        }
        self.login_controls.clear();
        self.logins.clear();
        self.linkers.clear();
        self.remembered.clear();
        Ok(())
    }

    pub fn set_passphrase(
        &mut self,
        passphrase: Option<String>,
        current_password: Option<String>,
    ) -> Result<VaultStatus, AuthError> {
        if !self.active_links.is_empty() {
            return Err(AuthError::Busy);
        }
        self.vault.set_passphrase(passphrase, current_password)?;
        Ok(self.vault.status())
    }

    pub fn accounts(&self) -> Vec<StoredAuthenticator> {
        if self.vault.locked {
            return Vec::new();
        }
        self.vault
            .accounts
            .values()
            .filter(|account| !self.vault.pending.contains(&account.steam_id.to_string()))
            .map(|account| StoredAuthenticator {
                steam_id: account.steam_id.to_string(),
                account_name: account.account_name.clone(),
                serial_number: account.serial_number.clone(),
                device_id: account.device_id.clone(),
                has_tokens: account.tokens.is_some(),
            })
            .collect()
    }

    pub fn has(&self, steam_id: &str) -> bool {
        !self.vault.locked
            && self.vault.accounts.contains_key(steam_id)
            && !self.vault.pending.contains(steam_id)
    }

    pub fn code(&self, steam_id: &str) -> Result<CodeInfo, AuthError> {
        let account = self.vault.get(steam_id)?;
        let time = now_seconds();
        Ok(CodeInfo {
            valid_until: time + seconds_remaining(time),
            code: account.generate_code(time),
            seconds_remaining: seconds_remaining(time),
            period: CODE_PERIOD_SECONDS,
        })
    }

    pub fn revocation_code(&self, steam_id: &str) -> Result<String, AuthError> {
        use steamguard::ExposeSecret;
        Ok(self
            .vault
            .get(steam_id)?
            .revocation_code
            .expose_secret()
            .to_string())
    }

    pub fn import(
        &mut self,
        path: &Path,
        passkey: Option<&str>,
        expected_steam_id: Option<&str>,
    ) -> Result<StoredAuthenticator, AuthError> {
        self.vault.require_available()?;
        let account = import_mafile(path, passkey)?;
        if let Some(expected) = expected_steam_id.filter(|value| !value.is_empty()) {
            if expected != account.steam_id.to_string() {
                return Err(AuthError::Code(ErrorCode::AccountMismatch));
            }
        }
        if self
            .vault
            .accounts
            .contains_key(&account.steam_id.to_string())
        {
            return Err(AuthError::Code(ErrorCode::AuthenticatorAlreadySaved));
        }
        let stored = StoredAuthenticator {
            steam_id: account.steam_id.to_string(),
            account_name: account.account_name.clone(),
            serial_number: account.serial_number.clone(),
            device_id: account.device_id.clone(),
            has_tokens: account.tokens.is_some(),
        };
        self.vault.insert(account)?;
        Ok(stored)
    }

    /// Copy secrets under the lock; the caller releases it before network work.
    pub fn confirmation_account(&self, steam_id: &str) -> Result<SteamGuardAccount, AuthError> {
        self.vault.require_available()?;
        if self.vault.pending.contains(steam_id) {
            return Err(AuthError::Code(ErrorCode::ActivationRequired));
        }
        let account = self.vault.get(steam_id)?.clone();
        if account.tokens.is_none() {
            return Err(AuthError::NeedsLogin);
        }
        Ok(account)
    }

    pub fn remember_tokens(&mut self, account: &SteamGuardAccount) -> Result<(), AuthError> {
        // ! A lock or removal while the request was in flight must not restore
        // ! secrets, and an older response must not overwrite a newer login.
        if self.vault.locked {
            return Ok(());
        }
        let key = account.steam_id.to_string();
        if let Some(stored) = self.vault.accounts.get(&key) {
            let same_session = match (&stored.tokens, &account.tokens) {
                (Some(old), Some(new)) => {
                    old.refresh_token().expose_secret() == new.refresh_token().expose_secret()
                }
                _ => false,
            };
            let unchanged = match (&stored.tokens, &account.tokens) {
                (Some(old), Some(new)) => {
                    old.access_token().expose_secret() == new.access_token().expose_secret()
                }
                _ => false,
            };
            if same_session
                && !unchanged
                && stored.shared_secret == account.shared_secret
                && stored.device_id == account.device_id
            {
                let mut updated = stored.clone();
                updated.tokens = account.tokens.clone();
                self.vault.insert(updated)?;
            }
        }
        Ok(())
    }

    pub fn forget_revoked(&mut self, steam_id: &str) -> Result<(), AuthError> {
        self.remembered.remove(&steam_id.parse().unwrap_or(0));
        if self.vault.locked {
            return Err(AuthError::Locked);
        }
        self.vault.remove(steam_id)?;
        Ok(())
    }

    pub fn forget_local(&mut self, steam_id: &str) -> Result<(), AuthError> {
        self.vault.require_unlocked()?;
        if !self.active_links.is_empty()
            || !self.logins.is_empty()
            || !self.linkers.is_empty()
            || !self.login_controls.is_empty()
        {
            return Err(AuthError::Busy);
        }
        self.vault.get(steam_id)?;
        self.vault.remove(steam_id)?;
        self.remembered.remove(&steam_id.parse().unwrap_or(0));
        Ok(())
    }

    fn next_session(&mut self) -> String {
        let mut random = [0u8; 16];
        OsRng.fill_bytes(&mut random);
        format!(
            "s{}",
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(random)
        )
    }

    pub fn prepare_login(&mut self) -> Result<(String, Login), AuthError> {
        self.vault.require_available()?;
        let session_id = self.next_session();
        let control = Arc::new(AtomicBool::new(false));
        let login = new_login(control.clone());
        self.login_controls.insert(session_id.clone(), control);
        Ok((session_id, login))
    }

    pub fn store_login(&mut self, session_id: &str, login: Login) -> Result<(), AuthError> {
        self.vault.require_available()?;
        if !self.login_controls.contains_key(session_id) {
            return Err(AuthError::UnknownSession);
        }
        self.logins.insert(session_id.to_string(), login);
        Ok(())
    }

    /// Moves a pending login out so the caller can block on it without holding
    /// the vault lock.
    pub fn take_login(&mut self, session_id: &str) -> Result<Login, AuthError> {
        self.logins
            .remove(session_id)
            .ok_or(AuthError::UnknownSession)
    }

    pub fn put_login(&mut self, session_id: &str, login: Login) {
        if self
            .login_controls
            .get(session_id)
            .is_some_and(|control| !control.load(Ordering::SeqCst))
        {
            self.logins.insert(session_id.to_string(), login);
        } else {
            self.login_controls.remove(session_id);
        }
    }

    pub fn finish_link_operation(&mut self, session_id: &str) {
        self.active_links.remove(session_id);
    }

    pub fn cancel_session(&mut self, session_id: &str) {
        if let Some(control) = self.login_controls.remove(session_id) {
            control.store(true, Ordering::SeqCst);
        }
        self.logins.remove(session_id);
        self.linkers.remove(session_id);
    }

    pub fn absorb_login(
        &mut self,
        session_id: &str,
        tokens: Tokens,
        account_name: &str,
    ) -> Result<LoginResult, AuthError> {
        self.vault.require_available()?;
        let control = self
            .login_controls
            .remove(session_id)
            .ok_or(AuthError::UnknownSession)?;
        if control.load(Ordering::SeqCst) {
            return Err(AuthError::UnknownSession);
        }
        let steam_id = token_steam_id(&tokens)?;
        self.remembered.insert(steam_id, tokens.clone());

        let key = steam_id.to_string();
        let mut stored = false;
        if !self.vault.locked {
            if let Some(account) = self.vault.accounts.get_mut(&key) {
                account.tokens = Some(tokens);
                stored = true;
            }
            if stored {
                self.vault.save()?;
            }
        }

        Ok(LoginResult {
            session_id: session_id.to_string(),
            steam_id: key,
            account_name: account_name.to_string(),
            stored,
        })
    }

    /// ! Checks the vault is writable before making a remote change, and copies
    /// ! the pending material out: network requests never hold the store mutex.
    pub fn prepare_link(
        &mut self,
        steam_id: u64,
        account_name: &str,
    ) -> Result<(String, PendingLink), AuthError> {
        self.vault.require_available()?;
        self.vault.save()?;
        if !self.active_links.is_empty() {
            return Err(AuthError::Busy);
        }
        // ! A disk error after AddAuthenticator keeps this session and its
        // ! secrets in memory; a retry must save and resume it, never start a
        // ! new link, or the only copy of the secret is lost.
        if let Some(session_id) = self
            .linkers
            .iter()
            .find_map(|(id, pending)| (pending.steam_id == steam_id).then(|| id.clone()))
        {
            let pending = self.take_linker(&session_id)?;
            return Ok((session_id, pending));
        }
        let key = steam_id.to_string();
        let account = if self.vault.pending.contains(&key) {
            Some(self.vault.get(&key)?.clone())
        } else {
            None
        };
        let tokens = self.tokens_for(steam_id)?;
        let session_id = self.next_session();
        self.active_links.insert(session_id.clone());
        Ok((
            session_id,
            PendingLink {
                linker: AccountLinker::new(transport(), tokens),
                account,
                account_name: account_name.to_string(),
                steam_id,
                transferring: false,
            },
        ))
    }

    pub fn store_link(&mut self, session_id: &str, pending: PendingLink) -> Result<(), AuthError> {
        let saved = if let Some(account) = &pending.account {
            let key = account.steam_id.to_string();
            self.vault.pending.insert(key.clone());
            if let Err(error) = self.vault.insert(account.clone()) {
                self.vault.accounts.insert(key, account.clone());
                Err(error)
            } else {
                Ok(())
            }
        } else {
            Ok(())
        };
        self.put_linker(session_id, pending);
        saved
    }

    pub fn take_linker(&mut self, session_id: &str) -> Result<PendingLink, AuthError> {
        self.vault.require_available()?;
        self.vault.save()?;
        let pending = self
            .linkers
            .remove(session_id)
            .ok_or(AuthError::UnknownSession)?;
        self.active_links.insert(session_id.to_string());
        Ok(pending)
    }

    pub fn put_linker(&mut self, session_id: &str, pending: PendingLink) {
        self.active_links.remove(session_id);
        self.linkers.insert(session_id.to_string(), pending);
    }

    pub fn absorb_link(
        &mut self,
        session_id: &str,
        account: SteamGuardAccount,
    ) -> Result<StoredAuthenticator, AuthError> {
        let stored = StoredAuthenticator {
            steam_id: account.steam_id.to_string(),
            account_name: account.account_name.clone(),
            serial_number: account.serial_number.clone(),
            device_id: account.device_id.clone(),
            has_tokens: account.tokens.is_some(),
        };
        let key = account.steam_id.to_string();
        let was_pending = self.vault.pending.remove(&key);
        if let Err(error) = self.vault.insert(account.clone()) {
            self.vault.accounts.insert(key.clone(), account);
            if was_pending {
                self.vault.pending.insert(key);
            }
            self.active_links.remove(session_id);
            return Err(error);
        }
        self.linkers.remove(session_id);
        self.active_links.remove(session_id);
        Ok(stored)
    }

    fn tokens_for(&self, steam_id: u64) -> Result<Tokens, AuthError> {
        self.vault.require_available()?;
        if let Some(tokens) = self.remembered.get(&steam_id) {
            return Ok(tokens.clone());
        }
        self.vault
            .accounts
            .get(&steam_id.to_string())
            .and_then(|account| account.tokens.clone())
            .ok_or(AuthError::NeedsLogin)
    }
}
