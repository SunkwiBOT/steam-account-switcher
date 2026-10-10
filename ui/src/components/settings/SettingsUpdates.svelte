<script lang="ts">
  import MessageText from "../ui/MessageText.svelte";
  import { t, rich, formatDate } from "../../lib/i18n/i18n.svelte";

  import { UpdateCheck, UpdateInterval, UpdateStage } from "../../lib/state/state.constants";
  import AsyncIcon from "../ui/AsyncIcon.svelte";
  import {
    checkForUpdate,
    checkForUpdateIfDue,
    openRepository,
    repositoryUrl,
  } from "../../lib/actions/actions.svelte";
  import { app, EMPTY_SETTINGS, run } from "../../lib/state/state.svelte";
  import { dropdown, openDropdown } from "../../lib/state/menus.svelte";
  import {
    CheckCircle,
    ChevronDown,
    Download,
    ExternalLink,
    Power,
    RefreshCw,
  } from "@lucide/svelte";

  const settings = $derived(app.snapshot?.settings ?? EMPTY_SETTINGS);
  const intervals = $derived([
    { value: UpdateInterval.NEVER, label: t.updates_check_never() },
    { value: UpdateInterval.STARTUP, label: t.updates_check_startup() },
    { value: UpdateInterval.FIFTEEN_MINUTES, label: t.updates_check_15_minutes() },
    { value: UpdateInterval.THIRTY_MINUTES, label: t.updates_check_30_minutes() },
    { value: UpdateInterval.ONE_HOUR, label: t.updates_check_hourly() },
    { value: UpdateInterval.SIX_HOURS, label: t.updates_check_6_hours() },
    { value: UpdateInterval.ONE_DAY, label: t.updates_check_daily() },
    { value: UpdateInterval.ONE_WEEK, label: t.updates_check_weekly() },
  ]);
  const intervalLabel = $derived(
    intervals.find((option) => option.value === settings.update_interval)?.label ??
      t.updates_check_6_hours(),
  );
  let intervalBusy = $state(false);
  let intervalButton = $state<HTMLButtonElement | null>(null);

  const changeInterval = async (interval: UpdateInterval): Promise<void> => {
    if (intervalBusy || interval === settings.update_interval) return;
    intervalBusy = true;
    try {
      if (await run("set_update_interval", { interval })) void checkForUpdateIfDue();
    } finally {
      intervalBusy = false;
    }
  };

  const pickInterval = (event: MouseEvent): void => {
    const anchor = event.currentTarget;
    if (intervalBusy || !(anchor instanceof HTMLElement)) return;
    openDropdown({
      anchor,
      value: settings.update_interval,
      options: intervals,
      onSelect: (value) => {
        const interval = Object.values(UpdateInterval).find((choice) => choice === value);
        if (interval) void changeInterval(interval);
      },
    });
  };
</script>

<div class="settings-cards">
  <section class="settings-card wide">
    <h3 class="settings-card-title"><Download size={15} /> {t.settings_updates_title()}</h3>
    <p class="settings-note">
      {t.updates_installed_version()}
      <strong>{app.info.version ? `v${app.info.version}` : t.common_unknown()}</strong>
    </p>
    <p class="settings-note update-status">
      {#if app.updateStage === UpdateStage.INSTALLING}
        <MessageText content={rich(t.updates_installing_status, { version: app.updateVersion })} />
      {:else if app.updateStage === UpdateStage.DOWNLOADED}
        <MessageText content={rich(t.updates_downloaded_status, { version: app.updateVersion })} />
      {:else if app.updateStage === UpdateStage.DOWNLOADING}
        <MessageText
          content={rich(t.updates_downloading_status, { version: app.update?.version ?? "" })}
        />
      {:else if app.updateCheck === UpdateCheck.CHECKING}
        <AsyncIcon size={15} />
        {t.updates_check_pending()}
      {:else if app.update}
        <MessageText
          content={rich(t.updates_available_status, {
            version: app.update.version,
            current: app.update.current_version,
          })}
        />
      {:else if app.updateCheck === UpdateCheck.ERROR}
        {t.updates_check_error()}
      {:else if app.updateCheck === UpdateCheck.SUCCESS}
        <CheckCircle size={15} class="update-success" />
        {t.updates_current()}
      {:else}
        {t.updates_session_unchecked()}
      {/if}
    </p>
    <p class="settings-note">
      {#if app.updateCheckedAt}
        <MessageText
          content={rich(t.updates_last_check, { time: formatDate(app.updateCheckedAt) })}
        />
      {:else}
        {t.updates_not_checked()}
      {/if}
    </p>
    <div class="settings-actions">
      <button
        class="button"
        type="button"
        disabled={app.updateCheck === UpdateCheck.CHECKING || Boolean(app.updateStage)}
        onclick={checkForUpdate}
      >
        <AsyncIcon icon={RefreshCw} size={14} busy={app.updateCheck === UpdateCheck.CHECKING} />
        {app.updateCheck === UpdateCheck.CHECKING ? t.common_checking() : t.updates_check_action()}
      </button>
      {#if app.update}
        <button
          class="button primary"
          type="button"
          disabled={app.updateStage === UpdateStage.INSTALLING}
          onclick={() => (app.updateDialog = true)}
        >
          <AsyncIcon
            icon={app.updateStage === UpdateStage.DOWNLOADED ||
            app.updateStage === UpdateStage.INSTALLING
              ? Power
              : Download}
            size={14}
            busy={app.updateStage === UpdateStage.DOWNLOADING ||
              app.updateStage === UpdateStage.INSTALLING}
          />
          {app.updateStage === UpdateStage.DOWNLOADED
            ? t.updates_restart_action()
            : app.updateStage === UpdateStage.DOWNLOADING
              ? t.common_downloading()
              : app.updateStage === UpdateStage.INSTALLING
                ? t.common_restarting()
                : t.updates_download_action()}
        </button>
      {/if}
      <button class="button" type="button" onclick={openRepository}>
        <ExternalLink size={14} />
        {t.project_open_action()}
      </button>
    </div>
    <p class="settings-note">
      {#each rich(t.updates_source, { github: "GitHub" }) as part}{#if part.bold}<a
            class="inline-link"
            href={repositoryUrl()}
            onclick={(event) => {
              event.preventDefault();
              void openRepository();
            }}>{part.text}</a
          >{:else}{part.text}{/if}{/each}
    </p>
    <div class="settings-update-frequency">
      <span class="field-label">{t.updates_schedule_label()}</span>
      <button
        class="select-button"
        type="button"
        aria-label={t.updates_schedule_label()}
        aria-haspopup="listbox"
        aria-expanded={dropdown.open && dropdown.anchor === intervalButton}
        aria-busy={intervalBusy}
        bind:this={intervalButton}
        disabled={intervalBusy}
        onclick={pickInterval}
      >
        <span>{intervalBusy ? t.common_saving() : intervalLabel}</span>
        <span class="picker-caret"
          ><AsyncIcon icon={ChevronDown} busy={intervalBusy} size={14} /></span
        >
      </button>
      <p class="settings-note">
        {settings.update_interval === UpdateInterval.NEVER
          ? t.updates_checks_disabled()
          : settings.update_interval === UpdateInterval.STARTUP
            ? t.updates_checks_startup_description()
            : t.updates_checks_description()}
      </p>
    </div>
  </section>
</div>
