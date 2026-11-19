import type { AppInfoDto } from "../bindings/AppInfoDto";
import type { BackupImportDto } from "../bindings/BackupImportDto";
import type { CodeInfo } from "../bindings/CodeInfo";
import type { ConfirmationItem } from "../bindings/ConfirmationItem";
import type { GuardKind } from "../bindings/GuardKind";
import type { Language } from "../bindings/Language";
import type { LinkStart } from "../bindings/LinkStart";
import type { LoginBegin } from "../bindings/LoginBegin";
import type { LoginResult } from "../bindings/LoginResult";
import type { LogSnapshot } from "../bindings/LogSnapshot";
import type { PlaytimeSnapshotDto } from "../bindings/PlaytimeSnapshotDto";
import type { ReportDto } from "../bindings/ReportDto";
import type { SearchScope } from "../bindings/SearchScope";
import type { SnapshotDto } from "../bindings/SnapshotDto";
import type { SwitchResultDto } from "../bindings/SwitchResultDto";
import type { UpdateInfoDto } from "../bindings/UpdateInfoDto";
import type { UpdateInterval } from "../bindings/UpdateInterval";
import type { UpdateProgressDto } from "../bindings/UpdateProgressDto";
import type { VaultLockInterval } from "../bindings/VaultLockInterval";
import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { listen as tauriListen } from "@tauri-apps/api/event";

export type UnlistenFnType = () => void;

// ! Goes through the bundled API module, never a global, so an injected script cannot reach `invoke` from `window`.
const call = (command: string, args?: Record<string, unknown>): Promise<unknown> =>
  tauriInvoke(command, args);

export type CommandsType = {
  logs_snapshot: { args: { previous: boolean }; result: LogSnapshot };
  logs_open_directory: { args: void; result: void };
  clipboard_write_text: { args: { text: string }; result: void };
  snapshot: { args: void; result: SnapshotDto };
  account_login_name: { args: { steamId: string }; result: string };
  playtime_snapshot: { args: void; result: PlaytimeSnapshotDto };
  refresh: { args: void; result: SnapshotDto };
  select_installation: { args: { index: number }; result: SnapshotDto };
  set_masked: { args: { masked: boolean }; result: SnapshotDto };
  set_steam_ids_visible: { args: { visible: boolean }; result: SnapshotDto };
  set_vault_offer_dismissed: { args: void; result: SnapshotDto };
  set_language: { args: { language: Language }; result: SnapshotDto };
  set_close_to_tray: { args: { closeToTray: boolean }; result: SnapshotDto };
  set_update_interval: { args: { interval: UpdateInterval }; result: SnapshotDto };
  set_search_scope: { args: { scope: SearchScope }; result: SnapshotDto };
  set_vault_lock_interval: { args: { interval: VaultLockInterval }; result: SnapshotDto };
  update_settings: { args: { showInstallationSelector: boolean }; result: SnapshotDto };
  set_tracking_enabled: { args: { enabled: boolean }; result: SnapshotDto };
  clear_playtime_data: { args: void; result: SnapshotDto };
  set_installation_hidden: { args: { root: string; hidden: boolean }; result: SnapshotDto };
  add_custom_installation: { args: { path: string }; result: SnapshotDto };
  remove_custom_installation: { args: { root: string }; result: SnapshotDto };
  create_folder: {
    args: { name: string; color?: string | null; icon?: string | null };
    result: SnapshotDto;
  };
  update_folder: {
    args: { id: string; name: string; color: string | null; icon: string | null };
    result: SnapshotDto;
  };
  delete_folder: { args: { id: string }; result: SnapshotDto };
  assign_folder: { args: { steamId: string; folderId: string | null }; result: SnapshotDto };
  set_account_color: { args: { steamId: string; color: string | null }; result: SnapshotDto };
  set_account_note: { args: { steamId: string; note: string }; result: SnapshotDto };
  set_folder_color: { args: { id: string; color: string | null }; result: SnapshotDto };
  set_folder_icon: { args: { id: string; icon: string | null }; result: SnapshotDto };
  set_rank: { args: { steamId: string; template: string; rank: string }; result: SnapshotDto };
  add_steam_account: { args: void; result: SnapshotDto };
  running_steam_games: { args: void; result: string[] };
  switch_account: { args: { steamId: string }; result: SwitchResultDto };
  remove_account: {
    args: { steamId: string; cleanupUserdata: boolean };
    result: SnapshotDto;
  };
  forget_saved_account: { args: { steamId: string }; result: SnapshotDto };
  account_report: { args: { steamId: string }; result: ReportDto };
  machine_report: { args: void; result: ReportDto };
  app_info: { args: void; result: AppInfoDto };
  open_repository: { args: void; result: void };
  set_window_active: { args: { active: boolean }; result: void };
  check_for_update: { args: void; result: UpdateInfoDto | null };
  download_update: { args: void; result: string };
  apply_update: { args: { version: string }; result: void };
  autostart_status: { args: void; result: boolean };
  set_autostart: { args: { enabled: boolean }; result: boolean };
  authenticator_unlock: { args: { passphrase: string }; result: SnapshotDto };
  authenticator_lock: { args: void; result: SnapshotDto };
  authenticator_set_passphrase: {
    args: { passphrase: string | null; currentPassword?: string | null };
    result: SnapshotDto;
  };
  steam_guard_code: { args: { steamId: string }; result: CodeInfo };
  steam_guard_time_offset: { args: void; result: number };
  steam_guard_revocation_code: { args: { steamId: string }; result: string };
  steam_guard_import: {
    args: { steamId?: string | null; passkey?: string | null };
    result: SnapshotDto | null;
  };
  steam_guard_export_backup: {
    args: { steamId: string | null; password: string | null };
    result: number | null;
  };
  steam_guard_import_backup: {
    args: { password: string | null };
    result: BackupImportDto | null;
  };
  steam_guard_confirmations: { args: { steamId: string }; result: ConfirmationItem[] };
  steam_guard_confirmation_action: {
    args: { steamId: string; id: string; nonce: string; accept: boolean };
    result: void;
  };
  steam_guard_remove: {
    args: { steamId: string; revocationCode?: string | null };
    result: SnapshotDto;
  };
  steam_guard_login_password: {
    args: { steamId: string; password: string };
    result: LoginBegin;
  };
  steam_guard_login_qr: { args: void; result: LoginBegin };
  steam_guard_login_finish: {
    args: {
      sessionId: string;
      guardKind?: GuardKind | null;
      code?: string | null;
      steamId: string;
    };
    result: LoginResult;
  };
  steam_guard_login_cancel: { args: { sessionId: string }; result: void };
  steam_guard_link_start: { args: { steamId: string }; result: LinkStart };
  steam_guard_link_finalize: {
    args: { sessionId: string; confirmCode: string };
    result: SnapshotDto;
  };
  steam_guard_link_transfer_start: { args: { sessionId: string }; result: void };
  steam_guard_link_transfer_finish: {
    args: { sessionId: string; smsCode: string };
    result: SnapshotDto;
  };
  steam_guard_link_revoke: { args: { sessionId: string; revocationCode: string }; result: void };
  steam_guard_link_cancel: { args: { sessionId: string }; result: void };
};

export const invoke = <K extends keyof CommandsType>(
  command: K,
  args?: CommandsType[K]["args"],
): Promise<CommandsType[K]["result"]> => {
  return call(command, args as Record<string, unknown> | undefined) as Promise<
    CommandsType[K]["result"]
  >;
};
export const invokePlugin = (command: string, args?: Record<string, unknown>): Promise<unknown> => {
  return call(command, args);
};

export type EventsType = {
  "playtime-tick": boolean;
  "accounts-changed": void;
  "vault-locked": void;
  "clock-drift-warning": number;
  "switch-requested": string;
  "update-progress": UpdateProgressDto;
  "update-downloaded": string;
};

export const listen = <K extends keyof EventsType>(
  name: K,
  handler: (event: { payload: EventsType[K] }) => void,
): Promise<UnlistenFnType> => {
  return tauriListen(name, handler as (event: { payload: unknown }) => void);
};
