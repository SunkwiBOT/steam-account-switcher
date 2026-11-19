import { errorMessage } from "../errors/errors.svelte";
import { t, locale } from "../i18n/i18n.svelte";
import { ToastKind, UpdateCheck, UpdateStage } from "../state/state.constants";
import { updateCheckDue } from "../updates/update-policy";

import { app, run, showToast } from "../state/state.svelte";
import { invoke } from "../api/tauri";

export const checkForUpdate = async (): Promise<void> => {
  if (app.updateStage || app.updateCheck === UpdateCheck.CHECKING) {
    return;
  }
  app.updateCheck = UpdateCheck.CHECKING;
  try {
    app.update = await invoke("check_for_update");
    app.updateCheck = UpdateCheck.SUCCESS;
  } catch (e: unknown) {
    app.update = null;
    app.updateCheck = UpdateCheck.ERROR;
  } finally {
    app.updateCheckedAt = Math.floor(Date.now() / 1000);
  }
};

export const checkForUpdateIfDue = async (startup = false): Promise<void> => {
  const interval = app.snapshot?.settings.update_interval ?? "SIX_HOURS";
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
  app.updateDialog = true;
  app.updateStage = UpdateStage.DOWNLOADING;
  app.progress = { downloaded: 0, total: 0 };

  const version = await run("download_update");
  if (typeof version !== "string") {
    app.updateStage = UpdateStage.IDLE;
    return;
  }
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
  } catch (cause: unknown) {
    app.updateStage = UpdateStage.DOWNLOADED;
    showToast(ToastKind.ERROR, t.toast_error_title(), { description: errorMessage(cause) });
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
