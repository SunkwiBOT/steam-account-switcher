import { errorMessage } from "../errors/errors.svelte";
import { t, rich, locale } from "../i18n/i18n.svelte";
import { ConfirmKind } from "../state/dialogs.constants";
import { UpdateCheck, UpdateInterval, UpdateStage, ReportTab } from "../state/state.constants";
import { updateCheckDue } from "../updates/update-policy";

import { copyValue } from "../state/clipboard.svelte";
import { askConfirm, confirmDialog } from "../state/dialogs.svelte";
import { trackingEnabled, accountById, app, run, showToast } from "../state/state.svelte";
import { invoke, invokePlugin } from "../api/tauri";
import type { ReportType } from "../types";

const FALLBACK_REPOSITORY_URL = "https://github.com/SunkwiBOT/steam-account-switcher";

export const repositoryUrl = (): string => app.info.repository || FALLBACK_REPOSITORY_URL;

export const openRepository = async (): Promise<void> => {
  const url = repositoryUrl();
  try {
    await invokePlugin("plugin:opener|open_url", { url, with: null });
  } catch (error) {
    showToast(errorMessage(error));
  }
};

let switchPending = false;

export const switchAccount = async (steamId: string): Promise<void> => {
  if (app.busy.active || app.switchingId || switchPending || confirmDialog.open) return;
  const entry = accountById(steamId);
  if (!entry) {
    return;
  }
  if (!entry.is_local) {
    showToast(t.account_switch_sign_in_required(), false, entry);
    return;
  }
  app.openAccountId = null;

  if (app.switchingId) {
    showToast(t.account_switch_busy());
    return;
  }

  const steamRunning = app.snapshot?.steam_running === true;
  if (entry.is_current && steamRunning) {
    showToast(rich(t.account_switch_already_active, { account: entry.display_name }), false, entry);
    return;
  }

  const installationRoot = app.snapshot?.installations[app.snapshot.selected]?.root;
  switchPending = true;
  try {
    const answer = await askConfirm({
      title: steamRunning ? t.account_switch_title() : t.account_start_action(),
      steamId,
      kind: steamRunning ? ConfirmKind.SWITCH : ConfirmKind.START,
      text: steamRunning
        ? rich(t.account_switch_confirm, { account: entry.display_name })
        : rich(t.account_start_confirm, { account: entry.display_name }),
      confirmLabel: steamRunning ? t.account_switch_action() : t.account_start_action(),
    });
    if (!answer) {
      return;
    }

    if (
      app.busy.active ||
      app.switchingId ||
      app.snapshot?.installations[app.snapshot.selected]?.root !== installationRoot
    )
      return;

    const games = await run("running_steam_games", undefined, {
      busy: t.account_switch_games_pending(),
      account: entry,
    });
    if (!games) return;
    if (
      app.busy.active ||
      app.switchingId ||
      confirmDialog.open ||
      app.snapshot?.installations[app.snapshot.selected]?.root !== installationRoot
    )
      return;
    if (games.length) {
      const proceed = await askConfirm({
        title: t.account_switch_games_title({ count: games.length }),
        steamId,
        kind: ConfirmKind.RUNNING_GAME,
        text: rich(t.account_switch_games_warning, {
          games: games.join(", "),
          account: entry.display_name,
        }),
        confirmLabel: t.account_switch_games_action(),
        danger: true,
      });
      if (!proceed) return;
    }
    if (
      app.busy.active ||
      app.switchingId ||
      app.snapshot?.installations[app.snapshot.selected]?.root !== installationRoot
    )
      return;

    app.switchingId = steamId;
    try {
      const result = await run("switch_account", { steamId });
      if (result) {
        showToast(
          steamRunning
            ? rich(t.account_switch_success, { account: entry.display_name })
            : rich(t.account_start_success, { account: entry.display_name }),
          true,
          entry,
        );
      }
    } finally {
      app.switchingId = "";
    }
  } catch (error) {
    showToast(errorMessage(error), false, entry);
  } finally {
    switchPending = false;
  }
};

export const removeAccount = async (steamId: string): Promise<void> => {
  if (app.busy.active || app.switchingId || switchPending || confirmDialog.open) return;
  const entry = accountById(steamId);
  if (!entry) {
    return;
  }
  app.openAccountId = null;

  if (!entry.is_local) {
    if (app.snapshot?.vault.locked) {
      app.vaultCopySteamId = null;
      app.vaultRemoveSteamId = steamId;
      app.vaultOpen = true;
      return;
    }
    const answer = await askConfirm({
      title: t.guard_saved_delete(),
      kind: ConfirmKind.REMOVE,
      steamId,
      text: rich(t.guard_saved_delete_warning, {
        account: entry.display_name,
      }),
      checkboxLabel: t.guard_saved_delete_acknowledgement(),
      checkboxRequired: true,
      confirmLabel: t.guard_saved_delete(),
      danger: true,
    });
    if (!answer?.cleanup) return;
    await run(
      "forget_saved_account",
      { steamId, confirmed: true },
      {
        busy: t.guard_saved_delete_pending(),
        success: rich(t.guard_saved_delete_success, {
          account: entry.display_name,
        }),
        account: entry,
      },
    );
    return;
  }

  const answer = await askConfirm({
    title: t.account_remove_title(),
    kind: ConfirmKind.REMOVE,
    steamId,
    text: entry.has_authenticator
      ? rich(t.account_remove_guard_confirm, {
          account: entry.display_name,
        })
      : rich(t.account_remove_confirm, {
          account: entry.display_name,
        }),
    checkboxLabel: t.account_remove_data_checkbox(),
    confirmLabel: t.common_remove(),
    danger: true,
  });
  if (!answer) {
    return;
  }
  await run(
    "remove_account",
    { steamId, cleanupUserdata: Boolean(answer.cleanup), confirmed: true },
    {
      busy: t.account_remove_pending(),
      success: entry.has_authenticator
        ? rich(t.account_remove_guard_success, {
            account: entry.display_name,
          })
        : rich(t.account_remove_success, { account: entry.display_name }),
      account: entry,
    },
  );
};

export const addAccount = async (): Promise<void> => {
  if (app.busy.active || app.switchingId || switchPending || confirmDialog.open) return;
  const answer = await askConfirm({
    title: t.account_add_title(),
    kind: ConfirmKind.ADD,
    text: t.account_add_description(),
    confirmLabel: t.account_add_action(),
  });
  if (answer) {
    await run("add_steam_account", undefined, {
      busy: t.account_add_pending(),
      success: t.account_add_success(),
    });
  }
};

export const copySteamId = async (steamId: string): Promise<void> => {
  try {
    await copyValue(steamId, `id:${steamId}:Steam ID 64`, t.steam_ids_64(), accountById(steamId));
  } catch {
    showToast(t.steam_ids_copy_error(), false, accountById(steamId));
  }
};

/** Fetch a fresh local code at click time, rather than copying a stale display. */
export const copySteamGuardCode = async (steamId: string): Promise<boolean> => {
  if (app.busy.active || app.switchingId) return false;
  const entry = accountById(steamId);
  if (app.snapshot?.vault.locked) {
    app.vaultRemoveSteamId = null;
    app.vaultCopySteamId = steamId;
    app.vaultOpen = true;
    return false;
  }
  try {
    const { code } = await invoke("steam_guard_code", { steamId });
    return await copyValue(code, `guard:${steamId}`, t.guard_code_label(), entry);
  } catch (cause) {
    showToast(errorMessage(cause), false, entry);
    return false;
  }
};

const showReport = (report: ReportType): void => {
  reportRetryAt = 0;
  app.report = report;
  app.reportTab = ReportTab.GAME;
  app.chartRange = 30;
};

let reportRefreshing = false;
let reportRetryAt = 0;

export const openAccountReport = async (steamId: string): Promise<void> => {
  if (!trackingEnabled() || app.reportLoadingId) return;
  app.reportLoadingId = steamId;
  try {
    const report = await run("account_report", { steamId });
    if (report && trackingEnabled()) showReport(report);
  } finally {
    app.reportLoadingId = "";
  }
};

export const openMachineReport = async (): Promise<void> => {
  if (!trackingEnabled() || app.reportLoadingId) return;
  app.reportLoadingId = "machine";
  try {
    const report = await run("machine_report");
    if (report && trackingEnabled()) showReport(report);
  } finally {
    app.reportLoadingId = "";
  }
};

export const refreshOpenReport = async (): Promise<void> => {
  const report = app.report;
  if (!report || !trackingEnabled()) {
    return;
  }
  if (reportRefreshing || Date.now() < reportRetryAt) return;
  reportRefreshing = true;
  try {
    const next = report.machine
      ? await invoke("machine_report")
      : await invoke("account_report", { steamId: report.account_id ?? "" });
    if (app.report === report && trackingEnabled()) {
      app.report = next;
      reportRetryAt = 0;
    }
  } catch {
    if (app.report === report) reportRetryAt = Date.now() + 30_000;
  } finally {
    reportRefreshing = false;
  }
};

export const checkForUpdate = async (): Promise<void> => {
  if (app.updateStage || app.updateCheck === UpdateCheck.CHECKING) {
    return;
  }
  app.updateCheck = UpdateCheck.CHECKING;
  try {
    app.update = await invoke("check_for_update");
    app.updateCheck = UpdateCheck.SUCCESS;
  } catch {
    app.update = null;
    app.updateCheck = UpdateCheck.ERROR;
  } finally {
    app.updateCheckedAt = Math.floor(Date.now() / 1000);
  }
};

export const checkForUpdateIfDue = async (startup = false): Promise<void> => {
  const interval = app.snapshot?.settings.update_interval ?? UpdateInterval.SIX_HOURS;
  if (updateCheckDue(interval, app.updateCheckedAt, Math.floor(Date.now() / 1000), startup)) {
    await checkForUpdate();
  }
};

export const updateButtonLabel = (): string => {
  if (app.updateStage === UpdateStage.INSTALLING) return t.common_restarting();
  if (app.updateStage === UpdateStage.DOWNLOADED) {
    return t.common_restart();
  }
  if (app.updateStage === UpdateStage.DOWNLOADING) {
    return t.common_downloading();
  }
  return app.update ? t.updates_action({ version: app.update.version }) : "";
};

export const downloadUpdate = async (): Promise<void> => {
  if (app.updateStage) return;
  app.updateDialog = false;
  app.updateStage = UpdateStage.DOWNLOADING;
  app.progress = { downloaded: 0, total: 0 };

  const version = await run("download_update");
  if (typeof version !== "string") {
    app.updateStage = UpdateStage.IDLE;
    return;
  }
  // The download always takes the newest release, which can differ from the one checked earlier.
  if (app.update && app.update.version !== version) {
    app.update = { ...app.update, version };
  }
  markDownloaded(version);
};

export const markDownloaded = (version: string): void => {
  if (app.updateStage === UpdateStage.INSTALLING) return;
  app.updateStage = UpdateStage.DOWNLOADED;
  app.updateVersion = version;
  app.updateDialog = true;
};

export const recordUpdateProgress = (payload: {
  downloaded?: number;
  total?: number | null;
}): void => {
  if (app.updateStage === UpdateStage.DOWNLOADED || app.updateStage === UpdateStage.INSTALLING) {
    return;
  }
  app.updateStage = UpdateStage.DOWNLOADING;
  app.progress = { downloaded: payload.downloaded ?? 0, total: payload.total ?? 0 };
};

export const applyUpdate = async (): Promise<void> => {
  if (app.updateStage !== UpdateStage.DOWNLOADED) return;
  app.updateStage = UpdateStage.INSTALLING;
  try {
    await invoke("apply_update", { version: app.updateVersion });
  } catch (cause) {
    app.updateStage = UpdateStage.DOWNLOADED;
    showToast(errorMessage(cause));
  }
};

const megabytes = (bytes: number): string => {
  return t.updates_size_megabytes({
    size: new Intl.NumberFormat(locale.current, {
      minimumFractionDigits: 1,
      maximumFractionDigits: 1,
    }).format(bytes / 1024 / 1024),
  });
};

export const updateProgressText = (): string => {
  const { downloaded, total } = app.progress;
  if (!total) {
    return t.updates_progress_downloaded({ downloaded: megabytes(downloaded) });
  }
  const percent = Math.min(100, Math.round((downloaded / total) * 100));
  return t.updates_progress_total({
    percent,
    downloaded: megabytes(downloaded),
    total: megabytes(total),
  });
};

export const updateProgressPercent = (): number => {
  const { downloaded, total } = app.progress;
  return total ? Math.min(100, Math.round((downloaded / total) * 100)) : 0;
};
