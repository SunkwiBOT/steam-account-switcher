use base64::Engine;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use steam_core::error_code::ErrorCode;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_updater::{Update, Updater, UpdaterExt};

use super::dto::{UpdateInfoDto, UpdateProgressDto};

// ! A stalled connection must not leave the UI's `checkForUpdate` stuck at CHECKING forever, so both the request and the read timeout are bounded.
const CHECK_TIMEOUT: Duration = Duration::from_secs(30);
const STALL_TIMEOUT: Duration = Duration::from_secs(30);

// ! Matches the extra platform key the release workflow adds to `latest.json` for the portable Windows copy.
const PORTABLE_TARGET: &str = "windows-x86_64-portable";

// ! The NSIS installer leaves its uninstaller beside the executable; a portable copy has none.
const PORTABLE_MARKER: &str = "uninstall.exe";

fn installed_beside(dir: &Path) -> bool {
    dir.join(PORTABLE_MARKER).exists()
}

fn portable() -> bool {
    cfg!(windows)
        && std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|dir| !installed_beside(dir)))
            .unwrap_or(false)
}

// ! A successful portable swap leaves the previous build at `<name>.bak`, which nothing else ever cleans up, so it is removed once here at every start.
pub fn cleanup_stale_backup() {
    if !portable() {
        return;
    }
    if let Ok(exe_path) = std::env::current_exe() {
        let backup_path = sibling(&exe_path, "bak");
        if backup_path.exists() {
            let _ = std::fs::remove_file(&backup_path);
        }
    }
}

fn updater(app: &AppHandle, target: Option<&str>) -> Result<Updater, String> {
    let mut builder = app
        .updater_builder()
        .timeout(CHECK_TIMEOUT)
        .configure_client(|client| client.read_timeout(STALL_TIMEOUT));
    if let Some(target) = target {
        builder = builder.target(target);
    }
    builder
        .build()
        .map_err(|_| ErrorCode::UpdateFailed.to_string())
}

async fn find_update(app: &AppHandle, target: Option<&str>) -> Result<Option<Update>, String> {
    updater(app, target)?
        .check()
        .await
        .map_err(|error| portable_update_error(target, &error))
}

// ! A missing `windows-x86_64-portable` key in `latest.json` means this release predates portable self-update.
fn portable_update_error(target: Option<&str>, error: &tauri_plugin_updater::Error) -> String {
    if target.is_some() && matches!(error, tauri_plugin_updater::Error::TargetNotFound(_)) {
        return ErrorCode::UpdatePortableUnsupported.to_string();
    }
    ErrorCode::UpdateFailed.to_string()
}

#[tauri::command]
pub async fn check_for_update(
    app: tauri::AppHandle,
    state: State<'_, crate::state::AppState>,
) -> Result<Option<UpdateInfoDto>, String> {
    let result = crate::logging::run_async("check_for_update", async {
        let update = find_update(&app, None).await?;

        Ok(update.map(|update| UpdateInfoDto {
            version: update.version.clone(),
            current_version: update.current_version.clone(),
            notes: update.body.clone(),
        }))
    })
    .await;
    let mut inner = state.lock();
    let previous = inner.settings.data().last_update_check_at;
    inner.settings.data_mut().last_update_check_at = Some(steam_core::util::now_unix() as u64);
    if inner.settings.save().is_err() {
        inner.settings.data_mut().last_update_check_at = previous;
        crate::logging::write(
            "action=save_update_check status=failed category=settings_save_failed",
        );
    }
    result
}

#[derive(Default)]
pub struct PendingUpdate(Mutex<Option<DownloadedUpdate>>);

impl PendingUpdate {
    pub fn new() -> Self {
        Self::default()
    }
}

struct DownloadedUpdate {
    update: Update,
    bytes: Vec<u8>,
}

#[tauri::command]
pub async fn download_update(
    app: tauri::AppHandle,
    pending: State<'_, PendingUpdate>,
) -> Result<String, String> {
    crate::logging::run_async("download_update", async {
        let target = portable().then_some(PORTABLE_TARGET);
        let update = find_update(&app, target)
            .await?
            .ok_or_else(|| ErrorCode::NoUpdate.as_str().to_string())?;
        let version = update.version.clone();

        let emitter = app.clone();
        let mut downloaded: u64 = 0;
        let bytes = update
            .download(
                move |chunk_length, total| {
                    downloaded += chunk_length as u64;
                    let _ =
                        emitter.emit("update-progress", UpdateProgressDto { downloaded, total });
                },
                || {},
            )
            .await
            .map_err(|error| portable_update_error(target, &error))?;

        let mut slot = pending
            .0
            .lock()
            .map_err(|_| ErrorCode::UpdateFailed.to_string())?;
        *slot = Some(DownloadedUpdate { update, bytes });
        drop(slot);

        let _ = app.emit("update-downloaded", version.clone());
        Ok(version)
    })
    .await
}

#[tauri::command]
pub async fn apply_update(
    app: tauri::AppHandle,
    state: State<'_, crate::state::AppState>,
    pending: State<'_, PendingUpdate>,
    version: String,
) -> Result<(), String> {
    crate::logging::run_async("apply_update", async {
        // ! Checked before the slot is taken: refusing here never loses the already-downloaded update, which can still be applied once the link finishes.
        if state.link_operation_in_flight() {
            crate::logging::write("action=apply_update status=blocked reason=link_in_flight");
            return Err(ErrorCode::LinkInFlight.to_string());
        }

        // ! Only the version the user was shown may be installed; a different downloaded build stays pending and is not taken.
        let downloaded = {
            let mut slot = pending
                .0
                .lock()
                .map_err(|_| ErrorCode::UpdateFailed.to_string())?;
            match slot.as_ref() {
                Some(candidate) if candidate.update.version == version => slot.take(),
                _ => None,
            }
        }
        .ok_or_else(|| ErrorCode::NoDownloadedUpdate.as_str().to_string())?;

        crate::logging::write("action=apply_update stage=installing");

        if portable() {
            let exe_path =
                std::env::current_exe().map_err(|_| ErrorCode::UpdateFailed.to_string())?;
            install_portable_update(
                &app,
                &exe_path,
                &downloaded.bytes,
                &downloaded.update.signature,
            )?;
        } else {
            downloaded
                .update
                .install(&downloaded.bytes)
                .map_err(|_| ErrorCode::UpdateFailed.to_string())?;
        }

        crate::logging::write("action=apply_update status=succeeded restart=true");

        app.restart();
    })
    .await
}

fn updater_pubkey(app: &AppHandle) -> Option<String> {
    app.config()
        .plugins
        .0
        .get("updater")?
        .get("pubkey")?
        .as_str()
        .map(str::to_owned)
}

fn decode_minisign_text(value: &str) -> Option<String> {
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(value)
        .ok()?;
    String::from_utf8(decoded).ok()
}

// ! Re-checks the downloaded bytes against the updater public key right before they replace the running executable.
fn verify_minisign(data: &[u8], signature_b64: &str, pubkey_b64: &str) -> bool {
    let Some(pubkey_text) = decode_minisign_text(pubkey_b64) else {
        return false;
    };
    let Some(signature_text) = decode_minisign_text(signature_b64) else {
        return false;
    };
    let Ok(public_key) = minisign_verify::PublicKey::decode(&pubkey_text) else {
        return false;
    };
    let Ok(signature) = minisign_verify::Signature::decode(&signature_text) else {
        return false;
    };
    public_key.verify(data, &signature, true).is_ok()
}

fn sibling(exe_path: &Path, suffix: &str) -> PathBuf {
    let mut name = exe_path.file_name().unwrap_or_default().to_os_string();
    name.push(".");
    name.push(suffix);
    exe_path.with_file_name(name)
}

// ! Windows allows renaming a file still mapped by the running process but not overwriting it, so the live exe is moved aside first.
fn replace_exe(exe_path: &Path, backup_path: &Path, temp_path: &Path) -> io::Result<()> {
    std::fs::rename(exe_path, backup_path)?;
    if let Err(error) = std::fs::rename(temp_path, exe_path) {
        if let Err(restore_error) = restore_backup(exe_path, backup_path) {
            crate::logging::write(&format!(
                "action=apply_update stage=restore status=failed error={restore_error}"
            ));
        }
        return Err(error);
    }
    Ok(())
}

// ! A plain rename, never delete-then-restore: once the target is gone there is nothing to fall back to.
fn restore_backup(exe_path: &Path, backup_path: &Path) -> io::Result<()> {
    std::fs::rename(backup_path, exe_path)
}

// ! A swapped-in exe launched with this flag exits before opening a window, taking the single-instance lock or touching the vault database.
pub const SELFTEST_ARG: &str = "--update-selftest";
const SELFTEST_TIMEOUT: Duration = Duration::from_secs(5);

// ! A signature check only proves the bytes were signed, not that the resulting executable can run on this machine, so this is the last chance to catch that before the user restarts into it.
fn verify_exe_starts(exe_path: &Path) -> bool {
    let Ok(mut child) = Command::new(exe_path).arg(SELFTEST_ARG).spawn() else {
        return false;
    };
    let deadline = Instant::now() + SELFTEST_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(20));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return false;
            }
            Err(_) => return false,
        }
    }
}

// ! No Tauri bundle format applies to a bare portable exe, so it is written beside itself and swapped in directly instead of going through `Update::install`.
fn install_portable_update(
    app: &AppHandle,
    exe_path: &Path,
    bytes: &[u8],
    signature: &str,
) -> Result<(), String> {
    let pubkey = updater_pubkey(app).ok_or_else(|| ErrorCode::UpdateFailed.to_string())?;
    if !verify_minisign(bytes, signature, &pubkey) {
        return Err(ErrorCode::UpdateFailed.to_string());
    }

    let temp_path = sibling(exe_path, "new");
    let backup_path = sibling(exe_path, "bak");
    std::fs::write(&temp_path, bytes).map_err(|_| ErrorCode::UpdateFailed.to_string())?;
    if !verify_exe_starts(&temp_path) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(ErrorCode::UpdateFailed.to_string());
    }

    if replace_exe(exe_path, &backup_path, &temp_path).is_err() {
        let _ = std::fs::remove_file(&temp_path);
        return Err(ErrorCode::UpdateFailed.to_string());
    }

    Ok(())
}
