<script lang="ts">
  import { app, emptyMessage, visibleAccounts, visibleFolders } from "../lib/state.svelte";
  import AccountCard from "./AccountCard.svelte";
  import FolderCard from "./FolderCard.svelte";
  import { ArrowLeft } from "@lucide/svelte";

  const entries = $derived(visibleAccounts());
  const folders = $derived(visibleFolders());
  const insideFolder = $derived(app.folderId !== null && !app.search.trim());
  const empty = $derived(entries.length === 0 && folders.length === 0 && !insideFolder);
</script>

<div class="cards" class:locked={Boolean(app.switchingId)}>
  {#each folders as folder (folder.id)}
    <FolderCard {folder} />
  {/each}

  {#if insideFolder}
    <button
      type="button"
      class="card back-card"
      title="Leave this folder"
      onclick={() => (app.folderId = null)}
    >
      <span class="folder-icon" aria-hidden="true"><ArrowLeft size={24} /></span>
      <span class="card-name">Back</span>
    </button>
  {/if}

  {#each entries as entry (entry.steam_id)}
    <AccountCard {entry} />
  {/each}
</div>

{#if empty}
  <p class="empty-state">{emptyMessage()}</p>
{/if}
