<script lang="ts">
  import DialogHeader from "../dialogs/DialogHeader.svelte";
  import AppDialog from "../dialogs/AppDialog.svelte";
  import { DialogContent } from "$lib/components/ui/dialog/index.js";
  import { t, rich, formatDate } from "../../lib/i18n/i18n.svelte";

  import {
    reportSummary,
    reportStatistics,
    reportNotice,
    statusLabel,
    coverageLabel,
    sessionEnd,
    sessionTooltip,
  } from "../../lib/playtime/reports";
  import { ReportTab } from "../../lib/state/state.constants";
  import { accountById, app, trackingEnabled } from "../../lib/state/state.svelte";
  import MessageText from "../ui/MessageText.svelte";
  import { formatSeconds } from "../../lib/api/api";
  import DailyChart from "./DailyChart.svelte";
  import DataTable from "../ui/DataTable.svelte";
  import { Timer, X } from "@lucide/svelte";

  const report = $derived(app.report);
  const entry = $derived(accountById(report?.account_id ?? ""));
  const tabs = $derived<[ReportTab, string][]>(
    report?.machine
      ? [
          [ReportTab.GAME, t.report_tab_games()],
          [ReportTab.ACCOUNT, t.report_tab_accounts()],
          [ReportTab.DAILY, t.report_tab_daily()],
          [ReportTab.HISTORY, t.report_tab_history()],
        ]
      : [
          [ReportTab.GAME, t.report_tab_games()],
          [ReportTab.DAILY, t.report_tab_daily()],
          [ReportTab.HISTORY, t.report_tab_history()],
        ],
  );

  const dailyPoints = $derived(
    app.chartRange === 7
      ? (report?.daily7 ?? [])
      : app.chartRange === 30
        ? (report?.daily30 ?? [])
        : (report?.daily90 ?? []),
  );

  const close = () => {
    app.report = null;
  };
  const uid = $props.id();

  const tabButtonClasses = (active: boolean): string =>
    active
      ? "min-w-[96px] border border-b-0 border-[var(--border)] bg-[var(--panel)] px-[18px] py-2 text-[13px] font-bold text-[var(--accent)]"
      : "min-w-[96px] border border-b-0 border-[var(--border)] bg-[var(--base-alt)] px-[18px] py-2 text-[13px] text-[var(--muted)] hover:bg-[var(--panel)] hover:text-[var(--text-bright)]";

  const rangeButtonClasses = (active: boolean, first: boolean, last: boolean): string => {
    const radius = first ? "rounded-l-md" : last ? "rounded-r-md" : "";
    const rightBorder = last ? "border-r" : "border-r-0";
    const base = `border px-3 py-[5px] text-[13px] ${radius} ${rightBorder}`;
    return active
      ? `${base} border-[var(--accent)] bg-[var(--button)] font-bold text-[var(--accent)]`
      : `${base} border-[var(--border-strong)] bg-[var(--base-alt)] text-[var(--muted)] hover:bg-[var(--panel)] hover:text-[var(--text-bright)]`;
  };
</script>

{#if report && trackingEnabled()}
  <AppDialog open={true} onclose={close}>
    <DialogContent
      showCloseButton={false}
      class="sm:max-w-[1160px] max-h-[88vh] flex flex-col overflow-hidden"
      aria-labelledby={`${uid}-title`}
    >
      <DialogHeader
        id={`${uid}-title`}
        title={report.machine ? t.report_machine_title() : t.playtime_local()}
        icon={Timer}
        onclose={close}
      />
      <p class="dialog-text">
        {#if entry}<MessageText
            content={rich(t.report_account_description, { account: entry.display_name })}
            account={entry}
          />{:else}{t.report_machine_subtitle()}{/if}
      </p>

      <p class="m-0 py-1 text-[13px] font-bold text-[var(--green)]">
        <MessageText content={reportSummary(report)} />
      </p>
      <p class="m-0 py-0.5 text-xs font-semibold text-[var(--text-strong)]">
        <MessageText content={reportStatistics(report)} />
      </p>
      <p class="history-notice"><MessageText content={reportNotice(report)} /></p>

      <div class="flex border-b border-[var(--border)]" role="tablist">
        {#each tabs as [key, label] (key)}
          <button
            type="button"
            class={tabButtonClasses(app.reportTab === key)}
            role="tab"
            aria-selected={app.reportTab === key}
            onclick={() => (app.reportTab = key)}
          >
            {label}
          </button>
        {/each}
      </div>

      <div class="dialog-body">
        {#if app.reportTab === ReportTab.GAME}
          {#if report.machine}
            <DataTable
              headers={[
                t.report_game_label(),
                t.steam_ids_app(),
                t.accounts_label(),
                t.report_sessions_label(),
                t.playtime_title(),
                t.report_status_label(),
              ]}
              rows={report.machine_games.map((game) => [
                { text: game.name },
                { text: game.app_id },
                { text: String(game.accounts) },
                { text: String(game.sessions) },
                { text: formatSeconds(game.seconds) },
                { text: statusLabel(game.status) },
              ])}
              empty={t.report_games_empty()}
            />
          {:else}
            <DataTable
              headers={[
                t.report_game_label(),
                t.steam_ids_app(),
                t.report_sessions_label(),
                t.playtime_title(),
                t.report_status_label(),
              ]}
              rows={report.games.map((game) => [
                { text: game.name },
                { text: game.app_id },
                { text: String(game.sessions) },
                { text: formatSeconds(game.seconds) },
                { text: statusLabel(game.status) },
              ])}
              empty={t.report_games_empty()}
            />
          {/if}
        {:else if app.reportTab === ReportTab.ACCOUNT}
          <DataTable
            headers={[
              t.account_display_name(),
              t.account_login_name(),
              t.report_games_label(),
              t.playtime_title(),
              t.report_status_label(),
            ]}
            rows={report.accounts.map((entry) => [
              { avatar: entry, text: entry.display_name },
              { text: entry.account_name },
              { text: String(entry.games) },
              { text: formatSeconds(entry.seconds) },
              { text: statusLabel(entry.status) },
            ])}
            empty={t.report_accounts_empty()}
          />
        {:else if app.reportTab === ReportTab.DAILY}
          <div class="flex min-h-[260px] flex-col gap-2 pt-2.5">
            <div class="mt-2.5 mb-2 flex items-center gap-2.5">
              <span class="field-label">{t.report_range_label()}</span>
              <div class="flex">
                {#each [7, 30, 90] as days, index (days)}
                  <button
                    type="button"
                    class={rangeButtonClasses(app.chartRange === days, index === 0, index === 2)}
                    onclick={() => (app.chartRange = days)}
                  >
                    {t.report_days_count({ count: days })}
                  </button>
                {/each}
              </div>
            </div>
            <DailyChart points={dailyPoints} />
            <DataTable
              headers={[t.report_date_label(), t.report_sessions_label(), t.playtime_title()]}
              rows={[...dailyPoints]
                .reverse()
                .map((point) => [
                  { text: formatDate(point.day, false) },
                  { text: String(point.sessions) },
                  { text: formatSeconds(point.seconds) },
                ])}
              empty={t.report_daily_empty()}
            />
          </div>
        {:else}
          <DataTable
            headers={report.machine
              ? [
                  t.account_label(),
                  t.report_game_label(),
                  t.steam_ids_app(),
                  t.report_started_label(),
                  t.report_ended_label(),
                  t.report_duration_label(),
                  t.report_coverage_label(),
                  t.report_end_label(),
                ]
              : [
                  t.report_game_label(),
                  t.steam_ids_app(),
                  t.report_started_label(),
                  t.report_ended_label(),
                  t.report_duration_label(),
                  t.report_coverage_label(),
                  t.report_end_label(),
                ]}
            rows={report.sessions.map((session) => [
              ...(report.machine
                ? [{ avatar: session, text: session.display_name, title: sessionTooltip(session) }]
                : []),
              { text: session.game_name, title: sessionTooltip(session) },
              { text: session.app_id, title: sessionTooltip(session) },
              { text: formatDate(session.started_at), title: sessionTooltip(session) },
              {
                text:
                  session.ended_at === null
                    ? t.report_status_running()
                    : formatDate(session.ended_at),
                title: sessionTooltip(session),
              },
              { text: formatSeconds(session.seconds), title: sessionTooltip(session) },
              {
                text: coverageLabel(session.coverage),
                title: sessionTooltip(session),
              },
              { text: sessionEnd(session), title: sessionTooltip(session) },
            ])}
            empty={t.report_history_empty()}
          />
        {/if}
      </div>

      <footer class="dialog-foot">
        <button class="button" type="button" onclick={close}
          ><X size={15} /> {t.common_close()}</button
        >
      </footer>
    </DialogContent>
  </AppDialog>
{/if}
