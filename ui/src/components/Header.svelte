<script lang="ts">
  import { addAccount, openRepository, updateButtonLabel } from "../lib/actions.svelte";
  import { openDropdown } from "../lib/menus.svelte";
  import { app, run } from "../lib/state.svelte";
  import { ChevronDown, Download, Plus, RefreshCw, Search, Settings } from "@lucide/svelte";

  let refreshing = $state(false);

  const snapshot = $derived(app.snapshot);
  const installations = $derived(snapshot?.installations ?? []);
  const selected = $derived(installations.find((entry) => entry.index === snapshot?.selected));
  const activeGames = $derived(snapshot?.active_games ?? []);
  const showUpdate = $derived(Boolean(app.update || app.updateStage));
  const updateLabel = $derived(updateButtonLabel());

  async function refresh() {
    refreshing = true;
    try {
      await run("refresh");
    } finally {
      refreshing = false;
    }
  }

  function openInstallations(event: MouseEvent): void {
    openDropdown({
      anchor: event.currentTarget as HTMLElement,
      value: String(snapshot?.selected ?? 0),
      options: installations.map((entry) => ({
        value: String(entry.index),
        label: entry.label,
      })),
      onSelect: (value) => run("select_installation", { index: Number(value) }),
    });
  }
</script>

<header class="app-header">
  <button
    class="brand"
    type="button"
    title="Open the project page on GitHub"
    aria-label="Open the project page on GitHub"
    onclick={openRepository}
  >
    <img class="brand-icon" src="/app-icon.png" alt="" draggable="false" />
  </button>

  <div class="header-actions">
    <button
      class="icon-button"
      type="button"
      title="Refresh accounts"
      disabled={refreshing}
      onclick={refresh}
    >
      {#if refreshing}
        <span class="spinner tiny"></span>
      {:else}
        <RefreshCw size={16} />
      {/if}
    </button>
    <button
      class="icon-button"
      type="button"
      title="Add a Steam account (Steam asks you to sign in)"
      disabled={Boolean(app.switchingId)}
      onclick={addAccount}
    >
      <Plus size={18} />
    </button>
    <button
      class="icon-button"
      type="button"
      title="Settings"
      onclick={() => (app.settingsOpen = true)}
    >
      <Settings size={16} />
    </button>
    {#if showUpdate}
      <button
        class="button primary"
        type="button"
        title={app.update
          ? `Install version ${app.update.version}`
          : "Restart to finish installing"}
        disabled={app.updateStage === "downloading"}
        onclick={() => (app.updateDialog = true)}
      >
        <Download size={15} />
        <span>{updateLabel}</span>
      </button>
    {/if}
    {#if activeGames.length > 0}
      <span class="active-game">
        {activeGames.map((game) => `${game.name} (${game.elapsed_text})`).join(", ")}
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
        <span>{selected?.label ?? "Installation"}</span>
        <span class="picker-caret"><ChevronDown size={14} /></span>
      </button>
    {/if}
    <label class="search-wrap">
      <span class="search-icon"><Search size={16} /></span>
      <input
        class="search"
        type="search"
        placeholder="Search accounts"
        autocomplete="off"
        spellcheck="false"
        bind:value={app.search}
      />
    </label>
  </div>
</header>
