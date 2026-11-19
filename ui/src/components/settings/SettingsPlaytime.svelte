<script lang="ts">
  import { Timer, Trash2 } from "@lucide/svelte";
  import { t, rich } from "../../lib/i18n/i18n.svelte";
  import { installationLabel } from "../../lib/state/status";
  import { app, run } from "../../lib/state/state.svelte";
  import { EMPTY_SETTINGS } from "../../lib/state/settings";
  import { askConfirm } from "../../lib/state/dialogs.svelte";
  import { settingsDraft } from "../../lib/state/settings-draft.svelte";
  import ActionButton from "../ui/ActionButton.svelte";
  import CopyButton from "../ui/CopyButton.svelte";
  import MessageText from "../ui/MessageText.svelte";
  import SettingToggle from "./SettingToggle.svelte";

  const settings = $derived(app.snapshot?.settings ?? EMPTY_SETTINGS);
  const hasStoredHistory = $derived(
    settings.tracking_history.some((history) => history.account_count > 0),
  );
  const draft = $derived(settingsDraft.value);
  let clearing = $state(false);

  const clearData = (): void => {
    if (clearing || settingsDraft.saving) return;
    askConfirm({
      title: t.playtime_clear_title(),
      text: settings.tracking_enabled
        ? t.playtime_clear_tracking_confirm()
        : t.playtime_clear_paused_confirm(),
      confirmLabel: t.playtime_clear_action(),
      busyLabel: t.playtime_clear_pending(),
      danger: true,
      action: async () => {
        if (clearing || settingsDraft.saving) return false;
        clearing = true;
        try {
          const result = await run("clear_playtime_data", undefined, {
            successTitle: t.playtime_clear_success(),
          });
          if (result) app.report = null;
          return result !== null;
        } finally {
          clearing = false;
        }
      },
    });
  };
  const uid = $props.id();
</script>

<section class="settings-card">
  <h3 class="settings-card-title"><Timer size={15} /> {t.playtime_title()}</h3>
  <SettingToggle
    label={t.playtime_tracking_label()}
    checked={draft.trackingEnabled}
    disabled={clearing || settingsDraft.saving}
    onchange={async (enabled) => {
      settingsDraft.set("trackingEnabled", enabled);
    }}
  />
  <p class="settings-note">
    {draft.trackingEnabled
      ? t.playtime_tracking_enabled_description()
      : t.playtime_tracking_disabled_description()}
  </p>
  <div class="vault-path">
    <label class="field-label" for={`${uid}-database`}>
      <MessageText content={t.playtime_data_file()} />
    </label>
    <div class="vault-path-row">
      <input
        id={`${uid}-database`}
        class="input"
        readonly
        value={settings.database_path}
        title={settings.database_path}
      />
      <CopyButton
        valueLabel={t.playtime_data_file()}
        copyKey={`playtime-path:${settings.database_path}`}
        label={t.playtime_path_copy()}
        read={() => settings.database_path}
      />
    </div>
  </div>
  {#each settings.tracking_history as history, index (history.installation?.root ?? `stored-${index}`)}
    <p class="settings-note">
      <MessageText
        content={history.installation
          ? rich(t.playtime_installation_history, {
              installation: installationLabel(history.installation),
              accounts: t.accounts_count({ count: history.account_count }),
            })
          : rich(t.playtime_removed_installation_history, {
              accounts: t.accounts_count({ count: history.account_count }),
            })}
      />
    </p>
  {:else}
    <p class="settings-note">{t.playtime_no_installation()}</p>
  {/each}
  <p class="settings-note">{t.playtime_local_data_description()}</p>
  <div class="settings-actions">
    <ActionButton
      icon={Trash2}
      label={t.playtime_clear_action()}
      busyLabel={t.playtime_clear_pending()}
      busy={clearing}
      disabled={settingsDraft.saving || !hasStoredHistory}
      className="button danger"
      onclick={clearData}
    />
  </div>
</section>
