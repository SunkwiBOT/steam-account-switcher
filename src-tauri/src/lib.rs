// ! Exposes the application's modules as a library so the `export-bindings` example can reach the DTOs and preference enums without duplicating them.

pub mod auth_commands;
pub mod avatars;
pub mod catalog;
pub mod commands;
pub mod locale;
pub mod logging;
pub mod open_url;
pub mod retained_accounts;
pub mod settings;
pub mod state;
pub mod tray;

// ! Called from the tray, the single-instance guard and the window close handler, so it lives at the crate root reachable from all of them.
pub fn show_main_window(app: &tauri::AppHandle) {
    use tauri::Manager;

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();

        // ! Wayland refuses focus changes from an unfocused application, so a tray click would only raise the window: present() asks properly.
        #[cfg(target_os = "linux")]
        {
            if let Ok(gtk_window) = window.gtk_window() {
                use gtk::prelude::GtkWindowExt;
                gtk_window.present();
            }
        }
    }
}
