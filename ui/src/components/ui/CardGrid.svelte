<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";

  import {
    app,
    currentFolder,
    emptyMessage,
    visibleAccounts,
    visibleFolders,
  } from "../../lib/state/state.svelte";
  import AccountCard from "../accounts/AccountCard.svelte";
  import FolderCard from "../folders/FolderCard.svelte";
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
      title={t.folder_leave()}
      onclick={() => (app.folderId = null)}
    >
      <span class="folder-icon" aria-hidden="true"><ArrowLeft size={24} /></span>
      <span class="card-name">{t.common_back()}</span>
    </button>
  {/if}

  {#each entries as entry (entry.steam_id)}
    <AccountCard {entry} />
  {/each}
</div>

{#if insideFolder && currentFolder()?.system}
  <p class="settings-note">
    {t.guard_saved_folder_description()}
  </p>
{/if}

{#if empty}
  <p class="empty-state">{emptyMessage()}</p>
{/if}
