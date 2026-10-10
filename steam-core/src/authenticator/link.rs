use super::*;

pub struct PendingLink {
    pub(super) linker: Linker,
    pub(super) account: Option<SteamGuardAccount>,
    pub(super) account_name: String,
    pub(super) steam_id: u64,
    pub(super) transferring: bool,
}

impl PendingLink {
    pub fn start(&mut self, session_id: &str) -> Result<LinkStart, AuthError> {
        use steamguard::ExposeSecret;
        let mut result = LinkStart {
            session_id: session_id.to_string(),
            status: "linked".to_string(),
            steam_id: self.steam_id.to_string(),
            account_name: self.account_name.clone(),
            revocation_code: None,
            phone_hint: None,
            confirm_kind: None,
        };
        if let Some(account) = &self.account {
            result.revocation_code = Some(account.revocation_code.expose_secret().to_string());
            result.status = "resume".to_string();
            return Ok(result);
        }
        match self.linker.link() {
            Ok(success) => {
                result.phone_hint = Some(success.phone_number_hint().to_string());
                result.confirm_kind = Some(match success.confirm_type() {
                    AccountLinkConfirmType::SMS => "sms".to_string(),
                    AccountLinkConfirmType::Email => "email".to_string(),
                    AccountLinkConfirmType::Unknown(_) => "unknown".to_string(),
                });
                let account = success.into_account();
                result.revocation_code = Some(account.revocation_code.expose_secret().to_string());
                self.account = Some(account);
            }
            Err(steamguard::AccountLinkError::AuthenticatorPresent) => {
                result.status = "authenticator_present".to_string()
            }
            Err(steamguard::AccountLinkError::MustProvidePhoneNumber) => {
                result.status = "needs_phone".to_string();
            }
            Err(steamguard::AccountLinkError::MustConfirmEmail) => {
                result.status = "needs_email".to_string();
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

    pub fn has_account(&self) -> bool {
        self.account.is_some()
    }

    pub fn finalize(&mut self, confirm_code: &str) -> Result<(), AuthError> {
        let account = self
            .account
            .as_mut()
            .ok_or(AuthError::Code(ErrorCode::LinkNotStarted))?;

        let transport = transport();
        let mut server_time = steamguard::steamapi::get_server_time(transport.clone())
            .map_err(steam_error)?
            .server_time();

        for _ in 0..30 {
            match self
                .linker
                .finalize(server_time, account, confirm_code.to_string())
            {
                Ok(()) => return Ok(()),
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

    pub fn transfer_finish(&mut self, sms_code: &str) -> Result<SteamGuardAccount, AuthError> {
        if !self.transferring {
            return Err(AuthError::Code(ErrorCode::TransferNotStarted));
        }
        let account = self
            .linker
            .transfer_finish(sms_code)
            .map_err(|_| AuthError::Code(ErrorCode::TransferFailed))?;
        self.account = Some(account.clone());
        Ok(account)
    }

    pub fn revoke(&mut self, revocation_code: &str) -> Result<(), AuthError> {
        self.linker
            .remove_authenticator(Some(&revocation_code.to_string()))
            .map_err(steam_error)
    }

    pub fn is_transferring(&self) -> bool {
        self.transferring
    }

    pub fn take_account(&mut self) -> Option<SteamGuardAccount> {
        self.account.take()
    }
}
