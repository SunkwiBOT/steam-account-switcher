<script lang="ts">
  import DialogHeader from "../dialogs/DialogHeader.svelte";
  import MessageText from "../ui/MessageText.svelte";
  import { t, rich } from "../../lib/i18n/i18n.svelte";

  import { UpdateStage } from "../../lib/state/state.constants";

  import ActionButton from "../ui/ActionButton.svelte";
  import {
    applyUpdate,
    downloadUpdate,
    updateProgressPercent,
    updateProgressText,
  } from "../../lib/actions/actions.svelte";
  import { app } from "../../lib/state/state.svelte";
  import { Check, Clock, Download, Power } from "@lucide/svelte";

  const percent = $derived(updateProgressPercent());
  const uid = $props.id();
</script>

{#if app.updateStage}
  <div class="modal">
    <div class="modal-backdrop"></div>
    <div
      class="dialog small"
      role="dialog"
      aria-modal="true"
      aria-labelledby={`${uid}-title`}
      tabindex="-1"
    >
      {#if app.updateStage === UpdateStage.DOWNLOADED || app.updateStage === UpdateStage.INSTALLING}
        <DialogHeader
          id={`${uid}-title`}
          title={rich(t.updates_downloaded_title, { version: app.updateVersion })}
          icon={Check}
        />
        <p class="dialog-text">{t.updates_restart_description()}</p>
        <div class="progress"><span class="progress-bar" style="width:100%"></span></div>
        <footer class="dialog-foot">
          <button
            class="button"
            type="button"
            disabled={app.updateStage === UpdateStage.INSTALLING}
            onclick={() => (app.updateDialog = false)}
          >
            <Clock size={15} />
            {t.updates_restart_later()}
          </button>
          <ActionButton
            className="button primary"
            icon={Power}
            label={t.updates_restart_now()}
            busyLabel={t.common_restarting()}
            busy={app.updateStage === UpdateStage.INSTALLING}
            onclick={applyUpdate}
          />
        </footer>
      {:else}
        <DialogHeader id={`${uid}-title`} title={t.updates_download_title()} icon={Download} busy />
        <p class="dialog-text">{updateProgressText()}</p>
        <div class="progress"><span class="progress-bar" style="width:{percent}%"></span></div>
        <footer class="dialog-foot">
          <button class="button" type="button" onclick={() => (app.updateDialog = false)}>
            <Clock size={15} />
            {t.common_later()}
          </button>
        </footer>
      {/if}
    </div>
  </div>
{:else if app.updateDialog && app.update}
  <div class="modal">
    <div class="modal-backdrop"></div>
    <div
      class="dialog small"
      role="dialog"
      aria-modal="true"
      aria-labelledby={`${uid}-title`}
      tabindex="-1"
    >
      <DialogHeader
        id={`${uid}-title`}
        title={rich(t.updates_available_title, { version: app.update.version })}
        icon={Download}
      />
      <p class="dialog-text">
        {app.update.notes?.trim() || t.updates_available_description()}
      </p>
      <p class="dialog-subtitle">
        <MessageText
          content={rich(t.updates_current_status, { version: app.update.current_version })}
        />
      </p>
      <footer class="dialog-foot">
        <button class="button" type="button" onclick={() => (app.updateDialog = false)}
          ><Clock size={15} /> {t.common_later()}</button
        >
        <button class="button primary" type="button" onclick={downloadUpdate}
          ><Download size={15} /> {t.common_download()}</button
        >
      </footer>
    </div>
  </div>
{/if}
