<script lang="ts">
  import { formatSeconds } from "../../lib/api/api";
  import { t } from "../../lib/i18n/i18n.svelte";

  import { openMachineReport, openRepository } from "../../lib/actions/actions.svelte";
  import { app, trackingEnabled } from "../../lib/state/state.svelte";
  import { Monitor } from "@lucide/svelte";
  import AsyncIcon from "../ui/AsyncIcon.svelte";

  const snapshot = $derived(app.snapshot);
</script>

<footer class="footer">
  <button
    class="version"
    type="button"
    title="{app.info.name} {app.info.version} {t.footer_project_link()}"
    onclick={openRepository}
  >
    v{app.info.version}
  </button>
  {#if trackingEnabled()}
    <button
      class="playtime-total"
      type="button"
      disabled={!snapshot?.installations.length || Boolean(app.reportLoadingId)}
      aria-busy={app.reportLoadingId === "machine"}
      onclick={openMachineReport}
    >
      <AsyncIcon icon={Monitor} size={14} busy={app.reportLoadingId === "machine"} />
      <span
        >{#if app.reportLoadingId === "machine"}{t.playtime_loading()}{:else}{t.footer_playtime_label()}
          <strong>{formatSeconds(snapshot?.machine_seconds ?? 0)}</strong>{/if}</span
      >
    </button>
  {/if}
</footer>
