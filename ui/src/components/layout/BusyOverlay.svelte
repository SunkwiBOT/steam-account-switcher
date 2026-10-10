<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";

  import { app } from "../../lib/state/state.svelte";
</script>

{#if app.busy.active || app.switchingId}
  <div class="busy" role="status" aria-live="polite" tabindex="-1">
    <div class="spinner"></div>
    <p>{app.busy.active ? app.busy.text : t.account_switch_pending()}</p>
  </div>
{/if}
