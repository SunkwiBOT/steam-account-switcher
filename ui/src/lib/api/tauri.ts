import type { Language } from "../i18n/i18n.constants";
import type { GuardKind } from "../steam-guard/steam-guard.constants";
import type { SearchScope, UpdateInterval, VaultLockInterval } from "../state/state.constants";
import { ErrorApiMessage } from "../errors/errors.constants";

import type {
  AppInfoType,
  BackupImportType,
  CodeInfoType,
  ConfirmationItemType,
  LinkStartType,
  LoginBeginType,
  LoginResultType,
  LogSnapshotType,
  PlaytimeSnapshotType,
  ReportType,
  SnapshotType,
  StoredAuthenticatorType,
  UpdateInfoType,
  UpdateProgressType,
  VaultStatusType,
} from "../types";

export type UnlistenFnType = () => void;

type TauriGlobalType = {
  core: {
    invoke(command: string, args?: Record<string, unknown>): Promise<unknown>;
  };
  event: {
    listen(name: string, handler: (event: { payload: unknown }) => void): Promise<UnlistenFnType>;
  };
};

declare global {
  interface Window {
    __TAURI__?: TauriGlobalType;
  }
}

const bridge = (): TauriGlobalType => {
  const api = window.__TAURI__;
  if (!api) {
    throw new Error(ErrorApiMessage.UNKNOWN_ERROR);
  }
  return api;
};

export type CommandsType = {
  logs_snapshot: { args: { previous: boolean }; result: LogSnapshotType };
  logs_open_directory: { args: void; result: void };
  clipboard_write_text: { args: { text: string }; result: void };
  snapshot: { args: void; result: SnapshotType };
  account_login_name: { args: { steamId: string }; result: string };
  playtime_snapshot: { args: void; result: PlaytimeSnapshotType };
  refresh: { args: void; result: SnapshotType };
  select_installation: { args: { index: number }; result: SnapshotType };
  set_masked: { args: { masked: boolean }; result: SnapshotType };
  set_steam_ids_visible: { args: { visible: boolean }; result: SnapshotType };
  set_language: { args: { language: Language }; result: SnapshotType };
  set_close_to_tray: { args: { closeToTray: boolean }; result: SnapshotType };
  set_update_interval: { args: { interval: UpdateInterval }; result: SnapshotType };
  set_search_scope: { args: { scope: SearchScope }; result: SnapshotType };
  set_vault_lock_interval: { args: { interval: VaultLockInterval }; result: SnapshotType };
  update_settings: { args: { showInstallationSelector: boolean }; result: SnapshotType };
  set_tracking_enabled: { args: { enabled: boolean }; result: SnapshotType };
  clear_playtime_data: { args: { confirmed: boolean }; result: SnapshotType };
  set_installation_hidden: { args: { root: string; hidden: boolean }; result: SnapshotType };
  add_custom_installation: { args: { path: string }; result: SnapshotType };
  remove_custom_installation: { args: { root: string }; result: SnapshotType };
  create_folder: {
    args: { name: string; color?: string | null; icon?: string | null };
    result: SnapshotType;
  };
  update_folder: {
    args: { id: string; name: string; color: string | null; icon: string | null };
    result: SnapshotType;
  };
  delete_folder: { args: { id: string }; result: SnapshotType };
  assign_folder: { args: { steamId: string; folderId: string | null }; result: SnapshotType };
  set_account_color: { args: { steamId: string; color: string | null }; result: SnapshotType };
  set_folder_color: { args: { id: string; color: string | null }; result: SnapshotType };
  set_folder_icon: { args: { id: string; icon: string | null }; result: SnapshotType };
  set_rank: { args: { steamId: string; template: string; rank: string }; result: SnapshotType };
  add_steam_account: { args: void; result: SnapshotType };
  running_steam_games: { args: void; result: string[] };
  switch_account: { args: { steamId: string }; result: SnapshotType };
  remove_account: {
    args: { steamId: string; cleanupUserdata: boolean; confirmed: boolean };
    result: SnapshotType;
  };
  forget_saved_account: { args: { steamId: string; confirmed: boolean }; result: SnapshotType };
  account_report: { args: { steamId: string }; result: ReportType };
  machine_report: { args: void; result: ReportType };
  app_info: { args: void; result: AppInfoType };
  set_window_active: { args: { active: boolean }; result: void };
  check_for_update: { args: void; result: UpdateInfoType | null };
  download_update: { args: void; result: string };
  apply_update: { args: { version: string }; result: void };
  autostart_status: { args: void; result: boolean };
  set_autostart: { args: { enabled: boolean }; result: boolean };
  authenticator_status: { args: void; result: VaultStatusType };
  authenticator_accounts: { args: void; result: StoredAuthenticatorType[] };
  authenticator_unlock: { args: { passphrase: string }; result: SnapshotType };
  authenticator_lock: { args: void; result: SnapshotType };
  authenticator_set_passphrase: {
    args: { passphrase: string | null; currentPassword?: string | null };
    result: SnapshotType;
  };
  steam_guard_code: { args: { steamId: string }; result: CodeInfoType };
  steam_guard_revocation_code: { args: { steamId: string }; result: string };
  steam_guard_import: {
    args: { steamId?: string | null; path: string; passkey?: string | null };
    result: SnapshotType;
  };
  steam_guard_export_backup: {
    args: { steamId: string | null; path: string; password: string | null };
    result: number;
  };
  steam_guard_import_backup: {
    args: { path: string; password: string | null };
    result: BackupImportType;
  };
  steam_guard_confirmations: { args: { steamId: string }; result: ConfirmationItemType[] };
  steam_guard_confirmation_action: {
    args: { steamId: string; id: string; nonce: string; accept: boolean };
    result: void;
  };
  steam_guard_remove: {
    args: { steamId: string; revocationCode?: string | null };
    result: SnapshotType;
  };
  steam_guard_login_password: {
    args: { steamId: string; password: string };
    result: LoginBeginType;
  };
  steam_guard_login_qr: { args: void; result: LoginBeginType };
  steam_guard_login_finish: {
    args: {
      sessionId: string;
      guardKind?: GuardKind | null;
      code?: string | null;
      steamId: string;
    };
    result: LoginResultType;
  };
  steam_guard_login_cancel: { args: { sessionId: string }; result: void };
  steam_guard_link_start: { args: { steamId: string }; result: LinkStartType };
  steam_guard_link_finalize: {
    args: { sessionId: string; confirmCode: string };
    result: SnapshotType;
  };
  steam_guard_link_transfer_start: { args: { sessionId: string }; result: void };
  steam_guard_link_transfer_finish: {
    args: { sessionId: string; smsCode: string };
    result: SnapshotType;
  };
  steam_guard_link_revoke: { args: { sessionId: string; revocationCode: string }; result: void };
  steam_guard_link_cancel: { args: { sessionId: string }; result: void };
};

export const invoke = <K extends keyof CommandsType>(
  command: K,
  args?: CommandsType[K]["args"],
): Promise<CommandsType[K]["result"]> => {
  return bridge().core.invoke(command, args as Record<string, unknown> | undefined) as Promise<
    CommandsType[K]["result"]
  >;
};

/** Calls a plugin command: the typed `invoke` above only covers the commands
 *  the application itself registers. */
export const invokePlugin = (command: string, args?: Record<string, unknown>): Promise<unknown> => {
  return bridge().core.invoke(command, args);
};

export type EventsType = {
  "playtime-tick": boolean;
  "accounts-changed": void;
  "vault-locked": void;
  "switch-requested": string;
  "update-progress": UpdateProgressType;
  "update-downloaded": string;
};

export const listen = <K extends keyof EventsType>(
  name: K,
  handler: (event: { payload: EventsType[K] }) => void,
): Promise<UnlistenFnType> => {
  return bridge().event.listen(name, handler as (event: { payload: unknown }) => void);
};
