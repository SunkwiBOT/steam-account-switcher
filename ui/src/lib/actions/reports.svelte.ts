import { ReportTab } from "../state/state.constants";
import { app, run, trackingEnabled } from "../state/state.svelte";
import { invoke } from "../api/tauri";
import type { ReportDto } from "../bindings/ReportDto";

const showReport = (report: ReportDto): void => {
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
  } catch (e: unknown) {
    if (app.report === report) reportRetryAt = Date.now() + 30_000;
  } finally {
    reportRefreshing = false;
  }
};
