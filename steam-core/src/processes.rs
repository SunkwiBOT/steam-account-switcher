use crate::error_code::ErrorCode;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

use sysinfo::{
    Pid, Process, ProcessRefreshKind, ProcessStatus, ProcessesToUpdate, System, Uid, UpdateKind,
};

#[cfg(windows)]
use crate::autologin;
use crate::error::{Result, SteamError};
use crate::model::SteamInstallation;

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

// ! On Linux, `Process::name()` reads the kernel's `comm` field, capped at 15 bytes, so candidates are truncated the same way before comparing.
#[cfg(not(windows))]
const PROCESS_NAME_LIMIT: usize = 15;

fn is_steam_process(name: &str) -> bool {
    let lowered = name.to_ascii_lowercase();
    STEAM_PROCESS_NAMES.iter().any(|candidate| {
        let candidate = truncated_process_name(candidate);
        lowered == candidate || lowered == format!("{candidate}.exe")
    })
}

fn truncated_process_name(candidate: &str) -> String {
    let lowered = candidate.to_ascii_lowercase();
    #[cfg(windows)]
    {
        lowered
    }
    #[cfg(not(windows))]
    {
        lowered.chars().take(PROCESS_NAME_LIMIT).collect()
    }
}

// ! `steam -shutdown` runs as a `steam` process of its own and outlives the client; counting it as a client would make the shutdown never finish.
fn is_shutdown_request(process: &Process) -> bool {
    process.cmd().iter().any(|argument| argument == "-shutdown")
}

const LAUNCH_ATTEMPT_TIMEOUT: Duration = Duration::from_secs(8);
const LAUNCH_ATTEMPTS: usize = 4;
// ! No documented minimum exists for how long the launcher takes to hand off, so this gap is a cautious multiple of the retry loop's poll interval.
const LAUNCH_RETRY_GAP: Duration = Duration::from_millis(1500);
// ! Steam's shutdown can close the main UI process before `steamwebhelper` and its runtime helpers fully exit, so launching immediately risks handing off to one of those.
const SHUTDOWN_SETTLE: Duration = Duration::from_millis(1500);
// ! `steam_processes` does not exclude shutdown requests, so a leftover `-shutdown` helper is given this long to finish before a new launch proceeds anyway.
const SHUTDOWN_HELPER_GRACE: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProcessFacts {
    pub running: bool,
    pub start_time: Option<f64>,
}

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
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing()
                .with_exe(UpdateKind::OnlyIfNotSet)
                .with_user(UpdateKind::OnlyIfNotSet)
                .with_cmd(UpdateKind::OnlyIfNotSet),
        );
    }

    // ! Read back from this process itself rather than an OS-specific call, so another session's Steam is never treated as this one's client.
    fn current_user_id(&self) -> Option<&Uid> {
        self.system
            .process(Pid::from_u32(std::process::id()))
            .and_then(Process::user_id)
    }

    fn steam_processes_matching(&self, exclude_shutdown_requests: bool) -> Vec<(u32, String)> {
        let own_pid = std::process::id();
        let own_user = self.current_user_id();
        self.system
            .processes()
            .iter()
            .filter(|(pid, _)| pid.as_u32() != own_pid)
            .filter(|(_, process)| process.status() != ProcessStatus::Zombie)
            .filter(|(_, process)| matches_user(own_user, process.user_id()))
            .filter(|(_, process)| !exclude_shutdown_requests || !is_shutdown_request(process))
            .map(|(pid, process)| (pid.as_u32(), process.name().to_string_lossy().to_string()))
            .filter(|(_, name)| is_steam_process(name))
            .collect()
    }

    pub fn steam_processes(&self) -> Vec<(u32, String)> {
        self.steam_processes_matching(false)
    }

    pub fn steam_client_processes(&self) -> Vec<(u32, String)> {
        self.steam_processes_matching(true)
    }

    pub fn is_steam_running(&self) -> bool {
        !self.steam_client_processes().is_empty()
    }

    // ! Narrows a list already filtered by `steam_processes`/`steam_client_processes` to the ones plausibly belonging to this installation's own binary, where that binary's absolute path is known.
    fn filtered_for(
        &self,
        processes: Vec<(u32, String)>,
        installation: &SteamInstallation,
    ) -> Vec<(u32, String)> {
        processes
            .into_iter()
            .filter(|(pid, _)| {
                self.system
                    .process(Pid::from_u32(*pid))
                    .is_some_and(|process| matches_installation(process, installation))
            })
            .collect()
    }

    pub fn steam_processes_for(&self, installation: &SteamInstallation) -> Vec<(u32, String)> {
        self.filtered_for(self.steam_processes(), installation)
    }

    pub fn steam_client_processes_for(
        &self,
        installation: &SteamInstallation,
    ) -> Vec<(u32, String)> {
        self.filtered_for(self.steam_client_processes(), installation)
    }

    pub fn is_steam_running_for(&self, installation: &SteamInstallation) -> bool {
        !self.steam_client_processes_for(installation).is_empty()
    }

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

// ! A process whose owner cannot be read (permission boundary) is kept rather than hidden, since hiding it would be a guess in the other direction.
fn matches_user(mine: Option<&Uid>, theirs: Option<&Uid>) -> bool {
    match (mine, theirs) {
        (Some(mine), Some(theirs)) => mine == theirs,
        _ => true,
    }
}

// ! Checks containment in the installation root rather than equality, since the launcher script (`steam.sh`) and the client binary it hands off to (e.g. `ubuntu12_32/steam`) are different files; Windows' `SteamPath` is stored lower-case, so that side compares case-insensitively.
fn matches_installation(process: &Process, installation: &SteamInstallation) -> bool {
    match process.exe() {
        Some(exe) if !exe.as_os_str().is_empty() => contained_in(exe, &installation.root),
        // ! `exe()` can come back empty across a UID/namespace boundary (sysinfo's own documented caveat); keeping the process avoids hiding a client this process simply could not inspect.
        _ => true,
    }
}

fn canonicalize_best_effort(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn contained_in(exe: &Path, root: &Path) -> bool {
    let exe = canonicalize_best_effort(exe);
    let root = canonicalize_best_effort(root);
    #[cfg(windows)]
    {
        let exe = exe.to_string_lossy().to_ascii_lowercase();
        let root = root.to_string_lossy().to_ascii_lowercase();
        Path::new(&exe).starts_with(Path::new(&root))
    }
    #[cfg(not(windows))]
    {
        exe.starts_with(&root)
    }
}

// ! The shutdown request uses the same command line Steam was launched with, so Flatpak and Snap installations are handled without special casing.
pub fn terminate_steam(installation: &SteamInstallation, timeout: Duration) -> Result<()> {
    let mut monitor = ProcessMonitor::new();
    monitor.refresh();
    if !monitor.is_steam_running_for(installation) {
        wait_for_shutdown_helpers(&mut monitor, installation, SHUTDOWN_HELPER_GRACE);
        return Ok(());
    }

    let mut shutdown = request_shutdown_process(installation);

    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        reap(&mut shutdown);
        monitor.refresh();
        if !monitor.is_steam_running_for(installation) {
            wait_for_shutdown_helpers(&mut monitor, installation, SHUTDOWN_HELPER_GRACE);
            sleep(SHUTDOWN_SETTLE);
            return Ok(());
        }
        sleep(Duration::from_millis(250));
    }

    monitor.refresh();
    for (pid, _) in monitor.steam_client_processes_for(installation) {
        if let Some(process) = monitor.system.process(Pid::from_u32(pid)) {
            let _ = process.kill();
        }
    }

    let force_deadline = Instant::now() + Duration::from_secs(8);
    while Instant::now() < force_deadline {
        reap(&mut shutdown);
        monitor.refresh();
        if !monitor.is_steam_running_for(installation) {
            wait_for_shutdown_helpers(&mut monitor, installation, SHUTDOWN_HELPER_GRACE);
            sleep(SHUTDOWN_SETTLE);
            return Ok(());
        }
        sleep(Duration::from_millis(250));
    }

    Err(SteamError::config(ErrorCode::SteamStopFailed))
}

// ! Collects the process of a shutdown request, so it does not stay behind as a zombie that keeps looking like a running client.
fn reap(shutdown: &mut Option<Child>) {
    if let Some(child) = shutdown.as_mut() {
        if matches!(child.try_wait(), Ok(Some(_)) | Err(_)) {
            *shutdown = None;
        }
    }
}

// ! Starting Steam while a shutdown request is still running only feeds that dying helper, so give it a moment before the launch is attempted.
fn wait_for_shutdown_helpers(
    monitor: &mut ProcessMonitor,
    installation: &SteamInstallation,
    timeout: Duration,
) {
    let deadline = Instant::now() + timeout;
    loop {
        monitor.refresh();
        if monitor.steam_processes_for(installation).is_empty() {
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
    // ! Own process group: Steam keeps running when the switcher exits.
    command.process_group(0);
}

#[cfg(windows)]
fn detach(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    command.creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS);
}

// ! Inherited AppImage libraries and GIO modules break the Steam Linux Runtime's host Python, so these launcher variables must not reach Steam.
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

const APPIMAGE_PATH_VARIABLES: [&str; 3] = ["LD_LIBRARY_PATH", "PATH", "XDG_DATA_DIRS"];

fn running_from_appimage() -> bool {
    std::env::var_os("APPDIR").is_some() || std::env::var_os("APPIMAGE").is_some()
}

fn appimage_mount() -> Option<PathBuf> {
    std::env::var_os("APPDIR")
        .or_else(|| std::env::var_os("APPIMAGE"))
        .map(PathBuf::from)
}

// ! The mount point carries a random suffix, so a stale `APPDIR` is not enough: anything under `/tmp/.mount_` is an AppImage path as well.
fn belongs_to_appimage(entry: &Path, mount: Option<&Path>) -> bool {
    entry.to_string_lossy().starts_with("/tmp/.mount_")
        || mount.is_some_and(|mount| entry.starts_with(mount))
}

fn without_appimage_paths(value: &OsStr, mount: Option<&Path>) -> Vec<PathBuf> {
    std::env::split_paths(value)
        .filter(|entry| !belongs_to_appimage(entry, mount))
        .collect()
}

// ! Only an AppImage needs this: a plain build runs with the user's own environment, which is exactly what a menu launch gives Steam.
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
    let child = run_detached_process(command)?;
    reap_in_background(child);
    Ok(())
}

// ! `Child` is dropped without calling `wait`, which never reaps a finished process; a detached thread blocking on `wait` reaps it without stalling the caller.
fn reap_in_background(mut child: Child) {
    std::thread::spawn(move || {
        let _ = child.wait();
    });
}

fn run_detached_process(command: &[String]) -> Result<Child> {
    let (program, arguments) = command
        .split_first()
        .ok_or_else(|| SteamError::config(ErrorCode::SteamStartFailed))?;

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

// ! The `steam` launcher hands its command line to an already running client and exits, so a launch issued while the previous client is still closing silently does nothing; the retries cover that window.
pub fn launch_steam_verified(installation: &SteamInstallation) -> Result<()> {
    if installation.launch_command.is_empty() {
        return Err(SteamError::config(ErrorCode::SteamStartFailed));
    }

    let mut monitor = ProcessMonitor::new();
    for attempt in 1..=LAUNCH_ATTEMPTS {
        run_detached(&installation.launch_command)?;
        if wait_for_client(&mut monitor, installation, LAUNCH_ATTEMPT_TIMEOUT) {
            return Ok(());
        }
        if attempt < LAUNCH_ATTEMPTS {
            sleep(LAUNCH_RETRY_GAP);
        }
    }

    Err(SteamError::config(ErrorCode::SteamStartFailed))
}

fn wait_for_client(
    monitor: &mut ProcessMonitor,
    installation: &SteamInstallation,
    timeout: Duration,
) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        monitor.refresh();
        if client_verified(monitor, installation) {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        sleep(Duration::from_millis(250));
    }
}

// ! The launcher itself is spawned as a process named "steam", which appears the instant it is spawned and well before the real client exists, so `is_steam_running` alone cannot verify a launch; on Windows the registry's own PID report is the only way to tell them apart.
#[cfg(windows)]
fn client_verified(monitor: &mut ProcessMonitor, installation: &SteamInstallation) -> bool {
    steam_owns_a_running_pid(monitor, installation)
}

// ! `is_steam_running_for`'s containment check already tells the client binary apart from the launcher script (B4), which a published pid cannot: a stale `steam.pid` surviving a reboot, or a Flatpak pid namespace, make that pid meaningless.
#[cfg(not(windows))]
fn client_verified(monitor: &mut ProcessMonitor, installation: &SteamInstallation) -> bool {
    monitor.is_steam_running_for(installation)
}

#[cfg(windows)]
fn steam_owns_a_running_pid(
    monitor: &mut ProcessMonitor,
    installation: &SteamInstallation,
) -> bool {
    match autologin::read_active_process_pid(installation) {
        Ok(Some(pid)) if pid != 0 => monitor.process_facts(pid).running,
        _ => false,
    }
}

// ! `steam -shutdown` is not always forwarded to the running client: when it is not, the launcher starts a whole client that verifies the installation for half a minute first. The runtime helper talks to the client through its pipe and exits immediately, so it is preferred.
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
            // ! Reached only when the helper already exited with failure, not when it is still running past the grace period above: nothing else will ever call `wait` on it once this function returns without keeping the handle.
            reap_in_background(child);
        }
    }

    let mut command = installation.launch_command.clone();
    command.push("-shutdown".to_string());
    run_detached_process(&command).ok()
}

// ! `steam-runtime-steam-remote -shutdown` is a one-shot script: once it has forwarded the request to the running client it exits almost immediately. Twenty attempts of 100ms (2 seconds total) is a generous multiple of every forwarding time observed locally; if it is still alive past that point it is treated as a background helper rather than a failure, since killing it would also kill the shutdown request it is relaying.
#[cfg(target_os = "linux")]
fn child_exits(child: &mut Child) -> bool {
    const ATTEMPTS: usize = 20;
    const POLL_INTERVAL: Duration = Duration::from_millis(100);
    for _ in 0..ATTEMPTS {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) => sleep(POLL_INTERVAL),
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

pub trait ProcessControl: Send + Sync {
    fn is_running(&self, installation: &SteamInstallation) -> bool;
    fn terminate(&self, installation: &SteamInstallation, timeout: Duration) -> Result<()>;
    fn launch(&self, installation: &SteamInstallation) -> Result<()>;
}

#[derive(Debug, Default)]
pub struct SystemProcesses;

impl ProcessControl for SystemProcesses {
    fn is_running(&self, installation: &SteamInstallation) -> bool {
        let mut monitor = ProcessMonitor::new();
        monitor.refresh();
        monitor.is_steam_running_for(installation)
    }

    fn terminate(&self, installation: &SteamInstallation, timeout: Duration) -> Result<()> {
        terminate_steam(installation, timeout)
    }

    fn launch(&self, installation: &SteamInstallation) -> Result<()> {
        launch_steam_verified(installation)
    }
}
