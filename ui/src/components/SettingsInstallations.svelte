<script lang="ts">
  import { app, run, showToast } from "../lib/state.svelte";
  import type { Settings } from "../lib/types";
  import { Eye, EyeOff, Plus, Trash2 } from "@lucide/svelte";

  const EMPTY_SETTINGS: Settings = {
    show_installation_selector: true,
    show_machine_playtime: true,
    entries: [],
    settings_path: "",
  };
  const settings = $derived<Settings>(app.snapshot?.settings ?? EMPTY_SETTINGS);
  const entries = $derived(settings.entries);

  let path = $state("");
  let pending = $state("");

  async function add() {
    if (!path.trim()) {
      return;
    }
    pending = "add";
    const result = await run(
      "add_custom_installation",
      { path: path.trim() },
      { busy: "Checking folder…" },
    );
    pending = "";
    if (result) {
      showToast("Steam installation added.", true);
      path = "";
    }
  }

  async function toggle(root: string, hidden: boolean): Promise<void> {
    pending = root;
    await run("set_installation_hidden", { root, hidden: !hidden });
    pending = "";
  }

  async function remove(root: string): Promise<void> {
    pending = root;
    await run("remove_custom_installation", { root });
    pending = "";
  }
</script>

<div class="settings-cards">
  <section class="settings-card wide">
    <h3 class="settings-card-title">Steam installations</h3>
    <div class="settings-list">
      {#if entries.length === 0}
        <p class="empty-tab">No Steam installation detected yet.</p>
      {:else}
        {#each entries as entry (entry.root)}
          <div class="settings-row" class:hidden-entry={entry.hidden}>
            <span class="settings-label" title={entry.root}>{entry.label}</span>
            {#if entry.custom}<span class="tag custom">custom</span>{/if}
            {#if entry.hidden}<span class="tag hidden">hidden</span>{/if}
            {#if entry.custom}
              <button
                class="button danger"
                type="button"
                disabled={pending === entry.root}
                onclick={() => remove(entry.root)}
              >
                <Trash2 size={14} /> Remove
              </button>
            {:else}
              <button
                class="button"
                type="button"
                disabled={pending === entry.root}
                onclick={() => toggle(entry.root, entry.hidden)}
              >
                {#if entry.hidden}
                  <Eye size={14} /> Show
                {:else}
                  <EyeOff size={14} /> Hide
                {/if}
              </button>
            {/if}
          </div>
        {/each}
      {/if}
    </div>
    <div class="settings-add">
      <input
        class="input"
        type="text"
        placeholder="/home/user/.steam/steam"
        spellcheck="false"
        bind:value={path}
      />
      <button class="button primary" type="button" disabled={pending === "add"} onclick={add}>
        {#if pending === "add"}
          <span class="spinner tiny"></span>
        {:else}
          <Plus size={14} /> Add
        {/if}
      </button>
    </div>
    <p class="settings-note">
      Hide the installations you never use, or add a Steam folder the detector cannot guess (it must
      contain <code>config/loginusers.vdf</code>).
    </p>
    <p class="settings-path">{settings.settings_path ?? ""}</p>
  </section>
</div>
