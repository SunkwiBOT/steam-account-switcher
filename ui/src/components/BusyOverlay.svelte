<script lang="ts">
  import { app } from "../lib/state.svelte";
</script>

{#if app.busy.active}
  <div class="busy">
    <div class="spinner"></div>
    <p>{app.busy.text}</p>
  </div>
{/if}
