use std::sync::Mutex;
use tauri::menu::{Menu, MenuBuilder, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

use crate::state::AppState;

pub const TRAY_ID: &str = "steam-account-switcher-tray";
const ACCOUNT_PREFIX: &str = "account:";

/// The tray menu handle, kept because Tauri can only replace a tray menu on
/// Windows and macOS: on Linux the existing menu has to be edited in place.
pub struct TrayMenu(pub Mutex<Option<Menu<tauri::Wry>>>);

/// Creates the tray icon and reports whether it exists: without it the window
/// must not hide, or the application would be unreachable.
pub fn install(app: &AppHandle) -> tauri::Result<bool> {
    let menu = build_menu(app)?;
    app.manage(TrayMenu(Mutex::new(Some(menu.clone()))));

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("Steam Account Switcher")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| handle_menu_event(app, event.id().as_ref()))
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                crate::show_main_window(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }

    builder.build(app)?;
    Ok(true)
}

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let locale = app
        .state::<AppState>()
        .lock()
        .settings
        .data()
        .language
        .resolved();
    MenuBuilder::new(app)
        .item(&MenuItem::with_id(
            app,
            "open",
            rust_i18n::t!("tray_open", locale = locale),
            true,
            None::<&str>,
        )?)
        .item(&MenuItem::with_id(
            app,
            "refresh",
            rust_i18n::t!("tray_refresh", locale = locale),
            true,
            None::<&str>,
        )?)
        .separator()
        .item(&MenuItem::with_id(
            app,
            "empty",
            rust_i18n::t!("tray_no_accounts", locale = locale),
            false,
            None::<&str>,
        )?)
        .separator()
        .item(&MenuItem::with_id(
            app,
            "quit",
            rust_i18n::t!("tray_quit", locale = locale),
            true,
            None::<&str>,
        )?)
        .build()
}

/// Queues the menu update on the main thread instead of waiting for it: each
/// menu call blocks until the main thread runs it, so a caller that holds a
/// lock (the playtime thread, or a command holding AppState) would deadlock
/// against a main-thread handler that is waiting for the same lock.
pub fn refresh_menu(app: &AppHandle) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || update_menu(&handle));
}

fn update_menu(app: &AppHandle) {
    let Some(holder) = app.try_state::<TrayMenu>() else {
        return;
    };
    let Ok(guard) = holder.0.lock() else {
        return;
    };
    let Some(menu) = guard.as_ref() else {
        return;
    };
    let locale = app
        .state::<AppState>()
        .lock()
        .settings
        .data()
        .language
        .resolved();
    let Ok(items) = menu.items() else {
        return;
    };

    for (index, item) in items.iter().enumerate().rev() {
        let id = item.id().as_ref().to_string();
        if let tauri::menu::MenuItemKind::MenuItem(item) = item {
            let key = match id.as_str() {
                "open" => Some("tray_open"),
                "refresh" => Some("tray_refresh"),
                "quit" => Some("tray_quit"),
                _ => None,
            };
            if let Some(key) = key {
                let _ = item.set_text(rust_i18n::t!(key, locale = locale));
            }
        }
        if id.starts_with(ACCOUNT_PREFIX) || id == "empty" {
            let _ = menu.remove_at(index);
        }
    }

    let (accounts, auto_login_user, masked) = {
        let state = app.state::<AppState>();
        let inner = state.lock();
        (
            inner.accounts.clone(),
            inner.auto_login_user.clone(),
            inner.masked,
        )
    };

    let mut position = 3; // after Open, Refresh and the first separator
    if accounts.is_empty() {
        if let Ok(item) = MenuItem::with_id(
            app,
            "empty",
            rust_i18n::t!("tray_no_accounts", locale = locale),
            false,
            None::<&str>,
        ) {
            let _ = menu.insert(&item, position);
        }
        return;
    }

    for (index, account) in accounts.iter().enumerate() {
        let label = account_label(account, index + 1, masked, locale);
        let id = format!("{ACCOUNT_PREFIX}{}", account.steam_id);
        let is_current = is_current_account(account, &auto_login_user);

        let Ok(item) =
            tauri::menu::CheckMenuItem::with_id(app, id, label, true, is_current, None::<&str>)
        else {
            continue;
        };

        if menu.insert(&item, position).is_ok() {
            position += 1;
        }
    }
}

/// An account without a login name never matches: an empty AutoLoginUser would
/// otherwise mark every such entry as the current one.
fn is_current_account(account: &steam_core::SteamAccount, auto_login_user: &str) -> bool {
    !account.account_name.is_empty() && account.account_name.eq_ignore_ascii_case(auto_login_user)
}

fn account_label(
    account: &steam_core::SteamAccount,
    index: usize,
    masked: bool,
    locale: &str,
) -> String {
    if masked {
        return if !account.persona_name.is_empty() && account.persona_name != account.account_name {
            account.persona_name.clone()
        } else {
            rust_i18n::t!("tray_account", locale = locale, index = index).to_string()
        };
    }

    if !account.account_name.is_empty() && account.account_name != account.display_name() {
        format!("{} ({})", account.display_name(), account.account_name)
    } else {
        account.display_name().to_string()
    }
}

fn handle_menu_event(app: &AppHandle, id: &str) {
    match id {
        "open" => crate::show_main_window(app),
        "refresh" => {
            app.state::<AppState>().refresh_accounts();
            refresh_menu(app);
            let _ = app.emit("accounts-changed", ());
        }
        "quit" => app.exit(0),
        other if other.starts_with(ACCOUNT_PREFIX) => {
            let steam_id = other[ACCOUNT_PREFIX.len()..].to_string();
            if app.state::<AppState>().is_switching() {
                crate::show_main_window(app);
                return;
            }

            // The tray never switches on its own: it brings the window forward
            // and lets the interface ask for a confirmation, so both paths
            // behave exactly the same.
            crate::show_main_window(app);
            let _ = app.emit("switch-requested", steam_id);
        }
        _ => {}
    }
}
