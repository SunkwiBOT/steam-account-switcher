<script lang="ts">
  import { app } from "../lib/state.svelte";
</script>

{#if app.toast.visible}
  <div class="toast" class:ok={app.toast.ok}>{app.toast.text}</div>
{/if}
