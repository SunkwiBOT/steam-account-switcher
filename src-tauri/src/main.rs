#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//! Steam Account Switcher: Tauri shell around the `steam-core` crate.

mod base64;
mod commands;
mod logging;
mod settings;
mod state;
mod tray;

use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager, WindowEvent};

use crate::state::AppState;

fn main() {
    prepare_linux_display();

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
        .manage(AppState::new())
        .manage(commands::PendingUpdate::new())
        .invoke_handler(tauri::generate_handler![
            commands::snapshot,
            commands::playtime_snapshot,
            commands::refresh,
            commands::reload_installations,
            commands::select_installation,
            commands::set_masked,
            commands::set_steam_ids_visible,
            commands::update_settings,
            commands::set_installation_hidden,
            commands::add_custom_installation,
            commands::remove_custom_installation,
            commands::create_folder,
            commands::rename_folder,
            commands::delete_folder,
            commands::assign_folder,
            commands::set_account_color,
            commands::set_window_active,
            commands::app_info,
            commands::open_repository,
            commands::add_steam_account,
            commands::check_for_update,
            commands::download_update,
            commands::apply_update,
            commands::switch_account,
            commands::remove_account,
            commands::restart_steam,
            commands::set_rank,
            commands::account_report,
            commands::machine_report,
            commands::autostart_status,
            commands::set_autostart,
        ])
        .setup(|app| {
            logging::init(settings::config_dir());
            logging::write(&format!(
                "start: {} {}",
                app.package_info().name,
                app.package_info().version
            ));
            if let Some(window) = app.get_webview_window("main") {
                configure_window(&window);
            }
            let tray_available = tray::install(app.handle()).unwrap_or_else(|error| {
                eprintln!("Unable to create the tray icon: {error}");
                false
            });
            app.state::<AppState>().set_tray_available(tray_available);
            if tray_available {
                tray::refresh_menu(app.handle());
            }
            start_playtime_loop(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                // The switcher keeps running in the tray.
                // Without a tray icon hiding the window would leave the
                // application unreachable, so closing really quits.
                WindowEvent::CloseRequested { api, .. } => {
                    if window.app_handle().state::<AppState>().tray_available() {
                        api.prevent_close();
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
        glib::set_prgname(Some("steam-account-switcher"));
        glib::set_application_name("Steam Account Switcher");
        gtk::Window::set_default_icon_name("steam-account-switcher");
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

/// Brings the main window back to the front, used by the tray and by a second
/// application launch.
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

/// Polls Steam's game process log once per second and keeps the UIs in sync.
fn start_playtime_loop(app: AppHandle) {
    std::thread::spawn(move || {
        let mut tick: u64 = 0;
        loop {
            std::thread::sleep(Duration::from_millis(1000));
            tick += 1;

            let state = app.state::<AppState>();
            let active = state.window_active();

            // Full speed in the foreground, one sample every five seconds in
            // the background: local playtime does not need more, and the
            // application stays out of the way while the user plays.
            if !active && !tick.is_multiple_of(5) {
                continue;
            }

            let changed = match state.tracker() {
                Some(tracker) => match tracker.lock() {
                    Ok(mut tracker) => tracker.poll(),
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
