//! Bounded local diagnostics. Command arguments, results and raw errors must
//! never be logged: they can contain passwords, tokens or authenticator secrets.

use std::fs::{self, OpenOptions};
use std::future::Future;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;
use steam_core::error_code::ErrorCode;

use steam_core::util::absolute_time;

const MAX_BYTES: u64 = 256 * 1024;
const PREVIEW_BYTES: u64 = 64 * 1024;
static TARGET: Mutex<Option<PathBuf>> = Mutex::new(None);
static NEXT_OPERATION: AtomicU64 = AtomicU64::new(1);

#[derive(serde::Serialize)]
pub struct LogSnapshot {
    pub path: String,
    pub content: String,
    pub truncated: bool,
    pub previous_available: bool,
}

pub fn directory() -> Result<PathBuf, String> {
    let slot = TARGET
        .lock()
        .map_err(|_| ErrorCode::LogUnavailable.as_str().to_string())?;
    slot.as_ref()
        .and_then(|path| path.parent())
        .map(Path::to_path_buf)
        .ok_or_else(|| ErrorCode::LogUnavailable.as_str().to_string())
}

pub fn snapshot(previous: bool) -> Result<LogSnapshot, String> {
    // Keep rotation from changing the selected file during the read.
    let slot = TARGET
        .lock()
        .map_err(|_| ErrorCode::LogUnavailable.as_str().to_string())?;
    let current = slot
        .as_ref()
        .ok_or_else(|| ErrorCode::LogUnavailable.as_str().to_string())?;
    read_snapshot(current, previous).map_err(|_| ErrorCode::LogUnavailable.to_string())
}

fn read_snapshot(current: &Path, previous: bool) -> std::io::Result<LogSnapshot> {
    let backup = current.with_extension("log.old");
    let path = if previous { &backup } else { current };
    let (content, truncated) = match fs::File::open(path) {
        Ok(mut file) => {
            let length = file.metadata()?.len();
            let truncated = length > PREVIEW_BYTES;
            file.seek(SeekFrom::Start(length.saturating_sub(PREVIEW_BYTES)))?;
            let mut bytes = Vec::new();
            file.take(PREVIEW_BYTES).read_to_end(&mut bytes)?;
            // Omit the partial leading entry rather than showing a broken line.
            let start = if truncated {
                bytes
                    .iter()
                    .position(|byte| *byte == b'\n')
                    .map_or(0, |position| position + 1)
            } else {
                0
            };
            (
                String::from_utf8_lossy(&bytes[start..]).into_owned(),
                truncated,
            )
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (String::new(), false),
        Err(error) => return Err(error),
    };
    Ok(LogSnapshot {
        path: path.to_string_lossy().into_owned(),
        content,
        truncated,
        previous_available: backup.is_file(),
    })
}

pub fn init(directory: PathBuf) {
    if fs::create_dir_all(&directory).is_err() {
        return;
    }
    let path = directory.join("steam-account-switcher.log");
    // Restrict existing logs too; newly created files are already private.
    private_permissions(&path);
    private_permissions(&path.with_extension("log.old"));
    if let Ok(mut slot) = TARGET.lock() {
        *slot = Some(path);
    }
}

pub fn write(message: &str) {
    let Ok(slot) = TARGET.lock() else {
        return;
    };
    let Some(path) = slot.as_ref() else {
        return;
    };
    // Serialize rotation and writes as well as target access. Otherwise two
    // concurrent operations can overwrite the backup during rotation.
    let _ = append(path, message);
}

fn append(path: &Path, message: &str) -> std::io::Result<()> {
    if fs::metadata(path).is_ok_and(|meta| meta.len() >= MAX_BYTES) {
        let backup = path.with_extension("log.old");
        // Unix rename replaces the destination atomically; Windows does not.
        #[cfg(windows)]
        match fs::remove_file(&backup) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        fs::rename(path, backup)?;
    }
    let mut options = OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    // Keep each event on one line even if an internal diagnostic has newlines.
    writeln!(
        file,
        "[{}] {}",
        now_text(),
        message.replace(['\r', '\n'], " ")
    )
}

fn private_permissions(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }
    #[cfg(not(unix))]
    let _ = path;
}

/// Log a user action, including early returns via `?`, without serializing
/// any input, output or error. Names must be static, never supplied by the UI.
pub fn run<T>(name: &'static str, action: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    let operation = Operation::start(name, false);
    operation.finish(action())
}

pub async fn run_async<T>(
    name: &'static str,
    action: impl Future<Output = Result<T, String>>,
) -> Result<T, String> {
    let operation = Operation::start(name, false);
    operation.finish(action.await)
}

/// Successful reads and rotating code generation are intentionally quiet.
pub fn read<T>(
    name: &'static str,
    action: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    let operation = Operation::start(name, true);
    operation.finish(action())
}

struct Operation {
    name: &'static str,
    id: u64,
    started: Instant,
    quiet: bool,
    finished: bool,
}

impl Operation {
    fn start(name: &'static str, quiet: bool) -> Self {
        let operation = Self {
            name,
            id: NEXT_OPERATION.fetch_add(1, Ordering::Relaxed),
            started: Instant::now(),
            quiet,
            finished: false,
        };
        if !quiet {
            write(&format!(
                "operation={} action={name} status=started",
                operation.id
            ));
        }
        operation
    }

    fn finish<T>(mut self, result: Result<T, String>) -> Result<T, String> {
        let category = result.as_ref().err().map(|error| error_category(error));
        if !self.quiet || category.is_some() {
            write(&self.outcome(category));
        }
        self.finished = true;
        result
    }

    fn outcome(&self, category: Option<&'static str>) -> String {
        let status = if category.is_some() {
            "failed"
        } else {
            "succeeded"
        };
        let mut event = format!(
            "operation={} action={} status={status} duration_ms={}",
            self.id,
            self.name,
            self.started.elapsed().as_millis()
        );
        if let Some(category) = category {
            event.push_str(&format!(" error={category}"));
        }
        event
    }
}

impl Drop for Operation {
    fn drop(&mut self) {
        if !self.finished {
            write(&format!(
                "operation={} action={} status=interrupted duration_ms={}",
                self.id,
                self.name,
                self.started.elapsed().as_millis()
            ));
        }
    }
}

/// Return only fixed categories. Never append an arbitrary error string,
/// even one that looks harmless: SDK errors may contain request URLs/tokens.
pub fn error_category(message: &str) -> &'static str {
    ErrorCode::from_code(message).map_or("unclassified", ErrorCode::as_str)
}

pub fn clipboard_error_category(message: &str) -> &'static str {
    if message.contains("held by another party") {
        "clipboard_occupied"
    } else if message.contains("not supported") {
        "clipboard_unsupported"
    } else if message.contains("could not be converted") {
        "clipboard_conversion_failed"
    } else {
        "clipboard_unavailable"
    }
}

fn now_text() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or_default();
    absolute_time(Some(seconds))
}
