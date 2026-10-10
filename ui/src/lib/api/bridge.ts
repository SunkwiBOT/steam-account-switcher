import {
  checkForUpdateIfDue,
  markDownloaded,
  recordUpdateProgress,
  refreshOpenReport,
  switchAccount,
} from "../actions/actions.svelte";
import { app, applyPlaytime, applySnapshot, run } from "../state/state.svelte";
import { invoke, listen } from "./tauri";

const CHECK_INTERVAL_MS = 60 * 1000;

const syncSnapshot = async (): Promise<void> => {
  const next = await invoke("snapshot");
  if (next?.installations) {
    applySnapshot(next);
  }
};

const setActive = async (active: boolean): Promise<void> => {
  if (app.idle === !active) {
    return;
  }
  app.idle = !active;
  try {
    await invoke("set_window_active", { active });
  } catch {
    // Best effort: the Rust side also follows the window focus events.
  }
  if (active) {
    // The vault may have locked itself while the window was in the background.
    await syncSnapshot();
    applyPlaytime(await invoke("playtime_snapshot"));
    await checkForUpdateIfDue();
  }
};

export const startBridge = async (): Promise<void> => {
  try {
    app.info = await invoke("app_info");
  } catch {}

  await run("snapshot");

  await listen("playtime-tick", async (event) => {
    if (app.idle) {
      return;
    }
    applyPlaytime(await invoke("playtime_snapshot"));
    if (event.payload) {
      await refreshOpenReport();
    }
  });

  await listen("accounts-changed", syncSnapshot);
  await listen("vault-locked", syncSnapshot);

  // The tray asks the interface to switch, so both paths behave the same.
  await listen("switch-requested", async (event) => {
    const steamId = String(event.payload ?? "");
    if (steamId) {
      await switchAccount(steamId);
    }
  });

  await listen("update-progress", (event) => recordUpdateProgress(event.payload ?? {}));
  await listen("update-downloaded", (event) => markDownloaded(String(event.payload ?? "")));

  window.addEventListener("focus", () => setActive(true));
  window.addEventListener("blur", () => setActive(false));
  document.addEventListener("visibilitychange", () =>
    setActive(document.visibilityState === "visible"),
  );

  void checkForUpdateIfDue(true);
  setInterval(checkForUpdateIfDue, CHECK_INTERVAL_MS);
};
