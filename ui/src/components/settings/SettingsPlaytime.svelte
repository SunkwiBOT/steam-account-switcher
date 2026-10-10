<script lang="ts">
  import { Timer, Trash2 } from "@lucide/svelte";
  import { t, rich } from "../../lib/i18n/i18n.svelte";
  import { installationLabel } from "../../lib/state/status";
  import { app, EMPTY_SETTINGS, run } from "../../lib/state/state.svelte";
  import { askConfirm } from "../../lib/state/dialogs.svelte";
  import ActionButton from "../ui/ActionButton.svelte";
  import CopyButton from "../ui/CopyButton.svelte";
  import MessageText from "../ui/MessageText.svelte";
  import SettingToggle from "./SettingToggle.svelte";

  const settings = $derived(app.snapshot?.settings ?? EMPTY_SETTINGS);
  const hasStoredHistory = $derived(
    settings.tracking_history.some((history) => history.account_count > 0),
  );
  let clearing = $state(false);
  let trackingBusy = $state(false);

  const setTracking = async (enabled: boolean): Promise<void> => {
    if (trackingBusy || clearing) return;
    trackingBusy = true;
    try {
      await run("set_tracking_enabled", { enabled });
    } finally {
      trackingBusy = false;
    }
  };
  const uid = $props.id();

  const clearData = async (): Promise<void> => {
    if (clearing || trackingBusy) return;
    const answer = await askConfirm({
      title: t.playtime_clear_title(),
      text: settings.tracking_enabled
        ? t.playtime_clear_tracking_confirm()
        : t.playtime_clear_paused_confirm(),
      confirmLabel: t.playtime_clear_action(),
      danger: true,
    });
    if (!answer || clearing || trackingBusy) return;
    clearing = true;
    try {
      if (
        await run(
          "clear_playtime_data",
          { confirmed: true },
          { success: t.playtime_clear_success() },
        )
      )
        app.report = null;
    } finally {
      clearing = false;
    }
  };
</script>

<section class="settings-card">
  <h3 class="settings-card-title"><Timer size={15} /> {t.playtime_title()}</h3>
  <SettingToggle
    label={t.playtime_tracking_label()}
    checked={settings.tracking_enabled !== false}
    disabled={clearing}
    onchange={setTracking}
  />
  <p class="settings-note">
    {settings.tracking_enabled !== false
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
      disabled={trackingBusy || !hasStoredHistory}
      className="button danger"
      onclick={clearData}
    />
  </div>
</section>
