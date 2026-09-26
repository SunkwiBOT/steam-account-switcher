<script lang="ts">
  import { checkForUpdate, openRepository } from "../lib/actions.svelte";
  import { app } from "../lib/state.svelte";
  import { Download, ExternalLink, RefreshCw } from "@lucide/svelte";
</script>

<div class="settings-cards">
  <section class="settings-card wide">
    <h3 class="settings-card-title">Updates</h3>
    <p class="settings-note">
      Installed version: <strong>{app.info.version ? `v${app.info.version}` : "Unknown"}</strong>
    </p>
    <p class="settings-note">
      {#if app.updateStage === "downloaded"}
        Version {app.updateVersion} is downloaded and waiting for a restart.
      {:else if app.updateStage === "downloading"}
        Downloading version {app.update?.version ?? ""}…
      {:else if app.update}
        Version {app.update.version} is available (you have {app.update.current_version}).
      {:else if app.updateCheck === "checking"}
        Checking for updates…
      {:else if app.updateCheck === "error"}
        Unable to check for updates. Check your connection or visit the releases page.
      {:else if app.updateCheck === "success"}
        This build is up to date.
      {:else}
        Updates have not been checked yet.
      {/if}
    </p>
    <div class="settings-actions">
      <button
        class="button"
        type="button"
        disabled={app.updateCheck === "checking"}
        onclick={checkForUpdate}
      >
        {#if app.updateCheck === "checking"}
          <span class="spinner tiny"></span>
        {:else}
          <RefreshCw size={14} /> Check now
        {/if}
      </button>
      {#if app.update}
        <button class="button primary" type="button" onclick={() => (app.updateDialog = true)}>
          <Download size={14} /> Download update
        </button>
      {/if}
      <button class="button" type="button" onclick={openRepository}>
        <ExternalLink size={14} /> Project page
      </button>
    </div>
    <p class="settings-note">
      Releases are published from the project's GitHub releases page. The build is downloaded and
      verified first, then you choose when to restart.
    </p>
  </section>
</div>
