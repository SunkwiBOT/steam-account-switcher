use std::path::Path;
use steam_core::error_code::ErrorCode;

use steam_core::authenticator::{
    begin_password_login, begin_qr_login, confirmation_action, confirmations, finish_login, qr_svg,
    remove_authenticator, submit_login_code, AuthError, Authenticator, CodeInfo, ConfirmationItem,
    GuardKind, LinkStart, LoginBegin, LoginResult, StoredAuthenticator, VaultStatus,
};
use tauri::State;

use crate::commands::build_snapshot;
use crate::commands::SnapshotDto;
use crate::state::AppState;

pub mod backup;
pub mod guard;
pub mod link;
pub mod login;
pub mod vault;

use vault::vault_snapshot;
