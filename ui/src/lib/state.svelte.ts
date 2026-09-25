/* Application state: one object the components read, and the commands that
 * feed it. Nothing else in the interface calls Tauri directly. */

import { invoke, type Commands } from "./tauri";
import type {
  Account,
  AppInfo,
  Folder,
  PlaytimeSnapshot,
  Report,
  Snapshot,
  UpdateInfo,
} from "./types";

export interface AppState {
  snapshot: Snapshot | null;
  info: AppInfo;
  search: string;
  folderId: string | null;
  openAccountId: string | null;
  switchingId: string;
  idle: boolean;
  settingsOpen: boolean;
  settingsSection: string;
  updateDialog: boolean;
  update: UpdateInfo | null;
  updateCheck: "idle" | "checking" | "success" | "error";
  updateStage: "" | "downloading" | "downloaded";
  updateVersion: string;
  progress: { downloaded: number; total: number };
  report: Report | null;
  reportTab: string;
  chartRange: number;
  busy: { active: boolean; text: string };
  toast: { visible: boolean; text: string; ok: boolean };
}

export const app: AppState = $state({
  snapshot: null,
  info: { name: "Steam Account Switcher", version: "", repository: "" },
  search: "",
  folderId: null,
  openAccountId: null,
  switchingId: "",
  idle: false,
  settingsOpen: false,
  settingsSection: "general",
  updateDialog: false,
  update: null,
  updateCheck: "idle",
  updateStage: "",
  updateVersion: "",
  progress: { downloaded: 0, total: 0 },
  report: null,
  reportTab: "game",
  chartRange: 30,
  busy: { active: false, text: "Working…" },
  toast: { visible: false, text: "", ok: false },
});

let toastTimer = 0;

export function showToast(text: string, ok = false): void {
  app.toast = { visible: true, text, ok };
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => {
    app.toast = { ...app.toast, visible: false };
  }, 4200);
}

export function setBusy(active: boolean, text = "Working…"): void {
  app.busy = { active, text };
}

export function accounts(): Account[] {
  return app.snapshot?.accounts ?? [];
}

export function folders(): Folder[] {
  return app.snapshot?.folders ?? [];
}

export function accountById(steamId: string): Account | null {
  return accounts().find((entry) => entry.steam_id === steamId) ?? null;
}

export function currentFolder(): Folder | null {
  return folders().find((folder) => folder.id === app.folderId) ?? null;
}

/** Accounts in the grid: the open folder, or the search results, newest first. */
export function visibleAccounts(): Account[] {
  const needle = app.search.trim().toLowerCase();
  return accounts()
    .filter((entry) => {
      if (!needle) {
        return app.folderId === null ? !entry.folder_id : entry.folder_id === app.folderId;
      }
      return (
        entry.display_name.toLowerCase().includes(needle) ||
        entry.account_name.toLowerCase().includes(needle) ||
        entry.persona_name.toLowerCase().includes(needle) ||
        entry.steam_id.includes(needle)
      );
    })
    .sort((left, right) => right.last_login_seconds - left.last_login_seconds);
}

/** Folder cards only exist at the root, and never while searching. */
export function visibleFolders(): Folder[] {
  return app.search.trim() || app.folderId !== null ? [] : folders();
}

export function emptyMessage(): string {
  const snapshot = app.snapshot;
  if (!snapshot) {
    return "";
  }
  if (snapshot.installations.length === 0) {
    return "No Steam installation found. Add one from the settings panel.";
  }
  if (snapshot.accounts.length === 0) {
    return "No remembered Steam account here yet. Use + to sign in with one.";
  }
  return "No account matches this search.";
}

export function applySnapshot(snapshot: Snapshot): void {
  const known = new Set(accounts().map((entry) => entry.steam_id));
  app.snapshot = snapshot;

  if (app.folderId && !snapshot.folders.some((folder) => folder.id === app.folderId)) {
    app.folderId = null;
  }
  if (
    app.openAccountId &&
    !snapshot.accounts.some((entry) => entry.steam_id === app.openAccountId)
  ) {
    app.openAccountId = null;
  }
  if (known.size > 0 && snapshot.accounts.some((entry) => !known.has(entry.steam_id))) {
    showToast("Steam account list updated.", true);
  }
}

/** Refreshes the playtime figures without rebuilding the grid. */
export function applyPlaytime(playtime: PlaytimeSnapshot): void {
  if (!playtime || !app.snapshot) {
    return;
  }
  for (const cell of playtime.accounts) {
    const entry = accountById(cell.steam_id);
    if (entry) {
      entry.playtime_seconds = cell.seconds;
      entry.playtime_text = cell.text;
      entry.session_count = cell.session_count;
    }
  }
  app.snapshot.machine_text = playtime.machine_text;
  app.snapshot.tracker_ready = playtime.tracker_ready;
}

export interface RunOptions {
  busy?: string;
  success?: string;
}

/** Runs a command, applies snapshots automatically and reports failures. */
export async function run<K extends keyof Commands>(
  command: K,
  args?: Commands[K]["args"],
  options: RunOptions = {},
): Promise<Commands[K]["result"] | null> {
  if (options.busy) {
    setBusy(true, options.busy);
  }
  try {
    const result = await invoke(command, args);
    if (isSnapshot(result)) {
      applySnapshot(result);
    }
    if (options.success) {
      showToast(options.success, true);
    }
    return result;
  } catch (error) {
    showToast(String(error));
    return null;
  } finally {
    if (options.busy) {
      setBusy(false);
    }
  }
}

/** Only a snapshot carries the installation list and the settings block. */
function isSnapshot(value: unknown): value is Snapshot {
  const candidate = value as Snapshot | null;
  return Boolean(candidate && Array.isArray(candidate.installations) && candidate.settings);
}
