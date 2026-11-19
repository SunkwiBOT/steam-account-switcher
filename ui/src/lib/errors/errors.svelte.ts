import type { ErrorCode } from "../bindings/ErrorCode";
import { t } from "../i18n/i18n.svelte";

// ! Client-only validation codes with no Rust `ErrorCode` counterpart; unmapped codes fall back to `UNKNOWN_ERROR`.
export type UiErrorCode =
  | ErrorCode
  | "PASSWORD_MISMATCH"
  | "BACKUP_FILE_REQUIRED"
  | "STEAM_PASSWORD_REQUIRED"
  | "GUARD_CODE_REQUIRED"
  | "REVOCATION_CODE_REQUIRED";
const messages = {
  BACKUP_EMPTY: () => t.error_backup_empty(),
  BACKUP_EXPORT_FAILED: () => t.error_backup_export_failed(),
  BACKUP_TARGET_UNSAFE: () => t.error_backup_target_unsafe(),
  UNSUPPORTED_BACKUP_FORMAT: () => t.error_unsupported_backup_format(),

  VAULT_PASSWORD_REQUIRED: () => t.error_vault_password_required(),
  PASSWORD_MISMATCH: () => t.error_password_mismatch(),
  BACKUP_FILE_REQUIRED: () => t.error_backup_file_required(),
  STEAM_PASSWORD_REQUIRED: () => t.error_steam_password_required(),
  GUARD_CODE_REQUIRED: () => t.error_guard_code_required(),
  REVOCATION_CODE_REQUIRED: () => t.error_revocation_code_required(),

  UNKNOWN_ERROR: () => t.error_unknown_error(),
  VAULT_LOCKED: () => t.error_vault_locked(),
  AUTHENTICATOR_MISSING: () => t.error_authenticator_missing(),
  WRONG_VAULT_PASSWORD: () => t.error_wrong_vault_password(),
  INVALID_VAULT: () => t.error_invalid_vault(),
  INVALID_BACKUP: () => t.error_invalid_backup(),
  INVALID_BACKUP_MANIFEST: () => t.error_invalid_backup_manifest(),
  BACKUP_DECRYPTION_FAILED: () => t.error_backup_decryption_failed(),
  INVALID_AUTHENTICATOR_DATA: () => t.error_invalid_authenticator_data(),
  SESSION_EXPIRED: () => t.error_session_expired(),
  SIGN_IN_REQUIRED: () => t.error_sign_in_required(),
  AUTHENTICATOR_BUSY: () => t.error_authenticator_busy(),
  STEAM_ERROR: () => t.error_steam_error(),
  NETWORK_ERROR: () => t.error_network_error(),
  IO_ERROR: () => t.error_io_error(),
  FILE_PERMISSION: () => t.error_file_permission(),
  INVALID_STEAM_ID: () => t.error_invalid_steam_id(),
  QR_CODE_FAILED: () => t.error_qr_code_failed(),
  LINK_NOT_STARTED: () => t.error_link_not_started(),
  LINK_FAILED: () => t.error_link_failed(),
  LINK_NOT_CONFIRMED: () => t.error_link_not_confirmed(),
  TRANSFER_START_FAILED: () => t.error_transfer_start_failed(),
  TRANSFER_NOT_STARTED: () => t.error_transfer_not_started(),
  TRANSFER_FAILED: () => t.error_transfer_failed(),
  VAULT_PASSWORD_REQUIRED_TO_LOCK: () => t.error_vault_password_required_to_lock(),
  ACCOUNT_MISMATCH: () => t.error_account_mismatch(),
  AUTHENTICATOR_ALREADY_SAVED: () => t.error_authenticator_already_saved(),
  ACTIVATION_REQUIRED: () => t.error_activation_required(),
  VAULT_ENCRYPTION_FAILED: () => t.error_vault_encryption_failed(),
  VAULT_OPEN_FAILED: () => t.error_vault_open_failed(),
  UNSUPPORTED_VAULT_FORMAT: () => t.error_unsupported_vault_format(),
  UNSUPPORTED_VAULT_PROTECTION: () => t.error_unsupported_vault_protection(),
  CURRENT_PASSWORD_REQUIRED: () => t.error_current_password_required(),
  BACKUP_PASSWORD_REQUIRED: () => t.error_backup_password_required(),
  BACKUP_MANIFEST_MISSING: () => t.error_backup_manifest_missing(),
  BACKUP_MANIFEST_MISMATCH: () => t.error_backup_manifest_mismatch(),
  BACKUP_STEAM_ID_MISSING: () => t.error_backup_steam_id_missing(),
  BACKUP_DEVICE_MISSING: () => t.error_backup_device_missing(),
  STEAM_OPERATION_BUSY: () => t.error_steam_operation_busy(),
  NO_STEAM_INSTALLATION: () => t.error_no_steam_installation(),
  STEAM_FOLDER_REQUIRED: () => t.error_steam_folder_required(),
  FOLDER_NAME_REQUIRED: () => t.error_folder_name_required(),
  PROTECTED_FOLDER: () => t.error_protected_folder(),
  UNKNOWN_FOLDER: () => t.error_unknown_folder(),
  SAVED_ACCOUNT_FOLDER: () => t.error_saved_account_folder(),
  NOTE_TOO_LONG: () => t.error_note_too_long(),
  INVALID_NOTE: () => t.error_invalid_note(),
  CONFIRMATION_REQUIRED: () => t.error_confirmation_required(),
  ACCOUNT_LOCAL: () => t.error_account_local(),
  ACCOUNT_NOT_LOCAL: () => t.error_account_not_local(),
  ACCOUNT_UNAVAILABLE: () => t.error_account_unavailable(),
  PLAYTIME_UNAVAILABLE: () => t.error_playtime_unavailable(),
  PLAYTIME_BUSY: () => t.error_playtime_busy(),
  RANK_STORE_UNAVAILABLE: () => t.error_rank_store_unavailable(),
  CLIPBOARD_UNAVAILABLE: () => t.error_clipboard_unavailable(),
  LOG_UNAVAILABLE: () => t.error_log_unavailable(),
  OPEN_FOLDER_FAILED: () => t.error_open_folder_failed(),
  OPEN_URL_FAILED: () => t.error_open_url_failed(),
  OPEN_URL_REFUSED: () => t.error_open_url_refused(),
  LINK_IN_FLIGHT: () => t.error_link_in_flight(),
  UPDATE_FAILED: () => t.error_update_failed(),
  NO_UPDATE: () => t.error_no_update(),
  NO_DOWNLOADED_UPDATE: () => t.error_no_downloaded_update(),
  UPDATE_PORTABLE_UNSUPPORTED: () => t.error_update_portable_unsupported(),
  WORKER_FAILED: () => t.error_worker_failed(),
  SETTINGS_SAVE_FAILED: () => t.error_settings_save_failed(),
  AUTOSTART_FAILED: () => t.error_autostart_failed(),
  INVALID_STEAM_FILE: () => t.error_invalid_steam_file(),
  STEAM_START_FAILED: () => t.error_steam_start_failed(),
  STEAM_STOP_FAILED: () => t.error_steam_stop_failed(),
  UNSAFE_DATA_REMOVAL: () => t.error_unsafe_data_removal(),
  STEAM_REGISTRY_FAILED: () => t.error_steam_registry_failed(),
  FOLDER_NOT_FOUND: () => t.error_folder_not_found(),
  NOT_STEAM_FOLDER: () => t.error_not_steam_folder(),
  STEAM_NEVER_SIGNED_IN: () => t.error_steam_never_signed_in(),
  INVALID_LANGUAGE: () => t.error_invalid_language(),
  WRONG_STEAM_PASSWORD: () => t.error_wrong_steam_password(),
  WRONG_GUARD_CODE: () => t.error_wrong_guard_code(),
  STEAM_RATE_LIMITED: () => t.error_steam_rate_limited(),
  DATABASE_VERSION_MISMATCH: () => t.error_database_version_mismatch(),
  VAULT_REQUIRES_NEWER_VERSION: () => t.error_vault_requires_newer_version(),

  STEAM_INSTALLATION_READY: () => t.steam_status_installation_ready(),
} satisfies Partial<Record<UiErrorCode, () => string>>;

type KnownErrorCode = keyof typeof messages;

const isKnownErrorCode = (value: string): value is KnownErrorCode => value in messages;

export const resolveErrorCode = (value: unknown): KnownErrorCode => {
  const code = value instanceof Error ? value.message : value;
  return typeof code === "string" && isKnownErrorCode(code) ? code : "UNKNOWN_ERROR";
};

export const isVaultRequiresNewerVersion = (value: unknown): boolean =>
  resolveErrorCode(value) === "VAULT_REQUIRES_NEWER_VERSION";

export const errorMessage = (value: unknown): string => messages[resolveErrorCode(value)]();

export const vaultErrorHint = (value: unknown): string =>
  isVaultRequiresNewerVersion(value)
    ? t.vault_requires_newer_version_hint()
    : t.vault_restore_hint();
