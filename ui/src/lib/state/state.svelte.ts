import { errorMessage } from "../errors/errors.svelte";
import { Language } from "../i18n/i18n.constants";
import { VaultLockInterval } from "./state.constants";
import type { BackupMode } from "../vault/backup.constants";
import { applyLanguage } from "../i18n/i18n.svelte";
import { t } from "../i18n/i18n.svelte";
import {
  SearchScope,
  SettingsSection,
  UpdateCheck,
  UpdateInterval,
  UpdateStage,
  ReportTab,
} from "./state.constants";
import { matchesSearch } from "../accounts/search";

import { invoke, type CommandsType } from "../api/tauri";
import type { MessageType, MessagePartType } from "../i18n/messages";
import type {
  AccountType,
  AppInfoType,
  FolderType,
  PlaytimeSnapshotType,
  ReportType,
  SettingsType,
  SnapshotType,
  ToastAccountType,
  ToastFolderType,
  UpdateInfoType,
} from "../types";

export const EMPTY_SETTINGS: SettingsType = {
  language: Language.AUTO,
  resolved_language: Language.EN,
  show_installation_selector: true,
  tracking_enabled: true,
  tracking_history: [],
  close_to_tray: true,
  update_interval: UpdateInterval.SIX_HOURS,
  search_scope: SearchScope.BOTH,
  vault_lock_interval: VaultLockInterval.STARTUP,
  last_update_check_at: null,
  entries: [],
  database_path: "",
};

export type AppStateType = {
  snapshot: SnapshotType | null;
  info: AppInfoType;
  search: string;
  folderId: string | null;
  openAccountId: string | null;
  switchingId: string;
  idle: boolean;
  settingsOpen: boolean;
  settingsSection: SettingsSection;
  steamGuardOpen: boolean;
  steamGuardAccountId: string | null;
  vaultOpen: boolean;
  vaultCopySteamId: string | null;
  vaultRemoveSteamId: string | null;
  backup: { mode: BackupMode; steamId: string | null } | null;
  updateDialog: boolean;
  update: UpdateInfoType | null;
  updateCheck: UpdateCheck;
  updateCheckedAt: number;
  updateStage: UpdateStage;
  updateVersion: string;
  progress: {
    downloaded: number;
    total: number;
  };
  report: ReportType | null;
  reportLoadingId: string;
  reportTab: ReportTab;
  chartRange: number;
  busy: {
    active: boolean;
    text: string;
  };
  toast: {
    visible: boolean;
    text: string;
    parts: MessagePartType[];
    ok: boolean;
    account: ToastAccountType | null;
    folder: ToastFolderType | null;
  };
};

export const app: AppStateType = $state({
  snapshot: null,
  info: {
    name: "Steam Account Switcher",
    version: "",
    repository: "",
  },
  search: "",
  folderId: null,
  openAccountId: null,
  switchingId: "",
  idle: false,
  settingsOpen: false,
  settingsSection: SettingsSection.PREFERENCES,
  steamGuardOpen: false,
  steamGuardAccountId: null,
  vaultOpen: false,
  vaultCopySteamId: null,
  vaultRemoveSteamId: null,
  backup: null,
  updateDialog: false,
  update: null,
  updateCheck: UpdateCheck.IDLE,
  updateCheckedAt: 0,
  updateStage: UpdateStage.IDLE,
  updateVersion: "",
  progress: { downloaded: 0, total: 0 },
  report: null,
  reportLoadingId: "",
  reportTab: ReportTab.GAME,
  chartRange: 30,
  busy: { active: false, text: t.common_working() },
  toast: { visible: false, text: "", parts: [], ok: false, account: null, folder: null },
});

let toastTimer = 0;

export const showToast = (
  content: MessageType,
  ok = false,
  account: ToastAccountType | null = null,
  folder: ToastFolderType | null = null,
): void => {
  const parts = typeof content === "string" ? [{ text: content }] : content;
  app.toast = {
    visible: true,
    text: parts.map((part) => part.text).join(""),
    parts,
    ok,
    account: account
      ? {
          avatar: account.avatar,
          initial: account.initial,
          display_name: account.display_name,
          persona_name: account.persona_name,
        }
      : null,
    folder: folder
      ? {
          name: folder.name,
          color: folder.color,
          icon: folder.icon,
          system: folder.system,
        }
      : null,
  };
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => {
    app.toast = { ...app.toast, visible: false };
  }, 4200);
};

export const setBusy = (active: boolean, text: string = t.common_working()): void => {
  app.busy = { active, text };
};

export const accounts = (): AccountType[] => {
  return app.snapshot?.accounts ?? [];
};

export const trackingEnabled = (): boolean => app.snapshot?.settings.tracking_enabled !== false;

export const folders = (): FolderType[] => {
  const list = app.snapshot?.folders ?? [];
  return [...list.filter((folder) => folder.system), ...list.filter((folder) => !folder.system)];
};

export const accountById = (steamId: string): AccountType | null => {
  return accounts().find((entry) => entry.steam_id === steamId) ?? null;
};

export const currentFolder = (): FolderType | null => {
  return folders().find((folder) => folder.id === app.folderId) ?? null;
};

export const folderById = (id: string): FolderType | null => {
  return folders().find((folder) => folder.id === id) ?? null;
};

export const visibleAccounts = (): AccountType[] => {
  const needle = app.search.trim().toLowerCase();
  const scope = app.snapshot?.settings.search_scope ?? SearchScope.BOTH;
  return accounts()
    .filter((entry) => {
      // A folder always scopes the list, with or without a search term.
      if (app.folderId !== null && entry.folder_id !== app.folderId) return false;
      if (!needle) {
        return app.folderId === null ? !entry.folder_id : true;
      }
      return matchesSearch(entry, needle, scope);
    })
    .sort((left, right) => right.last_login_seconds - left.last_login_seconds);
};

export const visibleFolders = (): FolderType[] => {
  return app.search.trim() || app.folderId !== null ? [] : folders();
};

export const emptyMessage = (): string => {
  const snapshot = app.snapshot;
  if (!snapshot) {
    return "";
  }
  if (snapshot.installations.length === 0) {
    return t.accounts_no_installation();
  }
  if (snapshot.accounts.length === 0) {
    return t.accounts_empty();
  }
  return t.accounts_search_empty();
};

export const applySnapshot = (snapshot: SnapshotType): void => {
  if ((snapshot.revision ?? 0) < (app.snapshot?.revision ?? 0)) return;
  const known = new Set(accounts().map((entry) => entry.steam_id));
  app.snapshot = snapshot;
  if (!trackingEnabled()) app.report = null;
  app.updateCheckedAt = Math.max(app.updateCheckedAt, snapshot.settings.last_update_check_at ?? 0);
  applyLanguage(snapshot.settings.language ?? Language.AUTO, snapshot.settings.resolved_language);

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
    showToast(t.accounts_refresh_success(), true);
  }
};

export const applyPlaytime = (playtime: PlaytimeSnapshotType): void => {
  if (!playtime || !app.snapshot) {
    return;
  }

  for (const cell of playtime.accounts) {
    const entry = accountById(cell.steam_id);

    if (entry) {
      entry.playtime_seconds = cell.seconds;
      entry.session_count = cell.session_count;
    }
  }

  app.snapshot.active_games = playtime.active_games;
  app.snapshot.machine_seconds = playtime.machine_seconds;
  app.snapshot.tracker_ready = playtime.tracker_ready;
};

export type RunOptionsType = {
  busy?: string;
  success?: MessageType;
  account?: ToastAccountType | null;
};

export const run = async <K extends keyof CommandsType>(
  command: K,
  args?: CommandsType[K]["args"],
  options: RunOptionsType = {},
): Promise<CommandsType[K]["result"] | null> => {
  const account =
    options.account ??
    (args && typeof args === "object" && "steamId" in args && typeof args.steamId === "string"
      ? accountById(args.steamId)
      : null);
  if (options.busy) {
    setBusy(true, options.busy);
  }
  try {
    const result = await invoke(command, args);
    if (isSnapshot(result)) {
      applySnapshot(result);
    }
    if (options.success) {
      showToast(options.success, true, account);
    }
    return result;
  } catch (error) {
    showToast(errorMessage(error), false, account);
    return null;
  } finally {
    if (options.busy) {
      setBusy(false);
    }
  }
};

export const isSnapshot = (value: unknown): value is SnapshotType => {
  return (
    typeof value === "object" &&
    value !== null &&
    "installations" in value &&
    Array.isArray(value.installations) &&
    "settings" in value &&
    Boolean(value.settings)
  );
};
