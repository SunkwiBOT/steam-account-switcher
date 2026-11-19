<script lang="ts">
  import { t, folderLabel } from "../../lib/i18n/i18n.svelte";

  import { app } from "../../lib/state/state.svelte";
  import { currentFolder } from "../../lib/state/folders.svelte";

  const folder = $derived(currentFolder());
</script>

<nav class="breadcrumb" aria-label={t.folder_path()}>
  <button
    type="button"
    class="crumb"
    class:current={app.folderId === null}
    onclick={() => (app.folderId = null)}
  >
    {t.accounts_all()}
  </button>
  {#if folder}
    <span class="crumb-sep">/</span>
    <button type="button" class="crumb current" onclick={() => (app.folderId = folder.id)}>
      {folderLabel(folder)}
    </button>
  {/if}
</nav>
