#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

use steam_account_switcher_lib::{
    auth_commands, commands, logging, open_url, settings, show_main_window, state, tray,
};
use tauri::{AppHandle, Emitter, Manager, WindowEvent};

use state::AppState;

fn main() {
    // ! This flag exits before any window, lock file or database is touched, so a self-update check never collides with the running instance.
    if std::env::args().any(|arg| arg == commands::updates::SELFTEST_ARG) {
        std::process::exit(0);
    }
    prepare_linux_display();
    // ! Before the state is built, so a settings or vault file that cannot be read leaves a trace instead of failing silently.
    logging::init(settings::config_dir());

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            show_main_window(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None::<Vec<&str>>,
        ))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(commands::updates::PendingUpdate::new())
        .invoke_handler(tauri::generate_handler![
            commands::log_commands::logs_snapshot,
            commands::log_commands::logs_open_directory,
            commands::clipboard::clipboard_write_text,
            commands::snapshots::snapshot,
            commands::accounts::account_login_name,
            commands::snapshots::playtime_snapshot,
            commands::snapshots::refresh,
            commands::preferences::select_installation,
            commands::preferences::set_masked,
            commands::preferences::set_steam_ids_visible,
            commands::preferences::set_vault_offer_dismissed,
            commands::preferences::update_settings,
            commands::tracking::set_tracking_enabled,
            commands::tracking::clear_playtime_data,
            commands::preferences::set_language,
            commands::preferences::set_close_to_tray,
            commands::preferences::set_update_interval,
            commands::preferences::set_search_scope,
            commands::preferences::set_vault_lock_interval,
            commands::preferences::set_installation_hidden,
            commands::preferences::add_custom_installation,
            commands::preferences::remove_custom_installation,
            commands::preferences::create_folder,
            commands::preferences::update_folder,
            commands::preferences::delete_folder,
            commands::preferences::assign_folder,
            commands::preferences::set_account_color,
            commands::preferences::set_account_note,
            commands::preferences::set_folder_color,
            commands::preferences::set_folder_icon,
            commands::preferences::set_window_active,
            commands::preferences::app_info,
            open_url::open_repository,
            commands::accounts::add_steam_account,
            commands::updates::check_for_update,
            commands::updates::download_update,
            commands::updates::apply_update,
            commands::accounts::running_steam_games,
            commands::accounts::switch_account,
            commands::accounts::remove_account,
            commands::accounts::forget_saved_account,
            commands::accounts::set_rank,
            commands::reports::account_report,
            commands::reports::machine_report,
            commands::preferences::autostart_status,
            commands::preferences::set_autostart,
            auth_commands::vault::authenticator_unlock,
            auth_commands::vault::authenticator_lock,
            auth_commands::vault::authenticator_set_passphrase,
            auth_commands::guard::steam_guard_code,
            auth_commands::guard::steam_guard_time_offset,
            auth_commands::guard::steam_guard_revocation_code,
            auth_commands::guard::steam_guard_import,
            auth_commands::backup::steam_guard_export_backup,
            auth_commands::backup::steam_guard_import_backup,
            auth_commands::guard::steam_guard_confirmations,
            auth_commands::guard::steam_guard_confirmation_action,
            auth_commands::guard::steam_guard_remove,
            auth_commands::login::steam_guard_login_password,
            auth_commands::login::steam_guard_login_qr,
            auth_commands::login::steam_guard_login_finish,
            auth_commands::login::steam_guard_login_cancel,
            auth_commands::link::steam_guard_link_start,
            auth_commands::link::steam_guard_link_finalize,
            auth_commands::link::steam_guard_link_transfer_start,
            auth_commands::link::steam_guard_link_transfer_finish,
            auth_commands::link::steam_guard_link_revoke,
            auth_commands::link::steam_guard_link_cancel,
        ])
        .setup(|app| {
            app.manage(AppState::new());
            let previous_panic_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                // ! Panic payloads can contain secrets: only where it happened is logged.
                let location = info
                    .location()
                    .map(|location| location.to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                logging::write(&format!("runtime: panic location={location}"));
                previous_panic_hook(info);
            }));
            logging::write(&format!(
                "start: {} {}",
                app.package_info().name,
                app.package_info().version
            ));
            logging::write(&format!(
                "environment: os={} arch={} debug={}",
                std::env::consts::OS,
                std::env::consts::ARCH,
                cfg!(debug_assertions)
            ));
            {
                let state = app.state::<AppState>();
                let inner = state.lock();
                logging::write(&format!(
                    "catalog: installations={} accounts={} tracker_available={}",
                    inner.installations.len(),
                    inner.accounts.len(),
                    inner.tracker.is_some()
                ));
                drop(inner);
                let authenticator = state.authenticator();
                let vault = authenticator
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .status();
                logging::write(&format!(
                    "vault: encrypted={} locked={} indexed={} accounts={} load_error={}",
                    vault.encrypted,
                    vault.locked,
                    vault.indexed,
                    vault.count,
                    vault.error.is_some()
                ));
                if let Some(backup) = state.take_corrupt_db_backup() {
                    let locale = state.lock().settings.data().language.resolved();
                    state::show_database_recovered_notice(app.handle(), locale, &backup);
                }
            }
            if let Some(window) = app.get_webview_window("main") {
                configure_window(&window);
            }
            let tray_available = tray::install(app.handle()).unwrap_or_else(|error| {
                logging::write("tray: creation_failed");
                eprintln!("Unable to create the tray icon: {error}");
                false
            });
            logging::write(&format!("tray: available={tray_available}"));
            app.state::<AppState>().set_tray_available(tray_available);
            if tray_available {
                tray::refresh_menu(app.handle());
            }
            start_playtime_loop(app.handle().clone());
            start_clock_offset_loop(app.handle().clone());
            commands::updates::cleanup_stale_backup();
            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                WindowEvent::CloseRequested { api, .. } => {
                    let app = window.app_handle();
                    let state = app.state::<AppState>();
                    let close_to_tray = state.lock().settings.data().close_to_tray;
                    // ! Hiding without a tray icon would leave the app unreachable.
                    if state.tray_available() && close_to_tray {
                        api.prevent_close();
                        state.set_window_active(false);
                        let _ = window.hide();
                    } else {
                        state::request_close(app, api);
                    }
                }
                WindowEvent::Focused(focused) => {
                    window
                        .app_handle()
                        .state::<AppState>()
                        .set_window_active(*focused);
                }
                _ => {}
            }
        })
        .build(tauri::generate_context!())
        .expect("failed to build the Steam Account Switcher window")
        .run(|app_handle, event| {
            // ! All windows closed without a tray icon: the same guard as the close button and the tray Quit item applies.
            if let tauri::RunEvent::ExitRequested { api, .. } = &event {
                if state::exit_blocked(app_handle) {
                    api.prevent_exit();
                    return;
                }
            }
            if matches!(
                event,
                tauri::RunEvent::Exit | tauri::RunEvent::ExitRequested { .. }
            ) {
                app_handle.state::<AppState>().close_tracker();
            }
            if matches!(event, tauri::RunEvent::Exit) {
                logging::write("exit: normal");
            }
        });
}

// ! Without the icon, X11 `WM_CLASS` and Wayland `app_id`, the compositor cannot pair the window with the installed `.desktop` entry.
fn configure_window(window: &tauri::WebviewWindow) {
    if let Ok(icon) = tauri::image::Image::from_bytes(include_bytes!("../icons/128x128.png")) {
        let _ = window.set_icon(icon);
    }

    #[cfg(target_os = "linux")]
    gtk::Window::set_default_icon_name("accountswitcher");
}

// ! WebKitGTK's DMABUF renderer fails on Wayland with the NVIDIA driver, so it is disabled in that case only.
#[cfg(target_os = "linux")]
fn prepare_linux_display() {
    // ! Before GTK starts: KWin looks up `<app id>.desktop` for the window icon and the bundler names that entry after the product name; the icon name stays `accountswitcher` because a `steam-` prefix resolves to the Steam client's icon in some themes.
    glib::set_prgname(Some("Steam Account Switcher"));
    glib::set_application_name("Steam Account Switcher");

    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_some() {
        return;
    }

    let wayland = std::env::var_os("WAYLAND_DISPLAY").is_some()
        || std::env::var("XDG_SESSION_TYPE").is_ok_and(|value| value == "wayland");
    let nvidia = std::path::Path::new("/sys/module/nvidia").exists()
        || std::path::Path::new("/proc/driver/nvidia").exists();

    if wayland && nvidia {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }
}

#[cfg(not(target_os = "linux"))]
fn prepare_linux_display() {}

fn start_playtime_loop(app: AppHandle) {
    std::thread::spawn(move || {
        let mut tick: u64 = 0;
        loop {
            std::thread::sleep(Duration::from_millis(1000));
            tick += 1;

            let state = app.state::<AppState>();

            // ! Before the idle skip so the vault lock deadline stays exact in the background.
            if state.lock_vault_if_due() {
                let _ = app.emit("vault-locked", ());
            }

            let active = state.window_active();

            if !active && !tick.is_multiple_of(5) {
                continue;
            }

            let changed = match state.tracker() {
                Some(tracker) => match tracker.lock() {
                    Ok(mut tracker) => tracker.poll().unwrap_or_else(|_| {
                        logging::write(
                            "action=playtime_poll status=failed category=state_not_saved",
                        );
                        false
                    }),
                    Err(_) => false,
                },
                None => false,
            };

            if active {
                let _ = app.emit("playtime-tick", changed);
            }

            // ! Steam can add or switch accounts behind our back; re-read the client files every fifteen seconds (a minute when idle).
            let refresh_due = if active {
                tick.is_multiple_of(15)
            } else {
                tick.is_multiple_of(60)
            };
            if refresh_due {
                state.refresh_accounts();
                tray::refresh_menu(&app);
                let _ = app.emit("accounts-changed", ());
            }
        }
    });
}

// ! Re-measured every `CLOCK_OFFSET_TTL`: a session running for hours must not keep trusting a stale measurement.
const CLOCK_OFFSET_TTL: Duration = Duration::from_secs(15 * 60);
const CLOCK_DRIFT_THRESHOLD_SECONDS: i64 = 60;

fn start_clock_offset_loop(app: AppHandle) {
    std::thread::spawn(move || loop {
        match steam_core::authenticator::measure_time_offset() {
            Ok(offset) => {
                let authenticator = app.state::<AppState>().authenticator();
                authenticator
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .set_time_offset(offset);
                logging::write(&format!("clock: steam_offset_seconds={offset}"));
                if offset.abs() > CLOCK_DRIFT_THRESHOLD_SECONDS {
                    let _ = app.emit("clock-drift-warning", offset);
                }
            }
            Err(error) => {
                logging::write(&format!("clock: steam_offset_unavailable code={error}"));
            }
        }
        std::thread::sleep(CLOCK_OFFSET_TTL);
    });
}
