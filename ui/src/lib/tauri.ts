/* Typed access to the Tauri commands and events the interface uses. */

import type {
  AppInfo,
  PlaytimeSnapshot,
  Report,
  Snapshot,
  UpdateInfo,
  UpdateProgress,
} from "./types";

export type UnlistenFn = () => void;

interface TauriGlobal {
  core: {
    invoke(command: string, args?: Record<string, unknown>): Promise<unknown>;
  };
  event: {
    listen(name: string, handler: (event: { payload: unknown }) => void): Promise<UnlistenFn>;
  };
}

declare global {
  interface Window {
    __TAURI__?: TauriGlobal;
  }
}

function bridge(): TauriGlobal {
  const api = window.__TAURI__;
  if (!api) {
    throw new Error("The Tauri bridge is unavailable.");
  }
  return api;
}

/** Every command the interface calls, with its arguments and its result. */
export interface Commands {
  snapshot: { args: void; result: Snapshot };
  playtime_snapshot: { args: void; result: PlaytimeSnapshot };
  refresh: { args: void; result: Snapshot };
  select_installation: { args: { index: number }; result: Snapshot };
  set_masked: { args: { masked: boolean }; result: Snapshot };
  set_steam_ids_visible: { args: { visible: boolean }; result: Snapshot };
  update_settings: {
    args: { showInstallationSelector: boolean; showMachinePlaytime: boolean };
    result: Snapshot;
  };
  set_installation_hidden: { args: { root: string; hidden: boolean }; result: Snapshot };
  add_custom_installation: { args: { path: string }; result: Snapshot };
  remove_custom_installation: { args: { root: string }; result: Snapshot };
  create_folder: { args: { name: string }; result: Snapshot };
  rename_folder: { args: { id: string; name: string }; result: Snapshot };
  delete_folder: { args: { id: string }; result: Snapshot };
  assign_folder: { args: { steamId: string; folderId: string | null }; result: Snapshot };
  set_account_color: { args: { steamId: string; color: string | null }; result: Snapshot };
  set_rank: { args: { steamId: string; template: string; rank: string }; result: Snapshot };
  add_steam_account: { args: void; result: Snapshot };
  switch_account: { args: { steamId: string }; result: Snapshot };
  remove_account: { args: { steamId: string; cleanupUserdata: boolean }; result: Snapshot };
  account_report: { args: { steamId: string }; result: Report };
  machine_report: { args: void; result: Report };
  app_info: { args: void; result: AppInfo };
  set_window_active: { args: { active: boolean }; result: void };
  check_for_update: { args: void; result: UpdateInfo | null };
  download_update: { args: void; result: string };
  apply_update: { args: void; result: void };
  autostart_status: { args: void; result: boolean };
  set_autostart: { args: { enabled: boolean }; result: boolean };
}

export function invoke<K extends keyof Commands>(
  command: K,
  args?: Commands[K]["args"],
): Promise<Commands[K]["result"]> {
  return bridge().core.invoke(command, args as Record<string, unknown> | undefined) as Promise<
    Commands[K]["result"]
  >;
}

/** Calls a plugin command: the typed `invoke` above only covers the commands
 *  the application itself registers. */
export function invokePlugin(command: string, args?: Record<string, unknown>): Promise<unknown> {
  return bridge().core.invoke(command, args);
}

/** Events the Rust side emits while the application runs. */
export interface Events {
  "playtime-tick": boolean;
  "accounts-changed": void;
  "switch-requested": string;
  "update-progress": UpdateProgress;
  "update-downloaded": string;
}

export function listen<K extends keyof Events>(
  name: K,
  handler: (event: { payload: Events[K] }) => void,
): Promise<UnlistenFn> {
  return bridge().event.listen(name, handler as (event: { payload: unknown }) => void);
}
