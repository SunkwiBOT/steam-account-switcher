import {
  checkForUpdateIfDue,
  markDownloaded,
  recordUpdateProgress,
} from "../actions/updates.svelte";
import { refreshOpenReport } from "../actions/reports.svelte";
import { switchAccount } from "../actions/account-lifecycle.svelte";
import { t } from "../i18n/i18n.svelte";
import { app, applyPlaytime, applySnapshot, run, showToast } from "../state/state.svelte";
import { ToastKind } from "../state/state.constants";
import { createVaultOfferGate } from "../vault/vault-offer";
import { invoke, listen } from "./tauri";

const CHECK_INTERVAL_MS = 60 * 1000;

const vaultOfferGate = createVaultOfferGate();

const formatOffset = (seconds: number): string => {
  const sign = seconds < 0 ? "-" : "+";
  const total = Math.abs(seconds);
  const minutes = Math.floor(total / 60);
  const remainder = total % 60;
  return minutes > 0 ? `${sign}${minutes}m ${remainder}s` : `${sign}${remainder}s`;
};

const syncSnapshot = async (): Promise<void> => {
  applySnapshot(await invoke("snapshot"));
};

const setActive = async (active: boolean): Promise<void> => {
  if (app.idle === !active) {
    return;
  }
  app.idle = !active;
  try {
    await invoke("set_window_active", { active });
  } catch (e: unknown) {}
  if (active) {
    try {
      await syncSnapshot();
      applyPlaytime(await invoke("playtime_snapshot"));
    } catch (e: unknown) {}
    await checkForUpdateIfDue();
  }
};

export const startBridge = async (): Promise<void> => {
  try {
    app.info = await invoke("app_info");
  } catch (e: unknown) {}

  await run("snapshot");

  if (
    app.snapshot &&
    !app.snapshot.settings.vault_offer_dismissed &&
    vaultOfferGate.check(app.snapshot.vault)
  ) {
    app.vaultOfferOpen = true;
  }

  await listen("playtime-tick", async (event) => {
    if (app.idle) {
      return;
    }
    try {
      applyPlaytime(await invoke("playtime_snapshot"));
    } catch {
      return;
    }
    if (event.payload) {
      await refreshOpenReport();
    }
  });

  await listen("accounts-changed", syncSnapshot);
  await listen("vault-locked", syncSnapshot);
  await listen("clock-drift-warning", (event) => {
    showToast(ToastKind.ERROR, t.toast_clock_drift_title(), {
      description: t.clock_drift_warning({ offset: formatOffset(event.payload) }),
    });
  });

  await listen("switch-requested", (event) => {
    if (event.payload) {
      switchAccount(event.payload);
    }
  });

  await listen("update-progress", (event) => recordUpdateProgress(event.payload));
  await listen("update-downloaded", (event) => markDownloaded(event.payload));

  window.addEventListener("focus", () => setActive(true));
  window.addEventListener("blur", () => setActive(false));
  document.addEventListener("visibilitychange", () =>
    setActive(document.visibilityState === "visible"),
  );

  void checkForUpdateIfDue(true);
  setInterval(checkForUpdateIfDue, CHECK_INTERVAL_MS);
};
