<script lang="ts">
  import { installationLabel } from "../../lib/state/status";
  import MessageText from "../ui/MessageText.svelte";
  import { t, rich } from "../../lib/i18n/i18n.svelte";

  import SettingToggle from "./SettingToggle.svelte";
  import ActionButton from "../ui/ActionButton.svelte";
  import { app, EMPTY_SETTINGS, run, showToast } from "../../lib/state/state.svelte";
  import type { SettingsType } from "../../lib/types";
  import { Eye, EyeOff, HardDrive, Plus, Trash2 } from "@lucide/svelte";

  const settings = $derived<SettingsType>(app.snapshot?.settings ?? EMPTY_SETTINGS);
  const entries = $derived(settings.entries);

  let path = $state("");
  let adding = $state(false);
  let pending = $state<Record<string, boolean>>({});

  const add = async () => {
    if (!path.trim() || adding) {
      return;
    }
    adding = true;
    const result = await run("add_custom_installation", { path: path.trim() });
    adding = false;
    if (result) {
      showToast(t.installation_add_success(), true);
      path = "";
    }
  };

  const toggle = async (root: string, hidden: boolean): Promise<void> => {
    if (pending[root]) return;
    pending[root] = true;
    await run("set_installation_hidden", { root, hidden: !hidden });
    delete pending[root];
  };

  const remove = async (root: string, label: string): Promise<void> => {
    if (pending[root]) return;
    pending[root] = true;
    await run(
      "remove_custom_installation",
      { root },
      { success: rich(t.installation_remove_success, { installation: label }) },
    );
    delete pending[root];
  };
</script>

<div class="settings-cards">
  <section class="settings-card wide">
    <h3 class="settings-card-title">
      <HardDrive size={15} />
      {t.settings_installations_page_title()}
    </h3>
    <SettingToggle
      label={t.installation_selector_label()}
      checked={settings.show_installation_selector !== false}
      onchange={(showInstallationSelector) => run("update_settings", { showInstallationSelector })}
    />
    <div class="settings-list">
      {#if entries.length === 0}
        <p class="empty-tab">{t.installation_empty()}</p>
      {:else}
        {#each entries as entry (entry.root)}
          <div class="settings-row" class:hidden-entry={entry.hidden}>
            <span class="settings-label" title={entry.root}>{installationLabel(entry)}</span>
            {#if entry.custom}<span class="tag custom">{t.installation_custom()}</span>{/if}
            {#if entry.hidden}<span class="tag hidden">{t.installation_hidden()}</span>{/if}
            {#if entry.custom}
              <ActionButton
                icon={Trash2}
                label={t.common_remove()}
                busyLabel={t.common_removing()}
                busy={pending[entry.root] === true}
                disabled={false}
                className="button danger"
                onclick={() => remove(entry.root, installationLabel(entry))}
              />
            {:else}
              <ActionButton
                icon={entry.hidden ? Eye : EyeOff}
                label={entry.hidden ? t.common_show() : t.common_hide()}
                busyLabel={entry.hidden ? t.common_showing() : t.common_hiding()}
                busy={pending[entry.root] === true}
                onclick={() => toggle(entry.root, entry.hidden)}
              />
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
      <ActionButton
        icon={Plus}
        label={t.common_add()}
        busyLabel={t.common_adding()}
        busy={adding}
        disabled={!path.trim()}
        className="button primary"
        onclick={add}
      />
    </div>
    <p class="settings-note">
      <MessageText
        content={rich(t.installation_settings_description, { file: "config/loginusers.vdf" }, [
          "file",
        ])}
      />
    </p>
    <p class="settings-path">{settings.database_path ?? ""}</p>
  </section>
</div>
