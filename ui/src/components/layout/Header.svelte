<script lang="ts">
  import { installationLabel } from "../../lib/state/status";
  import { t } from "../../lib/i18n/i18n.svelte";

  import { formatSeconds } from "../../lib/api/api";
  import { UpdateStage } from "../../lib/state/state.constants";
  import AsyncIcon from "../ui/AsyncIcon.svelte";
  import { addAccount, openRepository, updateButtonLabel } from "../../lib/actions/actions.svelte";
  import { openDropdown } from "../../lib/state/menus.svelte";
  import { app, run, trackingEnabled } from "../../lib/state/state.svelte";
  import { ChevronDown, Download, Plus, Power, RefreshCw, Search, Settings } from "@lucide/svelte";

  let refreshing = $state(false);

  const snapshot = $derived(app.snapshot);
  const installations = $derived(snapshot?.installations ?? []);
  const selected = $derived(installations.find((entry) => entry.index === snapshot?.selected));
  const activeGames = $derived(snapshot?.active_games ?? []);
  const showUpdate = $derived(Boolean(app.update || app.updateStage));
  const updateLabel = $derived(updateButtonLabel());

  const refresh = async () => {
    if (refreshing) return;
    refreshing = true;
    try {
      await run("refresh");
    } finally {
      refreshing = false;
    }
  };

  const openInstallations = (event: MouseEvent): void => {
    const anchor = event.currentTarget;
    if (!(anchor instanceof HTMLElement)) return;
    openDropdown({
      anchor,
      value: String(snapshot?.selected ?? 0),
      options: installations.map((entry) => ({
        value: String(entry.index),
        label: installationLabel(entry),
      })),
      onSelect: (value) => run("select_installation", { index: Number(value) }),
    });
  };
</script>

<header class="app-header">
  <button
    class="brand"
    type="button"
    title={t.project_open_tooltip()}
    aria-label={t.project_open_tooltip()}
    onclick={openRepository}
  >
    <img class="brand-icon" src="/app-icon.png" alt="" draggable="false" />
  </button>

  <div class="header-actions">
    <button
      class="icon-button"
      type="button"
      title={refreshing ? t.accounts_refresh_pending() : t.accounts_refresh()}
      aria-label={refreshing ? t.accounts_refresh_pending() : t.accounts_refresh()}
      aria-busy={refreshing}
      disabled={refreshing}
      onclick={refresh}
    >
      <AsyncIcon icon={RefreshCw} size={16} busy={refreshing} />
    </button>
    <button
      class="icon-button"
      type="button"
      title={t.account_add_title()}
      aria-label={t.account_add_title()}
      disabled={Boolean(app.switchingId)}
      onclick={addAccount}
    >
      <Plus size={18} />
    </button>
    <button
      class="icon-button"
      type="button"
      title={t.settings_title()}
      aria-label={t.settings_title()}
      onclick={() => (app.settingsOpen = true)}
    >
      <Settings size={16} />
    </button>
    {#if showUpdate}
      <button
        class="button primary"
        type="button"
        title={app.update
          ? t.updates_install_tooltip({ version: app.update.version })
          : t.updates_restart_tooltip()}
        disabled={app.updateStage === UpdateStage.DOWNLOADING ||
          app.updateStage === UpdateStage.INSTALLING}
        onclick={() => (app.updateDialog = true)}
      >
        <AsyncIcon
          icon={app.updateStage === UpdateStage.DOWNLOADED ||
          app.updateStage === UpdateStage.INSTALLING
            ? Power
            : Download}
          size={15}
          busy={app.updateStage === UpdateStage.DOWNLOADING ||
            app.updateStage === UpdateStage.INSTALLING}
        />
        <span>{updateLabel}</span>
      </button>
    {/if}
    {#if trackingEnabled() && activeGames.length > 0}
      <span class="active-game">
        {#each activeGames as game, index}
          {#if index > 0},
          {/if}<strong>{game.name}</strong> ({formatSeconds(game.elapsed_seconds)})
        {/each}
      </span>
    {/if}
  </div>

  <div class="header-right">
    {#if snapshot?.settings?.show_installation_selector !== false}
      <button
        class="select-button"
        type="button"
        aria-haspopup="listbox"
        onclick={openInstallations}
      >
        <span>{selected ? installationLabel(selected) : t.installation_label()}</span>
        <span class="picker-caret"><ChevronDown size={14} /></span>
      </button>
    {/if}
    <label class="search-wrap">
      <span class="search-icon"><Search size={16} /></span>
      <input
        class="search"
        type="search"
        placeholder={t.accounts_search_placeholder()}
        autocomplete="off"
        spellcheck="false"
        bind:value={app.search}
      />
    </label>
  </div>
</header>
