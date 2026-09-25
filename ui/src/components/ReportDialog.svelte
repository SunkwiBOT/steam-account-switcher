<script lang="ts">
  import { app } from "../lib/state.svelte";
  import { formatSeconds } from "../lib/api";
  import DailyChart from "./DailyChart.svelte";
  import DataTable from "./DataTable.svelte";
  import { X } from "@lucide/svelte";

  const report = $derived(app.report);
  const tabs = $derived(
    report?.machine
      ? [
          ["game", "By Game"],
          ["account", "By Account"],
          ["daily", "Daily"],
          ["history", "History"],
        ]
      : [
          ["game", "By Game"],
          ["daily", "Daily"],
          ["history", "History"],
        ],
  );

  const dailyPoints = $derived(
    app.chartRange === 7
      ? (report?.daily7 ?? [])
      : app.chartRange === 30
        ? (report?.daily30 ?? [])
        : (report?.daily90 ?? []),
  );

  function close() {
    app.report = null;
  }
</script>

{#if report}
  <div class="modal">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="modal-backdrop" onclick={close}></div>
    <section class="dialog" role="dialog" aria-modal="true">
      <header class="dialog-head">
        <div>
          <h2 class="dialog-title">{report.title}</h2>
          <p class="dialog-subtitle">{report.subtitle}</p>
        </div>
        <button class="icon-button" type="button" aria-label="Close" onclick={close}>
          <X size={16} />
        </button>
      </header>

      <p class="playtime-summary">{report.summary_text}</p>
      <p class="playtime-stats">{report.statistics_text}</p>
      <p class="history-notice">{report.notice_text}</p>

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
        {#if app.reportTab === "game"}
          {#if report.machine}
            <DataTable
              headers={["Game", "Steam App ID", "Accounts", "Sessions", "Playtime", "Status"]}
              rows={report.machine_games.map((game) => [
                { text: game.name },
                { text: game.app_id },
                { text: String(game.accounts) },
                { text: String(game.sessions) },
                { text: game.seconds_text },
                { text: game.status },
              ])}
              empty="No local game session has been detected yet."
            />
          {:else}
            <DataTable
              headers={["Game", "Steam App ID", "Sessions", "Playtime", "Status"]}
              rows={report.games.map((game) => [
                { text: game.name },
                { text: game.app_id },
                { text: String(game.sessions) },
                { text: game.seconds_text },
                { text: game.status },
              ])}
              empty="No local game session has been detected yet."
            />
          {/if}
        {:else if app.reportTab === "account"}
          <DataTable
            headers={["Display Name", "Account Name", "Games", "Playtime", "Status"]}
            rows={report.accounts.map((entry) => [
              { avatar: entry, text: entry.display_name },
              { text: entry.account_name },
              { text: String(entry.games) },
              { text: entry.seconds_text },
              { text: entry.status },
            ])}
            empty="No local playtime recorded yet."
          />
        {:else if app.reportTab === "daily"}
          <div class="tab-panel">
            <div class="range-row">
              <span class="field-label">Range</span>
              <div class="range-group">
                {#each [7, 30, 90] as days (days)}
                  <button
                    type="button"
                    class:active={app.chartRange === days}
                    onclick={() => (app.chartRange = days)}
                  >
                    {days} days
                  </button>
                {/each}
              </div>
            </div>
            <DailyChart points={dailyPoints} />
            <DataTable
              headers={["Date", "Sessions", "Playtime"]}
              rows={[...dailyPoints]
                .reverse()
                .map((point) => [
                  { text: point.day },
                  { text: String(point.sessions) },
                  { text: formatSeconds(point.seconds) },
                ])}
              empty="No session in this range."
            />
          </div>
        {:else}
          <DataTable
            headers={report.machine
              ? [
                  "Account",
                  "Game",
                  "Steam App ID",
                  "Started",
                  "Ended",
                  "Duration",
                  "Coverage",
                  "End",
                ]
              : ["Game", "Steam App ID", "Started", "Ended", "Duration", "Coverage", "End"]}
            rows={report.sessions.map((session) => [
              ...(report.machine
                ? [{ avatar: session, text: session.display_name, title: session.tooltip }]
                : []),
              { text: session.game_name, title: session.tooltip },
              { text: session.app_id, title: session.tooltip },
              { text: session.started_text, title: session.tooltip },
              { text: session.ended_text, title: session.tooltip },
              { text: session.duration_text, title: session.tooltip },
              { text: session.coverage, title: session.tooltip },
              { text: session.end_reason, title: session.tooltip },
            ])}
            empty="No detailed session recorded yet. Detailed history starts when this version first detects a launch."
          />
        {/if}
      </div>

      <footer class="dialog-foot">
        <button class="button" type="button" onclick={close}>Close</button>
      </footer>
    </section>
  </div>
{/if}
