<script lang="ts">
  import { app, currentFolder } from "../lib/state.svelte";

  const folder = $derived(currentFolder());
</script>

<nav class="breadcrumb" aria-label="Folder path">
  <button
    type="button"
    class="crumb"
    class:current={app.folderId === null}
    onclick={() => (app.folderId = null)}
  >
    All accounts
  </button>
  {#if folder}
    <span class="crumb-sep">/</span>
    <button type="button" class="crumb current" onclick={() => (app.folderId = folder.id)}>
      {folder.name}
    </button>
  {/if}
</nav>
