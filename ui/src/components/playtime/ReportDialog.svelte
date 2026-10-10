<script lang="ts">
  import DialogHeader from "../dialogs/DialogHeader.svelte";
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
</script>

{#if report && trackingEnabled()}
  <div class="modal">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="modal-backdrop" onclick={close}></div>
    <div
      class="dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby={`${uid}-title`}
      tabindex="-1"
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

      <p class="playtime-summary"><MessageText content={reportSummary(report)} /></p>
      <p class="playtime-stats"><MessageText content={reportStatistics(report)} /></p>
      <p class="history-notice"><MessageText content={reportNotice(report)} /></p>

      <div class="tabs" role="tablist">
        {#each tabs as [key, label] (key)}
          <button
            type="button"
            class:active={app.reportTab === key}
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
          <div class="tab-panel">
            <div class="range-row">
              <span class="field-label">{t.report_range_label()}</span>
              <div class="range-group">
                {#each [7, 30, 90] as days (days)}
                  <button
                    type="button"
                    class:active={app.chartRange === days}
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
              { text: coverageLabel(session.coverage), title: sessionTooltip(session) },
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
    </div>
  </div>
{/if}
