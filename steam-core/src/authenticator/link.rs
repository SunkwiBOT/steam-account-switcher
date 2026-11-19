use super::errors::{steam_error, AuthError};
use super::link_wal::WalContext;
use super::transport::transport;
use super::types::{LinkConfirmKind, LinkStart, LinkStatus};
use super::Linker;
use crate::error_code::ErrorCode;
use steamguard::accountlinker::AccountLinkConfirmType;
use steamguard::SteamGuardAccount;

pub struct PendingLink {
    pub(super) linker: Linker,
    pub(super) account: Option<SteamGuardAccount>,
    pub(super) account_name: String,
    pub(super) steam_id: u64,
    pub(super) transferring: bool,
    pub(super) wal: WalContext,
    // ! Set the moment Steam accepted a step, independent of whether the WAL write that
    // followed succeeded: a failed write must not make the next call resend an already
    // accepted `confirm_code`/`sms_code`, which Steam may reject as stale.
    accepted: bool,
    last_wal_warning: Option<String>,
}

impl PendingLink {
    pub(super) fn new(
        linker: Linker,
        account: Option<SteamGuardAccount>,
        account_name: String,
        steam_id: u64,
        wal: WalContext,
    ) -> Self {
        Self {
            linker,
            account,
            account_name,
            steam_id,
            transferring: false,
            wal,
            accepted: false,
            last_wal_warning: None,
        }
    }

    pub fn account(&self) -> Option<&SteamGuardAccount> {
        self.account.as_ref()
    }

    // ! A WAL write failure after Steam already accepted the step is a storage warning, not a link failure, kept here for the caller to log.
    pub fn take_wal_warning(&mut self) -> Option<String> {
        self.last_wal_warning.take()
    }

    pub fn start(&mut self, session_id: &str) -> Result<LinkStart, AuthError> {
        use steamguard::ExposeSecret;
        let mut result = LinkStart {
            session_id: session_id.to_string(),
            status: LinkStatus::Linked,
            steam_id: self.steam_id.to_string(),
            account_name: self.account_name.clone(),
            revocation_code: None,
            phone_hint: None,
            confirm_kind: None,
        };
        if let Some(account) = &self.account {
            result.revocation_code = Some(account.revocation_code.expose_secret().to_string());
            result.status = LinkStatus::Resume;
            return Ok(result);
        }
        match self.linker.link() {
            Ok(success) => {
                result.phone_hint = Some(success.phone_number_hint().to_string());
                result.confirm_kind = Some(match success.confirm_type() {
                    AccountLinkConfirmType::SMS => LinkConfirmKind::Sms,
                    AccountLinkConfirmType::Email => LinkConfirmKind::Email,
                    AccountLinkConfirmType::Unknown(_) => LinkConfirmKind::Unknown,
                });
                let account = success.into_account();
                result.revocation_code = Some(account.revocation_code.expose_secret().to_string());
                self.account = Some(account);
            }
            Err(steamguard::AccountLinkError::AuthenticatorPresent) => {
                result.status = LinkStatus::AuthenticatorPresent;
            }
            Err(steamguard::AccountLinkError::MustProvidePhoneNumber) => {
                result.status = LinkStatus::NeedsPhone;
            }
            Err(steamguard::AccountLinkError::MustConfirmEmail) => {
                result.status = LinkStatus::NeedsEmail;
            }
            Err(error) => return Err(steam_error(error)),
        }
        Ok(result)
    }

    pub fn steam_id(&self) -> u64 {
        self.steam_id
    }

    pub fn account_name(&self) -> &str {
        &self.account_name
    }

    // ! Idempotent: a step already accepted (in memory, or replayed from the WAL across
    // processes) replays success instead of resending `confirm_code`, which Steam may reject
    // as stale.
    pub fn finalize(&mut self, confirm_code: &str) -> Result<(), AuthError> {
        if self.accepted || self.wal.is_accepted(self.steam_id)? {
            if let Some(account) = self.wal.load_accepted(self.steam_id)? {
                self.account = Some(account);
            }
            self.accepted = true;
            return Ok(());
        }

        let account = self
            .account
            .as_mut()
            .ok_or(AuthError::Code(ErrorCode::LinkNotStarted))?;
        self.wal.before(self.steam_id, account)?;

        let transport = transport();
        let mut server_time = steamguard::steamapi::get_server_time(transport.clone())
            .map_err(steam_error)?
            .server_time();

        for _ in 0..30 {
            match self
                .linker
                .finalize(server_time, account, confirm_code.to_string())
            {
                Ok(()) => {
                    let confirmed = account.clone();
                    self.accepted = true;
                    self.note_wal_write("finalize", &confirmed);
                    return Ok(());
                }
                Err(steamguard::FinalizeLinkError::WantMore { server_time: next }) => {
                    server_time = next;
                }
                Err(error) => return Err(steam_error(error)),
            }
        }

        Err(AuthError::Code(ErrorCode::LinkFailed))
    }

    pub fn verify(&self) -> Result<(), AuthError> {
        let account = self
            .account
            .as_ref()
            .ok_or(AuthError::Code(ErrorCode::LinkNotStarted))?;
        let status = self.linker.query_status(account).map_err(steam_error)?;
        if status.state() == 0 {
            return Err(AuthError::Code(ErrorCode::LinkNotConfirmed));
        }
        Ok(())
    }

    pub fn transfer_start(&mut self) -> Result<(), AuthError> {
        self.linker
            .transfer_start()
            .map_err(|_| AuthError::Code(ErrorCode::TransferStartFailed))?;
        self.transferring = true;
        Ok(())
    }

    // ! Idempotent the same way `finalize` is: a step already accepted (in memory, or replayed
    // from the WAL) is replayed instead of resending `sms_code`.
    pub fn transfer_finish(&mut self, sms_code: &str) -> Result<SteamGuardAccount, AuthError> {
        if !self.transferring {
            return Err(AuthError::Code(ErrorCode::TransferNotStarted));
        }
        if self.accepted || self.wal.is_accepted(self.steam_id)? {
            if let Some(account) = self.wal.load_accepted(self.steam_id)? {
                self.account = Some(account.clone());
                self.accepted = true;
                return Ok(account);
            }
        }
        if let Some(account) = &self.account {
            self.wal.before(self.steam_id, account)?;
        }
        let account = self
            .linker
            .transfer_finish(sms_code)
            .map_err(|_| AuthError::Code(ErrorCode::TransferFailed))?;
        // ! Steam already invalidated the old authenticator; keep the new secret in memory before touching the WAL so a failed write never drops it.
        self.account = Some(account.clone());
        self.accepted = true;
        self.note_wal_write("transfer_finish", &account);
        Ok(account)
    }

    pub fn revoke(&mut self, revocation_code: &str) -> Result<(), AuthError> {
        self.linker
            .remove_authenticator(Some(&revocation_code.to_string()))
            .map_err(steam_error)
    }

    // ! The WAL is a safety net, not a gate: a failed write after Steam already accepted the step becomes a safe-to-log warning, not a failure.
    pub(super) fn note_wal_write(&mut self, step: &'static str, account: &SteamGuardAccount) {
        if let Err(error) = self.wal.after_accepted(self.steam_id, account) {
            self.last_wal_warning = Some(format!("after {step}: {error}"));
        }
    }

    pub fn take_account(&mut self) -> Option<SteamGuardAccount> {
        self.account.take()
    }
}
