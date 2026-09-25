/* Tauri events, window focus and the first snapshot. */

import {
  checkForUpdate,
  markDownloaded,
  recordUpdateProgress,
  refreshOpenReport,
  switchAccount,
} from "./actions.svelte";
import { app, applyPlaytime, applySnapshot, run } from "./state.svelte";
import { invoke, listen } from "./tauri";

const CHECK_INTERVAL_MS = 6 * 60 * 60 * 1000;

/** Suspends interval work and animations while the window is in the
 *  background: the application should cost nothing when it is not used. */
async function setActive(active: boolean): Promise<void> {
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
    applyPlaytime(await invoke("playtime_snapshot"));
    await checkForUpdate();
  }
}

export async function startBridge(): Promise<void> {
  try {
    app.info = await invoke("app_info");
  } catch {
    // The header and the footer simply keep their defaults.
  }

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

  await listen("accounts-changed", async () => {
    const next = await invoke("snapshot");
    if (next?.installations) {
      applySnapshot(next);
    }
  });

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

  // Register local events before contacting the release server.
  void checkForUpdate();
  setInterval(checkForUpdate, CHECK_INTERVAL_MS);
}
