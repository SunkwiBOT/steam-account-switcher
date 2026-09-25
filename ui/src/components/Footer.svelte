<script lang="ts">
  import { openMachineReport } from "../lib/actions.svelte";
  import { app, run } from "../lib/state.svelte";
  import { Monitor } from "@lucide/svelte";

  const snapshot = $derived(app.snapshot);
</script>

<footer class="footer">
  <button
    class="version"
    type="button"
    title="{app.info.name} {app.info.version} — open the project page"
    onclick={() => run("open_repository")}
  >
    v{app.info.version}
  </button>
  <span class="status">{snapshot?.status ?? ""}</span>
  {#if snapshot?.settings?.show_machine_playtime !== false}
    <button
      class="playtime-total"
      type="button"
      disabled={!snapshot?.tracker_ready}
      onclick={openMachineReport}
    >
      <Monitor size={14} />
      <span>This PC: {snapshot?.machine_text ?? "0m"}</span>
    </button>
  {/if}
</footer>
