import { errorMessage } from "../errors/errors.svelte";
import type { BackupMode } from "../vault/backup.constants";
import { applyLanguage } from "../i18n/i18n.svelte";
import type { ResolvedLanguageType } from "../i18n/i18n.constants";
import { t } from "../i18n/i18n.svelte";
import { SettingsSection, ToastKind, UpdateCheck, UpdateStage, ReportTab } from "./state.constants";

import { invoke, type CommandsType } from "../api/tauri";
import type { MessageType } from "../i18n/messages";
import type { AccountDto } from "../bindings/AccountDto";
import type { AppInfoDto } from "../bindings/AppInfoDto";
import type { PlaytimeSnapshotDto } from "../bindings/PlaytimeSnapshotDto";
import type { ReportDto } from "../bindings/ReportDto";
import type { SnapshotDto } from "../bindings/SnapshotDto";
import type { UpdateInfoDto } from "../bindings/UpdateInfoDto";
import type { ToastAccountType } from "../types";
import { showToast } from "./toasts.svelte";

export { showToast };

export type AppStateType = {
  snapshot: SnapshotDto | null;
  info: AppInfoDto;
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
  vaultOfferOpen: boolean;
  vaultCopySteamId: string | null;
  vaultRemoveSteamId: string | null;
  backup: { mode: BackupMode; steamId: string | null } | null;
  updateDialog: boolean;
  update: UpdateInfoDto | null;
  updateCheck: UpdateCheck;
  updateCheckedAt: number;
  updateStage: UpdateStage;
  updateVersion: string;
  progress: {
    downloaded: number;
    total: number;
  };
  report: ReportDto | null;
  reportLoadingId: string;
  reportTab: ReportTab;
  chartRange: number;
  busy: {
    active: boolean;
    text: string;
  };
};

export const app: AppStateType = $state({
  snapshot: null,
  info: {
    name: "Steam Account Switcher",
    version: "",
    repository: "",
    backup_extension: "",
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
  vaultOfferOpen: false,
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
});

export const setBusy = (active: boolean, text: string = t.common_working()): void => {
  app.busy = { active, text };
};

export const accounts = (): AccountDto[] => {
  return app.snapshot?.accounts ?? [];
};

export const trackingEnabled = (): boolean => app.snapshot?.settings.tracking_enabled !== false;

export const accountById = (steamId: string): AccountDto | null => {
  return accounts().find((entry) => entry.steam_id === steamId) ?? null;
};

export const applySnapshot = (snapshot: SnapshotDto): void => {
  if (snapshot.revision < (app.snapshot?.revision ?? 0)) return;
  const known = new Set(accounts().map((entry) => entry.steam_id));
  app.snapshot = snapshot;
  if (!trackingEnabled()) app.report = null;
  app.updateCheckedAt = Math.max(app.updateCheckedAt, snapshot.settings.last_update_check_at ?? 0);
  applyLanguage(
    snapshot.settings.language,
    snapshot.settings.resolved_language as ResolvedLanguageType,
  );

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
    showToast(ToastKind.SUCCESS, t.accounts_refresh_success());
  }
};

export const applyPlaytime = (playtime: PlaytimeSnapshotDto): void => {
  if (!app.snapshot) {
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
  successTitle?: string;
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
    if (options.successTitle) {
      showToast(ToastKind.SUCCESS, options.successTitle, { description: options.success, account });
    }
    return result;
  } catch (error: unknown) {
    showToast(ToastKind.ERROR, t.toast_error_title(), {
      description: errorMessage(error),
      account,
    });
    return null;
  } finally {
    if (options.busy) {
      setBusy(false);
    }
  }
};

export const isSnapshot = (value: unknown): value is SnapshotDto => {
  return (
    typeof value === "object" &&
    value !== null &&
    "installations" in value &&
    Array.isArray(value.installations) &&
    "settings" in value &&
    Boolean(value.settings)
  );
};
