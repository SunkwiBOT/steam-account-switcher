<script lang="ts">
  import { openFolderMenu } from "../lib/menus.svelte";
  import { app } from "../lib/state.svelte";
  import { Folder } from "@lucide/svelte";

  let { folder } = $props();
</script>

<button
  type="button"
  class="card folder-card"
  title="Open folder · right click to rename or delete"
  onclick={() => (app.folderId = folder.id)}
  oncontextmenu={(event) => openFolderMenu(event, folder.id)}
>
  <span class="folder-icon" aria-hidden="true"><Folder size={32} /></span>
  <span class="card-name">{folder.name}</span>
  <div class="card-meta">
    <span>{folder.account_count} account{folder.account_count === 1 ? "" : "s"}</span>
  </div>
</button>
