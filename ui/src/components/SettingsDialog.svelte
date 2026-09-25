<script lang="ts">
  import { app } from "../lib/state.svelte";
  import { X } from "@lucide/svelte";
  import SettingsFolders from "./SettingsFolders.svelte";
  import SettingsGeneral from "./SettingsGeneral.svelte";
  import SettingsInstallations from "./SettingsInstallations.svelte";
  import SettingsUpdates from "./SettingsUpdates.svelte";

  const SECTIONS = [
    { id: "general", label: "General" },
    { id: "folders", label: "Folders" },
    { id: "installations", label: "Installations" },
    { id: "updates", label: "Updates" },
  ];
</script>

{#if app.settingsOpen}
  <div class="modal">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="modal-backdrop" onclick={() => (app.settingsOpen = false)}></div>
    <section class="dialog settings-dialog" role="dialog" aria-modal="true">
      <header class="dialog-head">
        <h2 class="dialog-title">Settings</h2>
        <button
          class="icon-button"
          type="button"
          aria-label="Close"
          onclick={() => (app.settingsOpen = false)}
        >
          <X size={16} />
        </button>
      </header>

      <div class="settings-layout">
        <nav class="settings-nav" aria-label="Settings sections">
          {#each SECTIONS as section (section.id)}
            <button
              type="button"
              class="settings-tab"
              class:active={app.settingsSection === section.id}
              onclick={() => (app.settingsSection = section.id)}
            >
              {section.label}
            </button>
          {/each}
        </nav>

        <div class="settings-body">
          {#if app.settingsSection === "general"}
            <SettingsGeneral />
          {:else if app.settingsSection === "folders"}
            <SettingsFolders />
          {:else if app.settingsSection === "installations"}
            <SettingsInstallations />
          {:else}
            <SettingsUpdates />
          {/if}
        </div>
      </div>

      <footer class="dialog-foot">
        <button class="button" type="button" onclick={() => (app.settingsOpen = false)}
          >Close</button
        >
      </footer>
    </section>
  </div>
{/if}
