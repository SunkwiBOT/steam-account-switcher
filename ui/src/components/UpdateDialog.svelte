<script lang="ts">
  import {
    applyUpdate,
    downloadUpdate,
    updateProgressPercent,
    updateProgressText,
  } from "../lib/actions.svelte";
  import { app } from "../lib/state.svelte";
  import { Power } from "@lucide/svelte";

  const percent = $derived(updateProgressPercent());
</script>

{#if app.updateStage}
  <div class="modal">
    <div class="modal-backdrop"></div>
    <section class="dialog small" role="dialog" aria-modal="true">
      {#if app.updateStage === "downloaded"}
        <h2 class="dialog-title">Version {app.updateVersion} downloaded</h2>
        <p class="dialog-text">Restart the application to finish installing it.</p>
        <div class="progress"><span class="progress-bar" style="width:100%"></span></div>
        <footer class="dialog-foot">
          <button class="button" type="button" onclick={() => (app.updateDialog = false)}>
            Restart later
          </button>
          <button class="button primary" type="button" onclick={applyUpdate}>
            <Power size={14} /> Restart now
          </button>
        </footer>
      {:else}
        <h2 class="dialog-title">Downloading update</h2>
        <p class="dialog-text">{updateProgressText()}</p>
        <div class="progress"><span class="progress-bar" style="width:{percent}%"></span></div>
        <footer class="dialog-foot">
          <button class="button" type="button" onclick={() => (app.updateDialog = false)}>
            Later
          </button>
        </footer>
      {/if}
    </section>
  </div>
{:else if app.updateDialog && app.update}
  <div class="modal">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="modal-backdrop" onclick={() => (app.updateDialog = false)}></div>
    <section class="dialog small" role="dialog" aria-modal="true">
      <h2 class="dialog-title">Version {app.update.version} available</h2>
      <p class="dialog-text">
        {app.update.notes?.trim() || "A newer build of Steam Account Switcher is available."}
      </p>
      <p class="dialog-subtitle">You are running {app.update.current_version}.</p>
      <footer class="dialog-foot">
        <button class="button" type="button" onclick={() => (app.updateDialog = false)}
          >Later</button
        >
        <button class="button primary" type="button" onclick={downloadUpdate}>Download</button>
      </footer>
    </section>
  </div>
{/if}
