import { ErrorApiMessage } from "./errors.constants";
import { t } from "../i18n/i18n.svelte";

const codes = new Set<string>(Object.values(ErrorApiMessage));

const isErrorApiMessage = (value: string): value is ErrorApiMessage => codes.has(value);

export const resolveErrorCode = (value: unknown): ErrorApiMessage => {
  const code = value instanceof Error ? value.message : value;
  return typeof code === "string" && isErrorApiMessage(code) ? code : ErrorApiMessage.UNKNOWN_ERROR;
};

const messages: Record<ErrorApiMessage, () => string> = {
  [ErrorApiMessage.BACKUP_EMPTY]: () => t.error_backup_empty(),
  [ErrorApiMessage.BACKUP_EXPORT_FAILED]: () => t.error_backup_export_failed(),
  [ErrorApiMessage.BACKUP_TARGET_UNSAFE]: () => t.error_backup_target_unsafe(),
  [ErrorApiMessage.UNSUPPORTED_BACKUP_FORMAT]: () => t.error_unsupported_backup_format(),

  [ErrorApiMessage.VAULT_PASSWORD_REQUIRED]: () => t.error_vault_password_required(),
  [ErrorApiMessage.PASSWORD_MISMATCH]: () => t.error_password_mismatch(),
  [ErrorApiMessage.BACKUP_FILE_REQUIRED]: () => t.error_backup_file_required(),
  [ErrorApiMessage.STEAM_PASSWORD_REQUIRED]: () => t.error_steam_password_required(),
  [ErrorApiMessage.GUARD_CODE_REQUIRED]: () => t.error_guard_code_required(),
  [ErrorApiMessage.REVOCATION_CODE_REQUIRED]: () => t.error_revocation_code_required(),

  [ErrorApiMessage.UNKNOWN_ERROR]: () => t.error_unknown_error(),
  [ErrorApiMessage.VAULT_LOCKED]: () => t.error_vault_locked(),
  [ErrorApiMessage.AUTHENTICATOR_MISSING]: () => t.error_authenticator_missing(),
  [ErrorApiMessage.WRONG_VAULT_PASSWORD]: () => t.error_wrong_vault_password(),
  [ErrorApiMessage.INVALID_VAULT]: () => t.error_invalid_vault(),
  [ErrorApiMessage.INVALID_BACKUP]: () => t.error_invalid_backup(),
  [ErrorApiMessage.INVALID_BACKUP_MANIFEST]: () => t.error_invalid_backup_manifest(),
  [ErrorApiMessage.BACKUP_DECRYPTION_FAILED]: () => t.error_backup_decryption_failed(),
  [ErrorApiMessage.INVALID_AUTHENTICATOR_DATA]: () => t.error_invalid_authenticator_data(),
  [ErrorApiMessage.SESSION_EXPIRED]: () => t.error_session_expired(),
  [ErrorApiMessage.SIGN_IN_REQUIRED]: () => t.error_sign_in_required(),
  [ErrorApiMessage.AUTHENTICATOR_BUSY]: () => t.error_authenticator_busy(),
  [ErrorApiMessage.STEAM_ERROR]: () => t.error_steam_error(),
  [ErrorApiMessage.NETWORK_ERROR]: () => t.error_network_error(),
  [ErrorApiMessage.IO_ERROR]: () => t.error_io_error(),
  [ErrorApiMessage.FILE_PERMISSION]: () => t.error_file_permission(),
  [ErrorApiMessage.INVALID_STEAM_ID]: () => t.error_invalid_steam_id(),
  [ErrorApiMessage.QR_CODE_FAILED]: () => t.error_qr_code_failed(),
  [ErrorApiMessage.LINK_NOT_STARTED]: () => t.error_link_not_started(),
  [ErrorApiMessage.LINK_FAILED]: () => t.error_link_failed(),
  [ErrorApiMessage.LINK_NOT_CONFIRMED]: () => t.error_link_not_confirmed(),
  [ErrorApiMessage.TRANSFER_START_FAILED]: () => t.error_transfer_start_failed(),
  [ErrorApiMessage.TRANSFER_NOT_STARTED]: () => t.error_transfer_not_started(),
  [ErrorApiMessage.TRANSFER_FAILED]: () => t.error_transfer_failed(),
  [ErrorApiMessage.VAULT_PASSWORD_REQUIRED_TO_LOCK]: () =>
    t.error_vault_password_required_to_lock(),
  [ErrorApiMessage.ACCOUNT_MISMATCH]: () => t.error_account_mismatch(),
  [ErrorApiMessage.AUTHENTICATOR_ALREADY_SAVED]: () => t.error_authenticator_already_saved(),
  [ErrorApiMessage.ACTIVATION_REQUIRED]: () => t.error_activation_required(),
  [ErrorApiMessage.VAULT_ENCRYPTION_FAILED]: () => t.error_vault_encryption_failed(),
  [ErrorApiMessage.VAULT_OPEN_FAILED]: () => t.error_vault_open_failed(),
  [ErrorApiMessage.UNSUPPORTED_VAULT_FORMAT]: () => t.error_unsupported_vault_format(),
  [ErrorApiMessage.UNSUPPORTED_VAULT_PROTECTION]: () => t.error_unsupported_vault_protection(),
  [ErrorApiMessage.CURRENT_PASSWORD_REQUIRED]: () => t.error_current_password_required(),
  [ErrorApiMessage.BACKUP_PASSWORD_REQUIRED]: () => t.error_backup_password_required(),
  [ErrorApiMessage.BACKUP_MANIFEST_MISSING]: () => t.error_backup_manifest_missing(),
  [ErrorApiMessage.BACKUP_MANIFEST_MISMATCH]: () => t.error_backup_manifest_mismatch(),
  [ErrorApiMessage.BACKUP_STEAM_ID_MISSING]: () => t.error_backup_steam_id_missing(),
  [ErrorApiMessage.BACKUP_DEVICE_MISSING]: () => t.error_backup_device_missing(),
  [ErrorApiMessage.STEAM_OPERATION_BUSY]: () => t.error_steam_operation_busy(),
  [ErrorApiMessage.NO_STEAM_INSTALLATION]: () => t.error_no_steam_installation(),
  [ErrorApiMessage.STEAM_FOLDER_REQUIRED]: () => t.error_steam_folder_required(),
  [ErrorApiMessage.FOLDER_NAME_REQUIRED]: () => t.error_folder_name_required(),
  [ErrorApiMessage.PROTECTED_FOLDER]: () => t.error_protected_folder(),
  [ErrorApiMessage.UNKNOWN_FOLDER]: () => t.error_unknown_folder(),
  [ErrorApiMessage.SAVED_ACCOUNT_FOLDER]: () => t.error_saved_account_folder(),
  [ErrorApiMessage.CONFIRMATION_REQUIRED]: () => t.error_confirmation_required(),
  [ErrorApiMessage.ACCOUNT_LOCAL]: () => t.error_account_local(),
  [ErrorApiMessage.ACCOUNT_NOT_LOCAL]: () => t.error_account_not_local(),
  [ErrorApiMessage.ACCOUNT_UNAVAILABLE]: () => t.error_account_unavailable(),
  [ErrorApiMessage.PLAYTIME_UNAVAILABLE]: () => t.error_playtime_unavailable(),
  [ErrorApiMessage.PLAYTIME_BUSY]: () => t.error_playtime_busy(),
  [ErrorApiMessage.RANK_STORE_UNAVAILABLE]: () => t.error_rank_store_unavailable(),
  [ErrorApiMessage.CLIPBOARD_UNAVAILABLE]: () => t.error_clipboard_unavailable(),
  [ErrorApiMessage.LOG_UNAVAILABLE]: () => t.error_log_unavailable(),
  [ErrorApiMessage.OPEN_FOLDER_FAILED]: () => t.error_open_folder_failed(),
  [ErrorApiMessage.UPDATE_FAILED]: () => t.error_update_failed(),
  [ErrorApiMessage.NO_UPDATE]: () => t.error_no_update(),
  [ErrorApiMessage.NO_DOWNLOADED_UPDATE]: () => t.error_no_downloaded_update(),
  [ErrorApiMessage.WORKER_FAILED]: () => t.error_worker_failed(),
  [ErrorApiMessage.SETTINGS_SAVE_FAILED]: () => t.error_settings_save_failed(),
  [ErrorApiMessage.AUTOSTART_FAILED]: () => t.error_autostart_failed(),
  [ErrorApiMessage.INVALID_STEAM_FILE]: () => t.error_invalid_steam_file(),
  [ErrorApiMessage.STEAM_START_FAILED]: () => t.error_steam_start_failed(),
  [ErrorApiMessage.STEAM_STOP_FAILED]: () => t.error_steam_stop_failed(),
  [ErrorApiMessage.UNSAFE_DATA_REMOVAL]: () => t.error_unsafe_data_removal(),
  [ErrorApiMessage.STEAM_REGISTRY_FAILED]: () => t.error_steam_registry_failed(),
  [ErrorApiMessage.FOLDER_NOT_FOUND]: () => t.error_folder_not_found(),
  [ErrorApiMessage.NOT_STEAM_FOLDER]: () => t.error_not_steam_folder(),
  [ErrorApiMessage.STEAM_NEVER_SIGNED_IN]: () => t.error_steam_never_signed_in(),
  [ErrorApiMessage.INVALID_LANGUAGE]: () => t.error_invalid_language(),
  [ErrorApiMessage.WRONG_STEAM_PASSWORD]: () => t.error_wrong_steam_password(),
  [ErrorApiMessage.WRONG_GUARD_CODE]: () => t.error_wrong_guard_code(),
  [ErrorApiMessage.STEAM_RATE_LIMITED]: () => t.error_steam_rate_limited(),

  [ErrorApiMessage.STEAM_INSTALLATION_READY]: () => t.steam_status_installation_ready(),
};

export const errorMessage = (value: unknown): string => messages[resolveErrorCode(value)]();
