use steam_core::error_code::ErrorCode;

use super::{save_or_restore, AppState};

impl AppState {
    pub fn create_folder(
        &self,
        name: &str,
        color: Option<&str>,
        icon: Option<&str>,
    ) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err(ErrorCode::FolderNameRequired.as_str().to_string());
        }
        let mut inner = self.lock();
        let previous = inner.settings.data().clone();
        let id = inner.settings.data_mut().create_folder(name);
        inner.settings.data_mut().set_folder_color(&id, color);
        inner.settings.data_mut().set_folder_icon(&id, icon);
        save_or_restore(&mut inner.settings, previous)
    }

    pub fn update_folder(
        &self,
        id: &str,
        name: &str,
        color: Option<&str>,
        icon: Option<&str>,
    ) -> Result<(), String> {
        if id == crate::retained_accounts::SAVED_GUARD_FOLDER_ID {
            return Err(ErrorCode::ProtectedFolder.as_str().to_string());
        }
        if name.trim().is_empty() {
            return Err(ErrorCode::FolderNameRequired.as_str().to_string());
        }
        let mut inner = self.lock();
        let previous = inner.settings.data().clone();
        if !inner.settings.data_mut().rename_folder(id, name) {
            return Err(ErrorCode::UnknownFolder.as_str().to_string());
        }
        inner.settings.data_mut().set_folder_color(id, color);
        inner.settings.data_mut().set_folder_icon(id, icon);
        save_or_restore(&mut inner.settings, previous)
    }

    pub fn delete_folder(&self, id: &str) -> Result<(), String> {
        if id == crate::retained_accounts::SAVED_GUARD_FOLDER_ID {
            return Err(ErrorCode::ProtectedFolder.as_str().to_string());
        }
        let mut inner = self.lock();
        let previous = inner.settings.data().clone();
        inner.settings.data_mut().delete_folder(id);
        save_or_restore(&mut inner.settings, previous)
    }

    pub fn set_account_folder(&self, steam_id: &str, folder: Option<&str>) -> Result<(), String> {
        let mut inner = self.lock();
        if folder == Some(crate::retained_accounts::SAVED_GUARD_FOLDER_ID)
            || !inner
                .accounts
                .iter()
                .any(|account| account.steam_id == steam_id)
        {
            return Err(ErrorCode::SavedAccountFolder.as_str().to_string());
        }
        let previous = inner.settings.data().clone();
        inner
            .settings
            .data_mut()
            .set_account_folder(steam_id, folder);
        save_or_restore(&mut inner.settings, previous)
    }

    pub fn set_account_color(&self, steam_id: &str, color: Option<&str>) -> Result<(), String> {
        let mut inner = self.lock();
        let previous = inner.settings.data().clone();
        inner.settings.data_mut().set_account_color(steam_id, color);
        save_or_restore(&mut inner.settings, previous)
    }

    pub fn set_account_note(&self, steam_id: &str, note: &str) -> Result<(), String> {
        // Same accounts as `display_accounts`, without rebuilding the list.
        let (configured, indexed) = {
            let guard = self
                .authenticator
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            (guard.configured_ids(), guard.status().indexed)
        };
        let mut inner = self.lock();
        let known = inner
            .accounts
            .iter()
            .any(|account| account.steam_id == steam_id)
            || configured.iter().any(|id| id == steam_id)
            || (!indexed
                && inner
                    .settings
                    .data()
                    .guard_account_profiles
                    .contains_key(steam_id));
        // Removing a note stays possible for an account that is no longer listed.
        if !known && !note.trim().is_empty() {
            return Err(ErrorCode::InvalidSteamId.as_str().to_string());
        }
        let previous = inner.settings.data().clone();
        inner
            .settings
            .data_mut()
            .set_account_note(steam_id, note)
            .map_err(|code| code.as_str().to_string())?;
        save_or_restore(&mut inner.settings, previous)
    }

    pub fn set_folder_color(&self, id: &str, color: Option<&str>) -> Result<(), String> {
        if id == crate::retained_accounts::SAVED_GUARD_FOLDER_ID {
            return Err(ErrorCode::ProtectedFolder.as_str().to_string());
        }
        let mut inner = self.lock();
        let previous = inner.settings.data().clone();
        if !inner.settings.data_mut().set_folder_color(id, color) {
            return Err(ErrorCode::UnknownFolder.as_str().to_string());
        }
        save_or_restore(&mut inner.settings, previous)
    }

    pub fn set_folder_icon(&self, id: &str, icon: Option<&str>) -> Result<(), String> {
        if id == crate::retained_accounts::SAVED_GUARD_FOLDER_ID {
            return Err(ErrorCode::ProtectedFolder.as_str().to_string());
        }
        let mut inner = self.lock();
        let previous = inner.settings.data().clone();
        if !inner.settings.data_mut().set_folder_icon(id, icon) {
            return Err(ErrorCode::UnknownFolder.as_str().to_string());
        }
        save_or_restore(&mut inner.settings, previous)
    }
}
