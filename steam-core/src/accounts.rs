use crate::error_code::ErrorCode;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::autologin;
use crate::error::{Result, SteamError};
use crate::model::{
    is_selected_account, parse_timestamp, parse_vdf_bool, resolve, SteamAccount, SteamInstallation,
};
use crate::processes::{ProcessControl, SystemProcesses};
use crate::util::{now_unix, write_atomic, WriteMode};
use crate::vdf::{self, VdfObject};

const ACCOUNT_KEYS: [&str; 5] = [
    "InstallConfigStore",
    "Software",
    "Valve",
    "Steam",
    "Accounts",
];

const AVATAR_SUFFIXES: [&str; 4] = [".png", ".jpg", ".jpeg", ".webp"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, ts_rs::TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SwitchOutcomeKind {
    Switched,
    AlreadyActive,
    Started,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SwitchOutcome {
    Switched { account_name: String },
    AlreadyActive { account_name: String },
    Started { account_name: String },
}

impl SwitchOutcome {
    pub fn already_active(&self) -> bool {
        matches!(self, SwitchOutcome::AlreadyActive { .. })
    }

    pub fn started(&self) -> bool {
        matches!(self, SwitchOutcome::Started { .. })
    }

    pub fn kind(&self) -> SwitchOutcomeKind {
        match self {
            SwitchOutcome::Switched { .. } => SwitchOutcomeKind::Switched,
            SwitchOutcome::AlreadyActive { .. } => SwitchOutcomeKind::AlreadyActive,
            SwitchOutcome::Started { .. } => SwitchOutcomeKind::Started,
        }
    }
}

#[derive(Clone)]
pub struct SteamManager {
    pub installation: SteamInstallation,
    processes: Arc<dyn ProcessControl>,
}

impl std::fmt::Debug for SteamManager {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SteamManager")
            .field("installation", &self.installation)
            .finish_non_exhaustive()
    }
}

impl SteamManager {
    pub fn new(installation: SteamInstallation) -> Self {
        Self {
            installation,
            processes: Arc::new(SystemProcesses),
        }
    }

    pub fn list_accounts(&self) -> Result<Vec<SteamAccount>> {
        let data = self.read_loginusers()?;
        let Some(users) = data.get_object("users") else {
            return Ok(Vec::new());
        };

        let mut accounts = Vec::new();
        for (steam_id, value) in users.iter() {
            let Some(info) = value.as_object() else {
                continue;
            };

            let account_name = info.get_str("AccountName").unwrap_or_default().to_string();
            let persona_name = info.get_str("PersonaName").unwrap_or_default().to_string();

            accounts.push(SteamAccount {
                steam_id: steam_id.to_string(),
                avatar_path: self.find_avatar_path(steam_id, &account_name),
                most_recent: read_bool(info, "MostRecent"),
                remember_password: read_bool(info, "RememberPassword"),
                allow_auto_login: read_bool(info, "AllowAutoLogin"),
                wants_offline_mode: read_bool(info, "WantsOfflineMode"),
                skip_offline_mode_warning: read_bool(info, "SkipOfflineModeWarning"),
                timestamp: info.get_str("Timestamp").and_then(parse_timestamp),
                account_name,
                persona_name,
            });
        }

        accounts.sort_by(|left, right| {
            right
                .timestamp
                .unwrap_or(0)
                .cmp(&left.timestamp.unwrap_or(0))
                .then_with(|| left.most_recent.cmp(&right.most_recent).reverse())
                .then_with(|| {
                    left.account_name
                        .to_lowercase()
                        .cmp(&right.account_name.to_lowercase())
                })
        });

        Ok(accounts)
    }

    pub fn find_by_steam_id(&self, steam_id: &str) -> Result<Option<SteamAccount>> {
        Ok(self
            .list_accounts()?
            .into_iter()
            .find(|account| account.steam_id == steam_id))
    }

    pub fn auto_login_user(&self) -> Result<String> {
        autologin::read_auto_login(&self.installation)
    }

    // ! `AutoLoginUser` and the `MostRecent` flag must agree, or a switch that terminated Steam mid-write would show a different answer depending which source is asked.
    pub fn selected_account(&self) -> Result<Option<SteamAccount>> {
        let auto_login_user = self.auto_login_user()?;
        if auto_login_user.is_empty() {
            return Ok(None);
        }
        Ok(self
            .list_accounts()?
            .into_iter()
            .find(|account| is_selected_account(account, &auto_login_user)))
    }

    // ! For playtime attribution only: prefers the `MostRecent` flag, then the newest `Timestamp`, never the switch target (see `selected_account`).
    pub fn most_recently_played_account(&self) -> Result<Option<SteamAccount>> {
        let accounts = self.list_accounts()?;
        if let Some(account) = accounts.iter().find(|account| account.most_recent) {
            return Ok(Some(account.clone()));
        }
        Ok(accounts
            .into_iter()
            .max_by_key(|account| account.timestamp.unwrap_or(0)))
    }

    pub fn set_auto_login_user(&self, account_name: &str) -> Result<()> {
        autologin::write_auto_login(&self.installation, account_name)
    }

    // ! Writes the auto-login account and reads it back: a write that silently does not stick would otherwise surface as Steam logging into the wrong account.
    pub fn set_auto_login_user_verified(&self, account_name: &str) -> Result<()> {
        self.set_auto_login_user(account_name)?;
        let stored = self.auto_login_user()?;
        if stored != account_name {
            return Err(SteamError::config(ErrorCode::SteamRegistryFailed));
        }
        Ok(())
    }

    pub fn find_avatar_path(&self, steam_id: &str, account_name: &str) -> Option<PathBuf> {
        let directories = [
            self.installation.config_dir.join("avatarcache"),
            self.installation.root.join("config/avatarcache"),
            self.installation.root.join("appcache/avatarcache"),
        ];

        let mut names = vec![steam_id.to_string()];
        // ! The login name comes from a file Steam rewrites: never join it as a path unless it is a plain file name.
        if is_plain_file_name(account_name) {
            names.push(account_name.to_string());
        }
        for suffix in ["_full", "_medium", "_icon"] {
            names.push(format!("{steam_id}{suffix}"));
        }

        for directory in &directories {
            for name in &names {
                for suffix in AVATAR_SUFFIXES {
                    let candidate = directory.join(format!("{name}{suffix}"));
                    if candidate.is_file() {
                        return Some(candidate);
                    }
                }
            }
        }

        let wanted: Vec<String> = names.iter().map(|name| name.to_lowercase()).collect();
        let mut fallback: Option<PathBuf> = None;
        for directory in &directories {
            let Ok(entries) = fs::read_dir(directory) else {
                continue;
            };

            for entry in entries.flatten().take(1000) {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let extension = path
                    .extension()
                    .map(|value| format!(".{}", value.to_string_lossy().to_lowercase()))
                    .unwrap_or_default();
                if !AVATAR_SUFFIXES.contains(&extension.as_str()) {
                    continue;
                }

                let stem = path
                    .file_stem()
                    .map(|value| value.to_string_lossy().to_lowercase())
                    .unwrap_or_default();

                if wanted.iter().any(|candidate| candidate == &stem) {
                    return Some(path);
                }
                if !steam_id.is_empty()
                    && stem.contains(&steam_id.to_lowercase())
                    && fallback.is_none()
                {
                    fallback = Some(path);
                }
            }
        }

        fallback
    }

    pub fn switch_account(&self, steam_id: &str) -> Result<SwitchOutcome> {
        // ! Writing an unknown identifier would put a name Steam never saw into AutoLoginUser, so an unknown account is refused instead.
        let account = self
            .find_by_steam_id(steam_id)?
            .ok_or_else(|| SteamError::config(ErrorCode::AccountUnavailable))?;
        let account_name = account.account_name.clone();

        if account_name.is_empty() {
            return Err(SteamError::config(ErrorCode::AccountUnavailable));
        }

        let running = self.processes.is_running(&self.installation);
        let is_selected = self
            .selected_account()?
            .is_some_and(|selected| selected.steam_id == account.steam_id);

        if is_selected && running {
            return Ok(SwitchOutcome::AlreadyActive { account_name });
        }

        if running {
            self.processes
                .terminate(&self.installation, std::time::Duration::from_secs(30))?;
        }

        self.set_auto_login_user_verified(&account_name)?;
        self.prepare_loginusers_auto_login(&account.steam_id)?;
        self.processes.launch(&self.installation)?;

        Ok(if is_selected {
            SwitchOutcome::Started { account_name }
        } else {
            SwitchOutcome::Switched { account_name }
        })
    }

    // ! Opens Steam at its login window so the user can sign in: no credential ever reaches this application.
    pub fn add_account(&self) -> Result<()> {
        if self.processes.is_running(&self.installation) {
            self.processes
                .terminate(&self.installation, std::time::Duration::from_secs(30))?;
        }
        self.set_auto_login_user("")?;
        self.clear_loginusers_auto_login()?;
        self.processes.launch(&self.installation)
    }

    pub fn clear_loginusers_auto_login(&self) -> Result<()> {
        let mut data = self.read_loginusers()?;
        let Some(users) = data.get_object_mut("users") else {
            return Ok(());
        };

        for steam_id in users.keys().map(str::to_string).collect::<Vec<_>>() {
            let Some(info) = users.get_object_mut(&steam_id) else {
                continue;
            };
            info.set_string("MostRecent", "0");
            info.set_string("AllowAutoLogin", "0");
        }

        self.write_loginusers(&data)
    }

    pub fn prepare_loginusers_auto_login(&self, steam_id: &str) -> Result<()> {
        let mut data = self.read_loginusers()?;
        let Some(users) = data.get_object_mut("users") else {
            return Ok(());
        };

        let timestamp = now_unix() as i64;
        let ids: Vec<String> = users.keys().map(str::to_string).collect();

        for candidate in ids {
            let Some(info) = users.get_object_mut(&candidate) else {
                continue;
            };
            let is_target = candidate == steam_id;
            info.set_string("MostRecent", if is_target { "1" } else { "0" });
            info.set_string("AllowAutoLogin", if is_target { "1" } else { "0" });
            if is_target {
                info.set_string("RememberPassword", "1");
                info.set_string("WantsOfflineMode", "0");
                info.set_string("SkipOfflineModeWarning", "0");
                info.set_string("Timestamp", timestamp.to_string());
            }
        }

        self.write_loginusers(&data)
    }

    pub fn remove_account(&self, steam_id: &str, cleanup_userdata: bool) -> Result<()> {
        let account = self
            .find_by_steam_id(steam_id)?
            .ok_or_else(|| SteamError::config(ErrorCode::AccountUnavailable))?;

        // ! Steam keeps loginusers.vdf in memory and rewrites it on exit, so editing it while the client runs silently resurrects the entry.
        if self.processes.is_running(&self.installation) {
            self.processes
                .terminate(&self.installation, std::time::Duration::from_secs(30))?;
        }

        let mut data = self.read_loginusers()?;
        if let Some(users) = data.get_object_mut("users") {
            users.remove(&account.steam_id);
        }
        self.write_loginusers(&data)?;

        self.remove_config_account(&account.account_name)?;

        if self
            .auto_login_user()?
            .eq_ignore_ascii_case(&account.account_name)
        {
            self.set_auto_login_user("")?;
        }

        if cleanup_userdata {
            self.remove_userdata(&account.steam_id)?;
        }

        Ok(())
    }

    pub fn remove_userdata(&self, steam_id: &str) -> Result<()> {
        // ! Userdata folders use the low 32-bit account ID while `loginusers.vdf` keys hold the 64-bit Steam ID; an ID that does not convert, or resolves outside userdata, is refused before this deletes a directory tree.
        const INDIVIDUAL_BASE: u64 = 76_561_197_960_265_728;
        let account_id = steam_id
            .parse::<u64>()
            .ok()
            .filter(|_| steam_id.bytes().all(|byte| byte.is_ascii_digit()))
            .and_then(|id| id.checked_sub(INDIVIDUAL_BASE))
            .and_then(|id| u32::try_from(id).ok())
            .filter(|id| *id != 0)
            .ok_or_else(|| SteamError::config(ErrorCode::UnsafeDataRemoval))?;

        let userdata_root = resolve(&self.installation.userdata_dir);
        let target = resolve(&userdata_root.join(account_id.to_string()));
        if !target.starts_with(&userdata_root) || target == userdata_root {
            return Err(SteamError::config(ErrorCode::UnsafeDataRemoval));
        }

        if target.exists() {
            fs::remove_dir_all(&target).map_err(|error| {
                SteamError::io(format!("Unable to delete {}", target.display()), error)
            })?;
        }

        Ok(())
    }

    pub fn is_steam_running(&self) -> bool {
        self.processes.is_running(&self.installation)
    }

    fn read_loginusers(&self) -> Result<VdfObject> {
        read_vdf(&self.installation.loginusers_path)
    }

    fn write_loginusers(&self, data: &VdfObject) -> Result<()> {
        write_vdf(&self.installation.loginusers_path, data)
    }

    fn remove_config_account(&self, account_name: &str) -> Result<()> {
        let path = self.installation.config_dir.join("config.vdf");
        if !path.is_file() {
            return Ok(());
        }

        let mut data = read_vdf(&path)?;
        let Some(accounts) = data.path_mut(&ACCOUNT_KEYS, false) else {
            return Ok(());
        };

        // ! Steam nests cached sign-in data (SSFN hash, refresh token) under this same `Accounts\<name>` key, so removing the subtree already clears it.
        if accounts.remove(account_name).is_some() {
            write_vdf(&path, &data)?;
        }

        Ok(())
    }
}

fn read_bool(object: &VdfObject, key: &str) -> bool {
    object.get_str(key).map(parse_vdf_bool).unwrap_or(false)
}

pub fn read_vdf(path: &Path) -> Result<VdfObject> {
    if !path.exists() {
        return Ok(VdfObject::new());
    }

    let text = fs::read_to_string(path)
        .map_err(|error| SteamError::io(format!("Unable to read {}", path.display()), error))?;

    vdf::parse(&text).map_err(|error| SteamError::Vdf {
        path: path.display().to_string(),
        message: error.message,
    })
}

pub fn write_vdf(path: &Path, data: &VdfObject) -> Result<()> {
    let text = vdf::dump_verified(data).map_err(|error| SteamError::Vdf {
        path: path.display().to_string(),
        message: error.message,
    })?;
    write_atomic(path, &text, WriteMode::PreserveExisting)
        .map_err(|error| SteamError::io(format!("Unable to write {}", path.display()), error))
}

fn is_plain_file_name(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains(['/', '\\', '\0'])
}
