/* Everything that acts on Steam: switching, removing, adding, updating. */

import { strong } from "./api";
import { askConfirm } from "./dialogs.svelte";
import { accountById, app, run, showToast } from "./state.svelte";
import { invoke, invokePlugin } from "./tauri";
import type { Report } from "./types";

/* ------------------------------------------------------------------ switch */

/** Project page, used by the header logo, the footer and the settings panel.
 *  It is opened through the opener plugin command, which the plugin registers
 *  itself, rather than an application command. `app.info.repository` comes
 *  from `app_info`, the literal is only the fallback for the first moments of
 *  a session. */
const FALLBACK_REPOSITORY_URL = "https://github.com/SunkwiBOT/steam-account-switcher";

export async function openRepository(): Promise<void> {
  const url = app.info.repository || FALLBACK_REPOSITORY_URL;
  try {
    await invokePlugin("plugin:opener|open_url", { url, with: null });
  } catch (error) {
    showToast(String(error));
  }
}

export async function switchAccount(steamId: string): Promise<void> {
  const entry = accountById(steamId);
  if (!entry) {
    return;
  }
  app.openAccountId = null;

  if (app.switchingId) {
    showToast("A switch is already running. Wait for it to finish.");
    return;
  }

  const steamRunning = app.snapshot?.steam_running === true;
  if (entry.is_current && steamRunning) {
    showToast(`${entry.display_name} is already the active Steam account.`);
    return;
  }

  const answer = await askConfirm({
    title: steamRunning ? "Switch Steam account" : "Start Steam",
    text: steamRunning
      ? `Switch to ${strong(entry.display_name)} and restart Steam?`
      : `Steam is not running. Start it on ${strong(entry.display_name)}?`,
    confirmLabel: steamRunning ? "Switch" : "Start Steam",
  });
  if (!answer) {
    return;
  }

  if (app.switchingId) {
    showToast("A switch is already running. Wait for it to finish.");
    return;
  }

  app.switchingId = steamId;
  try {
    await run("switch_account", { steamId });
  } finally {
    app.switchingId = "";
  }
}

export async function removeAccount(steamId: string): Promise<void> {
  const entry = accountById(steamId);
  if (!entry) {
    return;
  }
  app.openAccountId = null;

  const answer = await askConfirm({
    title: "Remove Steam account",
    text: `Remove ${strong(entry.display_name)}? Steam is closed first, because it rewrites this file when it exits.`,
    checkboxLabel: "Delete this account's local Steam data, including saves and settings",
    confirmLabel: "Remove",
    danger: true,
  });
  if (!answer) {
    return;
  }
  await run(
    "remove_account",
    { steamId, cleanupUserdata: Boolean(answer.cleanup) },
    { busy: "Removing account…" },
  );
}

export async function addAccount(): Promise<void> {
  const answer = await askConfirm({
    title: "Add a Steam account",
    text: "Steam will restart at its login window so you can sign in with the account to add. Steam asks for the password and Steam Guard itself; this application never sees them.",
    confirmLabel: "Continue",
  });
  if (answer) {
    await run("add_steam_account", undefined, { busy: "Restarting Steam at its login window…" });
  }
}

export async function copySteamId(steamId: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(steamId);
    showToast("Steam ID 64 copied.", true);
  } catch {
    showToast("Unable to copy the Steam ID.");
  }
}

/* ------------------------------------------------------------- play reports */

function showReport(report: Report): void {
  app.report = report;
  app.reportTab = "game";
  app.chartRange = 30;
}

export async function openAccountReport(steamId: string): Promise<void> {
  const report = await run("account_report", { steamId });
  if (report) {
    showReport(report);
  }
}

export async function openMachineReport(): Promise<void> {
  const report = await run("machine_report");
  if (report) {
    showReport(report);
  }
}

/** Re-fetches the open report, used by the once-per-second playtime refresh. */
export async function refreshOpenReport(): Promise<void> {
  const report = app.report;
  if (!report) {
    return;
  }
  const next = report.machine
    ? await run("machine_report")
    : await run("account_report", { steamId: report.account_id ?? "" });
  if (next) {
    app.report = next;
  }
}

/* ------------------------------------------------------------------ update */

export async function checkForUpdate(): Promise<void> {
  if (app.updateStage || app.updateCheck === "checking") {
    return;
  }
  app.updateCheck = "checking";
  try {
    app.update = await invoke("check_for_update");
    app.updateCheck = "success";
  } catch {
    app.update = null;
    app.updateCheck = "error";
  }
}

export function updateButtonLabel(): string {
  if (app.updateStage === "downloaded") {
    return "Restart";
  }
  if (app.updateStage === "downloading") {
    return "Downloading…";
  }
  return app.update ? `Update ${app.update.version}` : "";
}

export async function downloadUpdate(): Promise<void> {
  app.updateDialog = false;
  app.updateStage = "downloading";
  app.progress = { downloaded: 0, total: 0 };

  const version = await run("download_update");
  if (typeof version !== "string") {
    app.updateStage = "";
    return;
  }
  markDownloaded(version);
}

export function markDownloaded(version: string): void {
  app.updateStage = "downloaded";
  app.updateVersion = version;
  app.updateDialog = true;
}

export function recordUpdateProgress(payload: {
  downloaded?: number;
  total?: number | null;
}): void {
  if (app.updateStage === "downloaded") {
    return;
  }
  app.updateStage = "downloading";
  app.progress = { downloaded: payload.downloaded ?? 0, total: payload.total ?? 0 };
}

export async function applyUpdate(): Promise<void> {
  await run("apply_update");
}

function megabytes(bytes: number): string {
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

export function updateProgressText(): string {
  const { downloaded, total } = app.progress;
  if (!total) {
    return `${megabytes(downloaded)} downloaded`;
  }
  const percent = Math.min(100, Math.round((downloaded / total) * 100));
  return `${percent}% · ${megabytes(downloaded)} of ${megabytes(total)}`;
}

export function updateProgressPercent(): number {
  const { downloaded, total } = app.progress;
  return total ? Math.min(100, Math.round((downloaded / total) * 100)) : 0;
}
