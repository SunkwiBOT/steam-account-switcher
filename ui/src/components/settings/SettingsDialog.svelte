<script lang="ts">
  import DialogHeader from "../dialogs/DialogHeader.svelte";
  import AppDialog from "../dialogs/AppDialog.svelte";
  import { DialogContent } from "$lib/components/ui/dialog/index.js";
  import { Tabs, TabsList, TabsTrigger, TabsContent } from "$lib/components/ui/tabs/index.js";
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
  import UnsavedBar from "../ui/UnsavedBar.svelte";
  import { settingsDraft } from "../../lib/state/settings-draft.svelte";

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

  const close = (): void => {
    if (settingsDraft.saving || settingsDraft.dirty) return;
    app.settingsOpen = false;
  };
</script>

<AppDialog open={app.settingsOpen} onclose={close}>
  <DialogContent
    showCloseButton={false}
    class="sm:max-w-3xl max-h-[86vh] flex flex-col overflow-hidden"
    aria-labelledby={`${uid}-title`}
  >
    <DialogHeader id={`${uid}-title`} title={t.settings_title()} icon={Settings} onclose={close} />

    <div class="settings-layout">
      <Tabs
        value={app.settingsSection}
        onValueChange={(value) => (app.settingsSection = value as SettingsSection)}
        orientation="vertical"
        class="contents"
      >
        <TabsList aria-label={t.settings_navigation_label()}>
          {#each sections as section (section.id)}
            <TabsTrigger value={section.id}>
              <section.icon size={15} />
              {section.label}
            </TabsTrigger>
          {/each}
        </TabsList>

        <div class="settings-body">
          <TabsContent value={SettingsSection.PREFERENCES}>
            {#if app.settingsSection === SettingsSection.PREFERENCES}<SettingsPreferences />{/if}
          </TabsContent>
          <TabsContent value={SettingsSection.FOLDERS}>
            {#if app.settingsSection === SettingsSection.FOLDERS}<SettingsFolders />{/if}
          </TabsContent>
          <TabsContent value={SettingsSection.INSTALLATIONS}>
            {#if app.settingsSection === SettingsSection.INSTALLATIONS}<SettingsInstallations
              />{/if}
          </TabsContent>
          <TabsContent value={SettingsSection.PLAYTIME}>
            {#if app.settingsSection === SettingsSection.PLAYTIME}<SettingsPlaytime />{/if}
          </TabsContent>
          <TabsContent value={SettingsSection.VAULT}>
            {#if app.settingsSection === SettingsSection.VAULT}
              <section class="settings-card">
                <h3 class="settings-card-title"><Lock size={15} /> {t.vault_title()}</h3>
                <VaultControls showSettings />
              </section>
              <VaultBackups />
            {/if}
          </TabsContent>
          <TabsContent value={SettingsSection.LOGS}>
            {#if app.settingsSection === SettingsSection.LOGS}<SettingsLogs />{/if}
          </TabsContent>
          <TabsContent value={SettingsSection.UPDATES}>
            {#if app.settingsSection === SettingsSection.UPDATES}<SettingsUpdates />{/if}
          </TabsContent>
        </div>
      </Tabs>
    </div>

    {#if settingsDraft.dirty}
      <UnsavedBar
        saving={settingsDraft.saving}
        oncancel={settingsDraft.cancel}
        onsave={settingsDraft.save}
      />
    {/if}

    <footer class="dialog-foot">
      <button class="button" type="button" onclick={close}
        ><X size={15} /> {t.common_close()}</button
      >
    </footer>
  </DialogContent>
</AppDialog>
