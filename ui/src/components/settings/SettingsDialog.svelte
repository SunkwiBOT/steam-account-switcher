<script lang="ts">
  import DialogHeader from "../dialogs/DialogHeader.svelte";
  import { focusTrap } from "../../lib/actions/focus-trap";
  import { t } from "../../lib/i18n/i18n.svelte";

  import { SettingsSection } from "../../lib/state/state.constants";
  import { app } from "../../lib/state/state.svelte";
  import {
    Download,
    FileText,
    Folder,
    HardDrive,
    Lock,
    Settings,
    SlidersHorizontal,
    Timer,
    X,
  } from "@lucide/svelte";
  import SettingsFolders from "./SettingsFolders.svelte";
  import SettingsPreferences from "./SettingsPreferences.svelte";
  import SettingsInstallations from "./SettingsInstallations.svelte";
  import SettingsUpdates from "./SettingsUpdates.svelte";
  import SettingsPlaytime from "./SettingsPlaytime.svelte";

  import SettingsLogs from "./SettingsLogs.svelte";

  import VaultControls from "../vault/VaultControls.svelte";
  import VaultBackups from "../vault/VaultBackups.svelte";

  const sections = $derived([
    {
      id: SettingsSection.PREFERENCES,
      label: t.settings_preferences_title(),
      icon: SlidersHorizontal,
    },
    { id: SettingsSection.FOLDERS, label: t.settings_folders_title(), icon: Folder },
    { id: SettingsSection.INSTALLATIONS, label: t.settings_installations_title(), icon: HardDrive },
    { id: SettingsSection.PLAYTIME, label: t.playtime_title(), icon: Timer },
    { id: SettingsSection.VAULT, label: t.guard_title(), icon: Lock },
    { id: SettingsSection.LOGS, label: t.settings_logs_title(), icon: FileText },
    { id: SettingsSection.UPDATES, label: t.settings_updates_title(), icon: Download },
  ]);
  const uid = $props.id();
</script>

{#if app.settingsOpen}
  <div class="modal" inert={app.vaultOpen}>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="modal-backdrop" onclick={() => (app.settingsOpen = false)}></div>
    <div
      class="dialog settings-dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby={`${uid}-title`}
      tabindex="-1"
      use:focusTrap={{ onEscape: () => (app.settingsOpen = false) }}
    >
      <DialogHeader
        id={`${uid}-title`}
        title={t.settings_title()}
        icon={Settings}
        onclose={() => (app.settingsOpen = false)}
      />

      <div class="settings-layout">
        <nav class="settings-nav" aria-label={t.settings_navigation_label()}>
          {#each sections as section (section.id)}
            <button
              type="button"
              class="settings-tab"
              class:active={app.settingsSection === section.id}
              onclick={() => (app.settingsSection = section.id)}
            >
              <section.icon size={15} />
              {section.label}
            </button>
          {/each}
        </nav>

        <div class="settings-body">
          {#if app.settingsSection === SettingsSection.PREFERENCES}
            <SettingsPreferences />
          {:else if app.settingsSection === SettingsSection.FOLDERS}
            <SettingsFolders />
          {:else if app.settingsSection === SettingsSection.INSTALLATIONS}
            <SettingsInstallations />
          {:else if app.settingsSection === SettingsSection.PLAYTIME}
            <SettingsPlaytime />
          {:else if app.settingsSection === SettingsSection.VAULT}
            <section class="settings-card">
              <h3 class="settings-card-title"><Lock size={15} /> {t.vault_title()}</h3>
              <VaultControls showSettings />
            </section>
            <VaultBackups />
          {:else if app.settingsSection === SettingsSection.LOGS}
            <SettingsLogs />
          {:else}
            <SettingsUpdates />
          {/if}
        </div>
      </div>

      <footer class="dialog-foot">
        <button class="button" type="button" onclick={() => (app.settingsOpen = false)}
          ><X size={15} /> {t.common_close()}</button
        >
      </footer>
    </div>
  </div>
{/if}
