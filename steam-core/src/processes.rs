//! Steam process supervision: detect, stop and start the client.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

use sysinfo::{Pid, ProcessRefreshKind, ProcessStatus, ProcessesToUpdate, System};

use crate::error::{Result, SteamError};
use crate::model::SteamInstallation;

/// Client processes that mean "Steam is running".
#[cfg(windows)]
pub const STEAM_PROCESS_NAMES: &[&str] = &["steam.exe", "steamwebhelper.exe"];

#[cfg(not(windows))]
pub const STEAM_PROCESS_NAMES: &[&str] = &[
    "steam",
    "steam.sh",
    "steamwebhelper",
    "steam-runtime-supervisor",
    "steam-runtime-launcher",
    "steam-runtime-check-requirements",
];

fn is_steam_process(name: &str) -> bool {
    let lowered = name.to_ascii_lowercase();
    STEAM_PROCESS_NAMES.iter().any(|candidate| {
        lowered == candidate.to_ascii_lowercase() || lowered == format!("{candidate}.exe")
    })
}

/// `steam -shutdown` runs as a `steam` process of its own, and stays alive
/// after the client is gone. It is a request, not a client.
fn is_shutdown_request(process: &sysinfo::Process) -> bool {
    process.cmd().iter().any(|argument| argument == "-shutdown")
}

/// How long the client may take to show up after a launch request.
const LAUNCH_ATTEMPT_TIMEOUT: Duration = Duration::from_secs(8);
/// The launcher forwards its arguments to a client that is still shutting
/// down, so a launch can be swallowed. Retrying is what makes it reliable.
const LAUNCH_ATTEMPTS: usize = 4;
/// Extra pause once no client process is left, so the next launch is not
/// forwarded to a client that is only halfway gone.
const SHUTDOWN_SETTLE: Duration = Duration::from_millis(1500);
/// Pause between two launch attempts.
const LAUNCH_RETRY_GAP: Duration = Duration::from_millis(1500);
/// A shutdown request is not a running client; give it a moment, then go on.
const SHUTDOWN_HELPER_GRACE: Duration = Duration::from_secs(10);

/// Facts the playtime tracker needs to confirm a tracked game process is alive.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProcessFacts {
    pub running: bool,
    pub start_time: Option<f64>,
}

/// Caches the process table so repeated polls stay cheap.
#[derive(Debug, Default)]
pub struct ProcessMonitor {
    system: System,
}

impl ProcessMonitor {
    pub fn new() -> Self {
        Self {
            system: System::new(),
        }
    }

    pub fn refresh(&mut self) {
        self.system.refresh_processes(ProcessesToUpdate::All, true);
    }

    pub fn steam_processes(&self) -> Vec<(u32, String)> {
        let own_pid = std::process::id();
        self.system
            .processes()
            .iter()
            .filter(|(pid, _)| pid.as_u32() != own_pid)
            .filter(|(_, process)| process.status() != ProcessStatus::Zombie)
            .map(|(pid, process)| (pid.as_u32(), process.name().to_string_lossy().to_string()))
            .filter(|(_, name)| is_steam_process(name))
            .collect()
    }

    /// Running Steam clients, excluding the helper that performs a shutdown.
    pub fn steam_client_processes(&self) -> Vec<(u32, String)> {
        let own_pid = std::process::id();
        self.system
            .processes()
            .iter()
            .filter(|(pid, _)| pid.as_u32() != own_pid)
            .filter(|(_, process)| process.status() != ProcessStatus::Zombie)
            .filter(|(_, process)| !is_shutdown_request(process))
            .map(|(pid, process)| (pid.as_u32(), process.name().to_string_lossy().to_string()))
            .filter(|(_, name)| is_steam_process(name))
            .collect()
    }

    pub fn is_steam_running(&self) -> bool {
        !self.steam_client_processes().is_empty()
    }

    /// Verifies a game process id, returning whether it runs and when it started.
    pub fn process_facts(&mut self, pid: u32) -> ProcessFacts {
        let target = Pid::from_u32(pid);
        if self.system.process(target).is_none() {
            self.system.refresh_processes_specifics(
                ProcessesToUpdate::Some(&[target]),
                true,
                ProcessRefreshKind::nothing(),
            );
        }

        match self.system.process(target) {
            Some(process) => ProcessFacts {
                running: true,
                start_time: Some(process.start_time() as f64),
            },
            None => ProcessFacts {
                running: false,
                start_time: None,
            },
        }
    }
}

/// Asks Steam to close, then makes sure it is gone before returning.
///
/// The shutdown request uses the same command line Steam was launched with, so
/// Flatpak and Snap installations are handled without special casing.
pub fn terminate_steam(installation: &SteamInstallation, timeout: Duration) -> Result<()> {
    let mut monitor = ProcessMonitor::new();
    monitor.refresh();
    if !monitor.is_steam_running() {
        wait_for_shutdown_helpers(&mut monitor, SHUTDOWN_HELPER_GRACE);
        return Ok(());
    }

    let mut shutdown = request_shutdown_process(installation);

    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        reap(&mut shutdown);
        monitor.refresh();
        if !monitor.is_steam_running() {
            wait_for_shutdown_helpers(&mut monitor, SHUTDOWN_HELPER_GRACE);
            sleep(SHUTDOWN_SETTLE);
            return Ok(());
        }
        sleep(Duration::from_millis(250));
    }

    monitor.refresh();
    for (pid, _) in monitor.steam_client_processes() {
        if let Some(process) = monitor.system.process(Pid::from_u32(pid)) {
            let _ = process.kill();
        }
    }

    let force_deadline = Instant::now() + Duration::from_secs(8);
    while Instant::now() < force_deadline {
        reap(&mut shutdown);
        monitor.refresh();
        if !monitor.is_steam_running() {
            wait_for_shutdown_helpers(&mut monitor, SHUTDOWN_HELPER_GRACE);
            sleep(SHUTDOWN_SETTLE);
            return Ok(());
        }
        sleep(Duration::from_millis(250));
    }

    Err(SteamError::config(
        "Steam did not stop; refusing to launch a second instance.",
    ))
}

/// Collects the process of a shutdown request, so it does not stay behind as a
/// zombie that keeps looking like a running client.
fn reap(shutdown: &mut Option<Child>) {
    if let Some(child) = shutdown.as_mut() {
        if matches!(child.try_wait(), Ok(Some(_)) | Err(_)) {
            *shutdown = None;
        }
    }
}

/// Starting Steam while a shutdown request is still running only feeds that
/// dying helper, so give it a moment before the launch is attempted.
fn wait_for_shutdown_helpers(monitor: &mut ProcessMonitor, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    loop {
        monitor.refresh();
        if monitor.steam_processes().is_empty() {
            return;
        }
        if Instant::now() >= deadline {
            return;
        }
        sleep(Duration::from_millis(200));
    }
}

#[cfg(unix)]
fn detach(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    // Own process group: Steam keeps running when the switcher exits.
    command.process_group(0);
}

#[cfg(windows)]
fn detach(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    command.creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS);
}

/// Variables the AppImage launcher exports for its own bundled GTK stack.
///
/// A process hands its environment to everything it starts, so a Steam client
/// started while this application runs from an AppImage would give its games
/// paths inside `/tmp/.mount_*`. The Steam Linux Runtime then fails to start
/// (the AppImage's libraries and GIO modules break the host Python its launch
/// scripts run on), and only a Steam started from the desktop menu works.
const APPIMAGE_VARIABLES: [&str; 19] = [
    "APPDIR",
    "APPIMAGE",
    "ARGV0",
    "OWD",
    "GDK_BACKEND",
    "GDK_PIXBUF_MODULE_FILE",
    "GDK_PIXBUF_MODULEDIR",
    "GI_TYPELIB_PATH",
    "GIO_EXTRA_MODULES",
    "GIO_MODULE_DIR",
    "GSETTINGS_SCHEMA_DIR",
    "GTK_DATA_PREFIX",
    "GTK_EXE_PREFIX",
    "GTK_IM_MODULE_FILE",
    "GTK_PATH",
    "GTK_THEME",
    "PERLLIB",
    "PYTHONHOME",
    "PYTHONPATH",
];

/// Path list variables the AppImage prepends its own mount point to.
const APPIMAGE_PATH_VARIABLES: [&str; 3] = ["LD_LIBRARY_PATH", "PATH", "XDG_DATA_DIRS"];

/// True when this application was started from an AppImage.
fn running_from_appimage() -> bool {
    std::env::var_os("APPDIR").is_some() || std::env::var_os("APPIMAGE").is_some()
}

/// Directory the AppImage is mounted in, when the launcher tells us.
fn appimage_mount() -> Option<PathBuf> {
    std::env::var_os("APPDIR")
        .or_else(|| std::env::var_os("APPIMAGE"))
        .map(PathBuf::from)
}

/// Whether one entry of a path list points inside the AppImage.
///
/// The mount point carries a random suffix, so a stale `APPDIR` is not enough:
/// anything under `/tmp/.mount_` is an AppImage path as well.
fn belongs_to_appimage(entry: &Path, mount: Option<&Path>) -> bool {
    entry.to_string_lossy().starts_with("/tmp/.mount_")
        || mount.is_some_and(|mount| entry.starts_with(mount))
}

/// Entries of a path list that do not belong to the AppImage.
fn without_appimage_paths(value: &OsStr, mount: Option<&Path>) -> Vec<PathBuf> {
    std::env::split_paths(value)
        .filter(|entry| !belongs_to_appimage(entry, mount))
        .collect()
}

/// Removes the AppImage launcher environment from a command.
///
/// Only an AppImage needs this: a plain build runs with the user's own
/// environment, which is exactly what a menu launch gives Steam.
fn strip_appimage_environment(command: &mut Command) {
    if !running_from_appimage() {
        return;
    }

    let mount = appimage_mount();
    for name in APPIMAGE_VARIABLES {
        command.env_remove(name);
    }

    for name in APPIMAGE_PATH_VARIABLES {
        let Some(value) = std::env::var_os(name) else {
            continue;
        };

        let kept = without_appimage_paths(&value, mount.as_deref());
        if kept.is_empty() {
            command.env_remove(name);
        } else if let Ok(filtered) = std::env::join_paths(&kept) {
            command.env(name, filtered);
        }
    }
}

fn run_detached(command: &[String]) -> Result<()> {
    run_detached_process(command).map(|_| ())
}

fn run_detached_process(command: &[String]) -> Result<Child> {
    let (program, arguments) = command
        .split_first()
        .ok_or_else(|| SteamError::config("No Steam launch command is configured."))?;

    let mut process = Command::new(program);
    process
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    detach(&mut process);
    strip_appimage_environment(&mut process);

    process
        .spawn()
        .map_err(|error| SteamError::io(format!("Unable to launch Steam ({program})"), error))
}

/// Starts Steam and waits until the client really runs.
///
/// The `steam` launcher hands its command line to an already running client
/// and exits, so a launch issued while the previous client is still closing
/// silently does nothing. Retrying covers that window instead of leaving the
/// user in front of a switcher that looks stuck.
pub fn launch_steam_verified(installation: &SteamInstallation) -> Result<()> {
    if installation.launch_command.is_empty() {
        return Err(SteamError::config("No Steam launch command is configured."));
    }

    let mut monitor = ProcessMonitor::new();
    for attempt in 1..=LAUNCH_ATTEMPTS {
        run_detached(&installation.launch_command)?;
        if wait_for_client(&mut monitor, LAUNCH_ATTEMPT_TIMEOUT) {
            return Ok(());
        }
        if attempt < LAUNCH_ATTEMPTS {
            sleep(LAUNCH_RETRY_GAP);
        }
    }

    Err(SteamError::config(
        "Steam did not start. Launch it once from your menu, then try the switch again.",
    ))
}

fn wait_for_client(monitor: &mut ProcessMonitor, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        monitor.refresh();
        if monitor.is_steam_running() {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        sleep(Duration::from_millis(250));
    }
}

/// Asks Steam to close, returning the helper process when one was started.
///
/// `steam -shutdown` is not always forwarded to the running client: when it is
/// not, the launcher starts a whole client that verifies the installation for
/// half a minute before the shutdown happens. The runtime helper talks to the
/// client through its pipe and exits immediately, so prefer it.
fn request_shutdown_process(installation: &SteamInstallation) -> Option<Child> {
    if installation.launch_command.is_empty() {
        return None;
    }

    #[cfg(target_os = "linux")]
    if let Some(command) = shutdown_command(installation) {
        if let Ok(mut child) = run_detached_process(&command) {
            if child_exits(&mut child) {
                return None;
            }
        }
    }

    let mut command = installation.launch_command.clone();
    command.push("-shutdown".to_string());
    run_detached_process(&command).ok()
}

/// Waits briefly for a helper that should finish on its own.
#[cfg(target_os = "linux")]
fn child_exits(child: &mut Child) -> bool {
    for _ in 0..20 {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) => sleep(Duration::from_millis(100)),
            Err(_) => return false,
        }
    }
    true
}

#[cfg(target_os = "linux")]
fn shutdown_command(installation: &SteamInstallation) -> Option<Vec<String>> {
    let mut bases = vec![installation.root.clone()];
    if let Some(prefix) = &installation.prefix {
        bases.push(prefix.join("root"));
    }

    bases
        .into_iter()
        .flat_map(|base| {
            ["amd64", "i386"].map(|arch| {
                base.join(format!(
                    "ubuntu12_32/steam-runtime/{arch}/usr/bin/steam-runtime-steam-remote"
                ))
            })
        })
        .find(|path| path.is_file())
        .map(|path| vec![path.to_string_lossy().to_string(), "-shutdown".to_string()])
}

/// Convenience wrapper for callers that do not keep a monitor around.
pub fn is_running() -> bool {
    let mut monitor = ProcessMonitor::new();
    monitor.refresh();
    monitor.is_steam_running()
}

/// Everything the account manager needs to do with Steam processes.
///
/// It is a trait so unit tests can use [`NoProcesses`]: without it a test that
/// removes an account would stop the Steam client of whoever runs the suite.
pub trait ProcessControl: Send + Sync {
    fn is_running(&self) -> bool;
    fn terminate(&self, installation: &SteamInstallation, timeout: Duration) -> Result<()>;
    fn launch(&self, installation: &SteamInstallation) -> Result<()>;
}

/// The real thing: talks to the running Steam client.
#[derive(Debug, Default)]
pub struct SystemProcesses;

impl ProcessControl for SystemProcesses {
    fn is_running(&self) -> bool {
        is_running()
    }

    fn terminate(&self, installation: &SteamInstallation, timeout: Duration) -> Result<()> {
        terminate_steam(installation, timeout)
    }

    fn launch(&self, installation: &SteamInstallation) -> Result<()> {
        launch_steam_verified(installation)
    }
}

/// Inert implementation for tests: Steam is never running and never touched.
#[derive(Debug, Default)]
pub struct NoProcesses;

impl ProcessControl for NoProcesses {
    fn is_running(&self) -> bool {
        false
    }

    fn terminate(&self, _installation: &SteamInstallation, _timeout: Duration) -> Result<()> {
        Ok(())
    }

    fn launch(&self, _installation: &SteamInstallation) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_steam_processes_on_this_platform() {
        for name in STEAM_PROCESS_NAMES {
            assert!(is_steam_process(name), "{name} should be recognised");
        }
        assert!(is_steam_process("steam.exe") || !cfg!(windows));
        assert!(!is_steam_process("firefox"));
        assert!(!is_steam_process("steam-account-switcher"));
    }

    #[cfg(windows)]
    #[test]
    fn background_services_do_not_keep_the_client_running() {
        // Services and crash reporters can outlive the client. Waiting for
        // them to exit can prevent a switch or try to kill a privileged service.
        assert!(!is_steam_process("steamservice.exe"));
        assert!(!is_steam_process("steamerrorreporter.exe"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn finds_the_runtime_helper_that_stops_steam() {
        use crate::model::InstallKind;

        let root = std::env::temp_dir().join("steam-core-shutdown-helper");
        let helper =
            root.join("ubuntu12_32/steam-runtime/amd64/usr/bin/steam-runtime-steam-remote");
        std::fs::create_dir_all(helper.parent().unwrap()).expect("helper directory");
        std::fs::write(&helper, b"").expect("helper file");

        let installation = SteamInstallation {
            kind: InstallKind::Native,
            config_dir: root.join("config"),
            loginusers_path: root.join("config/loginusers.vdf"),
            registry_path: root.join("registry.vdf"),
            userdata_dir: root.join("userdata"),
            launch_command: vec!["steam".to_string()],
            prefix: None,
            root: root.clone(),
        };

        let command = shutdown_command(&installation).expect("helper is used");
        assert_eq!(command[0], helper.to_string_lossy());
        assert_eq!(command[1], "-shutdown");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn monitor_reports_its_own_process_as_running() {
        let mut monitor = ProcessMonitor::new();
        let pid = std::process::id();

        // Reading the process table can be slow on a loaded machine, so retry
        // briefly instead of failing the suite on a single empty scan.
        let mut facts = monitor.process_facts(pid);
        for _ in 0..10 {
            if facts.running && facts.start_time.is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
            monitor.refresh();
            facts = monitor.process_facts(pid);
        }

        assert!(facts.running, "the monitor must see its own process");
        assert!(
            facts.start_time.is_some(),
            "the process must have a start time"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn appimage_paths_stay_out_of_child_environments() {
        let mount = PathBuf::from("/tmp/.mount_Steam.dBMkPE");

        assert!(belongs_to_appimage(
            Path::new("/tmp/.mount_Steam.dBMkPE/usr/lib/x86_64-linux-gnu"),
            Some(&mount)
        ));
        // The mount suffix is random, so an unknown APPDIR must not defeat the
        // filter: anything under the AppImage mount roots counts as well.
        assert!(belongs_to_appimage(
            Path::new("/tmp/.mount_Other123/usr/lib"),
            None
        ));
        assert!(!belongs_to_appimage(Path::new("/usr/lib"), Some(&mount)));

        let value =
            OsStr::new("/tmp/.mount_Steam.dBMkPE/usr/lib:/usr/lib:/tmp/.mount_Steam.dBMkPE/lib64");
        assert_eq!(
            without_appimage_paths(value, Some(&mount)),
            vec![PathBuf::from("/usr/lib")]
        );

        let only_image = OsStr::new("/tmp/.mount_Steam.dBMkPE/lib");
        assert!(without_appimage_paths(only_image, Some(&mount)).is_empty());
    }
}
