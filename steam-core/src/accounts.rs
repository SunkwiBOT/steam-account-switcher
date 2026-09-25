//! Reading, switching and removing the accounts Steam remembers locally.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::autologin;
use crate::error::{Result, SteamError};
use crate::model::{parse_timestamp, parse_vdf_bool, resolve, SteamAccount, SteamInstallation};
use crate::processes::{ProcessControl, SystemProcesses};
use crate::util::{now_unix, write_atomic};
use crate::vdf::{self, VdfObject};

const ACCOUNT_KEYS: [&str; 5] = [
    "InstallConfigStore",
    "Software",
    "Valve",
    "Steam",
    "Accounts",
];

const AVATAR_SUFFIXES: [&str; 4] = [".png", ".jpg", ".jpeg", ".webp"];

/// What a switch request actually did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SwitchOutcome {
    /// Steam was restarted with the requested account selected.
    Switched { account_name: String },
    /// The requested account was already the live session; nothing changed.
    AlreadyActive { account_name: String },
    /// The right account was already selected and Steam was closed, so the
    /// client was simply started.
    Started { account_name: String },
}

impl SwitchOutcome {
    pub fn account_name(&self) -> &str {
        match self {
            SwitchOutcome::Switched { account_name }
            | SwitchOutcome::AlreadyActive { account_name }
            | SwitchOutcome::Started { account_name } => account_name,
        }
    }

    pub fn already_active(&self) -> bool {
        matches!(self, SwitchOutcome::AlreadyActive { .. })
    }

    pub fn started(&self) -> bool {
        matches!(self, SwitchOutcome::Started { .. })
    }
}

/// Owns every operation that touches a Steam installation.
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
    /// Manager that really starts and stops the local Steam client.
    pub fn new(installation: SteamInstallation) -> Self {
        Self {
            installation,
            processes: Arc::new(SystemProcesses),
        }
    }

    /// Manager with an injected process controller; tests use an inert one so a
    /// unit test can never stop the Steam client of the machine running it.
    pub fn with_processes(
        installation: SteamInstallation,
        processes: Arc<dyn ProcessControl>,
    ) -> Self {
        Self {
            installation,
            processes,
        }
    }

    /// Every account in `loginusers.vdf`, most recently used first.
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

    pub fn find_account(&self, identifier: &str) -> Result<Option<SteamAccount>> {
        let lowered = identifier.to_lowercase();
        Ok(self.list_accounts()?.into_iter().find(|account| {
            account.steam_id == identifier
                || account.account_name.to_lowercase() == lowered
                || account.persona_name.to_lowercase() == lowered
        }))
    }

    pub fn auto_login_user(&self) -> Result<String> {
        autologin::read_auto_login(&self.installation)
    }

    /// The account Steam is really logged into: `loginusers.vdf` is written by
    /// Steam itself, the auto-login value is only this application's last write.
    pub fn current_account(&self) -> Result<Option<SteamAccount>> {
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

    /// Writes the auto-login account and reads it back.
    ///
    /// A write that silently does not stick (immutable file, redirected folder)
    /// would otherwise surface as "Steam logged into the wrong account".
    pub fn set_auto_login_user_verified(&self, account_name: &str) -> Result<()> {
        self.set_auto_login_user(account_name)?;
        let stored = self.auto_login_user()?;
        if stored != account_name {
            return Err(SteamError::config(format!(
                "Steam kept the previous auto-login account ({stored:?} instead of {account_name:?}). \
The registry file may be read-only or the Steam client is rewriting it."
            )));
        }
        Ok(())
    }

    /// Cached Steam avatar, when the client stored one on this machine.
    /// Checks the known file names first, then scans the local caches.
    pub fn find_avatar_path(&self, steam_id: &str, account_name: &str) -> Option<PathBuf> {
        let directories = [
            self.installation.config_dir.join("avatarcache"),
            self.installation.root.join("config/avatarcache"),
            self.installation.root.join("appcache/avatarcache"),
        ];

        let mut names = vec![steam_id.to_string()];
        if !account_name.is_empty() {
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

    /// Points Steam at another account, optionally restarting the client.
    pub fn switch_account(&self, identifier: &str, restart: bool) -> Result<SwitchOutcome> {
        let account = self.find_account(identifier)?;
        let account_name = account
            .as_ref()
            .map(|account| account.account_name.clone())
            .unwrap_or_else(|| identifier.to_string());

        if account_name.is_empty() {
            return Err(SteamError::config(
                "Cannot switch to an empty account name.",
            ));
        }

        // Steam is single instance, so a switch goes through a restart. Two
        // shortcuts keep that from doing pointless work.
        if restart {
            let running = self.processes.is_running();
            let is_target = self
                .current_account()?
                .map(|current| current.account_name.eq_ignore_ascii_case(&account_name))
                .unwrap_or(false);

            if is_target {
                if running {
                    // Restarting would only cost the user their running game.
                    return Ok(SwitchOutcome::AlreadyActive { account_name });
                }
                // The right account is already selected and Steam is closed:
                // starting the client is all that is left to do.
                self.processes.launch(&self.installation)?;
                return Ok(SwitchOutcome::Started { account_name });
            }
        }

        if restart {
            self.processes
                .terminate(&self.installation, std::time::Duration::from_secs(30))?;
        }

        self.set_auto_login_user_verified(&account_name)?;
        if let Some(account) = &account {
            self.prepare_loginusers_auto_login(&account.steam_id)?;
        }

        if restart {
            self.processes.launch(&self.installation)?;
        }

        Ok(SwitchOutcome::Switched { account_name })
    }

    /// Opens Steam at its login window so the user can sign in: no credential
    /// ever reaches this application.
    pub fn add_account(&self) -> Result<()> {
        if self.processes.is_running() {
            self.processes
                .terminate(&self.installation, std::time::Duration::from_secs(30))?;
        }
        self.set_auto_login_user("")?;
        self.clear_loginusers_auto_login()?;
        self.processes.launch(&self.installation)
    }

    /// Clears `MostRecent`/`AllowAutoLogin` so Steam shows its login window.
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

    /// Flags the target account so Steam auto-logs into it on the next start.
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

    /// Removes an account from Steam's login metadata.
    pub fn remove_account(&self, identifier: &str, cleanup_userdata: bool) -> Result<()> {
        let account = self
            .find_account(identifier)?
            .ok_or_else(|| SteamError::config(format!("Unknown Steam account: {identifier}")))?;

        // Steam keeps loginusers.vdf in memory and rewrites it when it exits,
        // so editing the file while the client runs silently resurrects the
        // entry. Stop Steam first.
        if self.processes.is_running() {
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

    /// Deletes `userdata/<account-id>` for a public individual SteamID64.
    pub fn remove_userdata(&self, steam_id: &str) -> Result<()> {
        // Steam's userdata directories use the low 32-bit account ID, while
        // loginusers.vdf keys contain the full 64-bit Steam ID.
        const INDIVIDUAL_BASE: u64 = 76_561_197_960_265_728;
        let account_id = steam_id
            .parse::<u64>()
            .ok()
            .filter(|_| steam_id.bytes().all(|byte| byte.is_ascii_digit()))
            .and_then(|id| id.checked_sub(INDIVIDUAL_BASE))
            .and_then(|id| u32::try_from(id).ok())
            .filter(|id| *id != 0)
            .ok_or_else(|| {
                SteamError::config(format!(
                    "Refusing to delete userdata for invalid steamid: {steam_id}"
                ))
            })?;

        let userdata_root = resolve(&self.installation.userdata_dir);
        let target = resolve(&userdata_root.join(account_id.to_string()));
        if !target.starts_with(&userdata_root) || target == userdata_root {
            return Err(SteamError::config(format!(
                "Refusing to delete path outside userdata: {}",
                target.display()
            )));
        }

        if target.exists() {
            fs::remove_dir_all(&target).map_err(|error| {
                SteamError::io(format!("Unable to delete {}", target.display()), error)
            })?;
        }

        Ok(())
    }

    pub fn restart_steam(&self) -> Result<()> {
        self.processes
            .terminate(&self.installation, std::time::Duration::from_secs(30))?;
        self.processes.launch(&self.installation)
    }

    pub fn is_steam_running(&self) -> bool {
        self.processes.is_running()
    }

    /// Path of the local playtime database for this installation.
    pub fn playtime_state_path(&self) -> PathBuf {
        self.installation
            .config_dir
            .join("steam-account-switcher-playtime.json")
    }

    /// Path of the local rank metadata for this installation.
    pub fn metadata_path(&self) -> PathBuf {
        self.installation
            .config_dir
            .join("steam-account-switcher-metadata.json")
    }

    /// Path of Steam's game process log used for playtime tracking.
    pub fn gameprocess_log_path(&self) -> PathBuf {
        self.installation.root.join("logs/gameprocess_log.txt")
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
    write_atomic(path, &vdf::dump(data))
        .map_err(|error| SteamError::io(format!("Unable to write {}", path.display()), error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::InstallKind;

    fn seed_installation(directory: &Path) -> SteamManager {
        let config_dir = directory.join("config");
        fs::create_dir_all(&config_dir).expect("config dir");
        fs::create_dir_all(directory.join("userdata/39734273")).expect("first userdata");
        fs::create_dir_all(directory.join("userdata/39734274")).expect("second userdata");

        fs::write(
            config_dir.join("loginusers.vdf"),
            r#"
"users"
{
	"76561198000000001"
	{
		"AccountName"		"first-account"
		"PersonaName"		"First"
		"MostRecent"		"1"
		"AllowAutoLogin"		"1"
		"RememberPassword"		"1"
		"Timestamp"		"1700000000"
	}
	"76561198000000002"
	{
		"AccountName"		"second-account"
		"PersonaName"		"Second"
		"MostRecent"		"0"
		"Timestamp"		"1600000000"
	}
}
"#,
        )
        .expect("loginusers");

        fs::write(
            config_dir.join("config.vdf"),
            r#"
"InstallConfigStore"
{
	"Software"
	{
		"Valve"
		{
			"Steam"
			{
				"Accounts"
				{
					"first-account"
					{
						"SteamID"		"76561198000000001"
					}
				}
			}
		}
	}
}
"#,
        )
        .expect("config.vdf");

        fs::write(
            directory.join("registry.vdf"),
            "\"Registry\"\n{\n\t\"HKCU\"\n\t{\n\t\t\"Software\"\n\t\t{\n\t\t\t\"Valve\"\n\t\t\t{\n\t\t\t\t\"Steam\"\n\t\t\t\t{\n\t\t\t\t\t\"AutoLoginUser\"\t\t\"first-account\"\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t}\n}\n",
        )
        .expect("registry.vdf");

        SteamManager::with_processes(
            SteamInstallation {
                kind: InstallKind::Native,
                root: directory.to_path_buf(),
                config_dir: config_dir.clone(),
                loginusers_path: config_dir.join("loginusers.vdf"),
                registry_path: directory.join("registry.vdf"),
                userdata_dir: directory.join("userdata"),
                launch_command: vec!["steam".to_string()],
                prefix: None,
            },
            Arc::new(crate::processes::NoProcesses),
        )
    }

    fn temp_installation(name: &str) -> (PathBuf, SteamManager) {
        let directory =
            std::env::temp_dir().join(format!("steam-core-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("temp dir");
        let manager = seed_installation(&directory);
        (directory, manager)
    }

    #[test]
    fn lists_accounts_most_recent_first() {
        let (directory, manager) = temp_installation("list");
        let accounts = manager.list_accounts().expect("accounts");

        assert_eq!(accounts.len(), 2);
        assert_eq!(accounts[0].account_name, "first-account");
        assert_eq!(accounts[0].display_name(), "First");
        assert!(accounts[0].most_recent);
        assert!(!accounts[1].most_recent);
        assert_eq!(accounts[0].avatar_initial(), "F");

        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn switch_marks_the_target_account_without_restarting_steam() {
        let (directory, manager) = temp_installation("switch");
        let outcome = manager
            .switch_account("second-account", false)
            .expect("switch");
        assert_eq!(outcome.account_name(), "second-account");
        assert!(!outcome.already_active());
        assert_eq!(
            manager
                .current_account()
                .expect("current")
                .map(|a| a.account_name),
            Some("second-account".to_string())
        );
        assert_eq!(
            manager.auto_login_user().expect("auto login"),
            "second-account"
        );

        let accounts = manager.list_accounts().expect("accounts");
        let second = accounts
            .iter()
            .find(|account| account.account_name == "second-account")
            .expect("second account");
        assert!(second.most_recent);
        assert!(second.allow_auto_login);
        assert!(second.remember_password);
        assert!(second.timestamp.unwrap_or(0) > 1_700_000_000);

        let first = accounts
            .iter()
            .find(|account| account.account_name == "first-account")
            .expect("first account");
        assert!(!first.most_recent);

        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn removing_an_account_clears_login_metadata_and_userdata() {
        let (directory, manager) = temp_installation("remove");

        manager
            .remove_account("first-account", true)
            .expect("remove account");

        let accounts = manager.list_accounts().expect("accounts");
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account_name, "second-account");
        assert!(!directory.join("userdata/39734273").exists());
        assert!(directory.join("userdata/39734274").exists());

        let config = read_vdf(&directory.join("config/config.vdf")).expect("config.vdf");
        let accounts_section = config.path(&ACCOUNT_KEYS).expect("accounts section");
        assert!(accounts_section.get("first-account").is_none());

        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn userdata_removal_refuses_to_escape_the_userdata_directory() {
        let (directory, manager) = temp_installation("guard");
        assert!(manager.remove_userdata("../outside").is_err());
        assert!(manager.remove_userdata("").is_err());
        assert!(manager.remove_userdata("39734273").is_err());
        assert!(manager.remove_userdata("76561197960265728").is_err());
        assert!(manager.remove_userdata("18446744073709551615").is_err());
        assert!(directory.join("userdata/39734273").exists());
        let _ = fs::remove_dir_all(directory);
    }

    #[cfg(unix)]
    #[test]
    fn userdata_removal_refuses_symlinks_outside_userdata() {
        let (directory, manager) = temp_installation("userdata-symlink");
        let outside = directory.join("outside");
        fs::create_dir(&outside).expect("outside directory");
        fs::write(outside.join("save.dat"), "keep").expect("save file");
        std::os::unix::fs::symlink(&outside, directory.join("userdata/39734275")).expect("symlink");
        assert!(manager.remove_userdata("76561198000000003").is_err());
        assert!(outside.join("save.dat").exists());
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn adding_an_account_clears_the_auto_login_selection() {
        let (directory, manager) = temp_installation("add");
        assert_eq!(
            manager.auto_login_user().expect("auto login"),
            "first-account"
        );

        // `add_account` also relaunches Steam; only the file/registry side is
        // exercised here so the test never starts a real client.
        manager.set_auto_login_user("").expect("clear auto login");
        manager
            .clear_loginusers_auto_login()
            .expect("clear login flags");

        assert_eq!(manager.auto_login_user().expect("auto login"), "");
        let accounts = manager.list_accounts().expect("accounts");
        assert!(accounts.iter().all(|account| !account.most_recent));
        assert!(accounts.iter().all(|account| !account.allow_auto_login));

        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn avatar_lookup_finds_the_cached_file() {
        let (directory, manager) = temp_installation("avatar");
        let cache = directory.join("config/avatarcache");
        fs::create_dir_all(&cache).expect("avatar cache");
        fs::write(cache.join("76561198000000001.png"), b"fake").expect("avatar");

        let accounts = manager.list_accounts().expect("accounts");
        let first = accounts
            .iter()
            .find(|account| account.steam_id == "76561198000000001")
            .expect("first account");
        assert_eq!(
            first.avatar_path.as_deref(),
            Some(cache.join("76561198000000001.png").as_path())
        );

        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn accounts_without_cached_avatar_still_have_an_initial() {
        let account = SteamAccount {
            steam_id: "76561198000000003".to_string(),
            account_name: String::new(),
            persona_name: String::new(),
            avatar_path: None,
            most_recent: false,
            remember_password: false,
            allow_auto_login: false,
            wants_offline_mode: false,
            skip_offline_mode_warning: false,
            timestamp: None,
        };
        assert_eq!(account.display_name(), "76561198000000003");
        assert_eq!(account.avatar_initial(), "7");
    }
}
