#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod auth_commands;
mod base64;
mod clipboard;
mod commands;
mod locale;
mod log_commands;
mod logging;
mod retained_accounts;
mod settings;
mod state;
mod tracking;
mod tray;

use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager, WindowEvent};

use crate::state::AppState;

rust_i18n::i18n!("locales", fallback = "en");

fn main() {
    prepare_linux_display();
    // Before the state is built, so a settings or vault file that cannot be read
    // leaves a trace instead of failing silently.
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
        .manage(AppState::new())
        .manage(commands::updates::PendingUpdate::new())
        .invoke_handler(tauri::generate_handler![
            log_commands::logs_snapshot,
            log_commands::logs_open_directory,
            clipboard::clipboard_write_text,
            commands::snapshots::snapshot,
            commands::accounts::account_login_name,
            commands::snapshots::playtime_snapshot,
            commands::snapshots::refresh,
            commands::snapshots::reload_installations,
            commands::preferences::select_installation,
            commands::preferences::set_masked,
            commands::preferences::set_steam_ids_visible,
            commands::preferences::update_settings,
            tracking::set_tracking_enabled,
            tracking::clear_playtime_data,
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
            commands::preferences::set_folder_color,
            commands::preferences::set_folder_icon,
            commands::preferences::set_window_active,
            commands::preferences::app_info,
            commands::accounts::add_steam_account,
            commands::updates::check_for_update,
            commands::updates::download_update,
            commands::updates::apply_update,
            commands::accounts::running_steam_games,
            commands::accounts::switch_account,
            commands::accounts::remove_account,
            commands::accounts::forget_saved_account,
            commands::accounts::restart_steam,
            commands::accounts::set_rank,
            commands::reports::account_report,
            commands::reports::machine_report,
            commands::preferences::autostart_status,
            commands::preferences::set_autostart,
            auth_commands::vault::authenticator_status,
            auth_commands::vault::authenticator_accounts,
            auth_commands::vault::authenticator_unlock,
            auth_commands::vault::authenticator_lock,
            auth_commands::vault::authenticator_set_passphrase,
            auth_commands::guard::steam_guard_code,
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
            let previous_panic_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                // Panic payloads can also contain secrets. Keep the file safe.
                logging::write("runtime: panic");
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
            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                WindowEvent::CloseRequested { api, .. } => {
                    let state = window.app_handle().state::<AppState>();
                    let close_to_tray = state.lock().settings.data().close_to_tray;
                    // Hiding without a tray icon would leave the app unreachable.
                    if state.tray_available() && close_to_tray {
                        api.prevent_close();
                        state.set_window_active(false);
                        let _ = window.hide();
                    } else {
                        window.app_handle().exit(0);
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

/// Gives the window its identity: icon, `WM_CLASS` on X11 and the Wayland
/// `app_id`. Without them a compositor has nothing to pair with the installed
/// `.desktop` entry, so the task bar shows a generic placeholder icon.
fn configure_window(window: &tauri::WebviewWindow) {
    if let Ok(icon) = tauri::image::Image::from_bytes(include_bytes!("../icons/128x128.png")) {
        let _ = window.set_icon(icon);
    }

    #[cfg(target_os = "linux")]
    {
        glib::set_prgname(Some("accountswitcher"));
        glib::set_application_name("Steam Account Switcher");
        gtk::Window::set_default_icon_name("accountswitcher");
    }
}

/// GTK picks Wayland or X11 on its own. WebKitGTK's DMABUF renderer however
/// fails on Wayland with the NVIDIA driver, so it is disabled in that case only.
#[cfg(target_os = "linux")]
fn prepare_linux_display() {
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

pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();

        // Wayland refuses focus changes asked by an unfocused application, so a
        // tray click would only raise the window: present() asks properly.
        #[cfg(target_os = "linux")]
        {
            if let Ok(gtk_window) = window.gtk_window() {
                use gtk::prelude::GtkWindowExt;
                gtk_window.present();
            }
        }

        let _ = window.set_always_on_top(true);
        let handle = window.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(700));
            let _ = handle.set_always_on_top(false);
            let _ = handle.set_focus();
        });
    }
}

fn start_playtime_loop(app: AppHandle) {
    std::thread::spawn(move || {
        let mut tick: u64 = 0;
        loop {
            std::thread::sleep(Duration::from_millis(1000));
            tick += 1;

            let state = app.state::<AppState>();

            // The vault locks itself once its configured unlocked time elapsed.
            // Kept before the idle skip so the deadline stays exact in the background.
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

            // Steam can add or switch accounts behind our back; re-read the
            // client files every fifteen seconds (a minute when idle).
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
