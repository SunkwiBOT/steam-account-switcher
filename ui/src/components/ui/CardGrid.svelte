<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";

  import { cn } from "$lib/utils";
  import { app } from "../../lib/state/state.svelte";
  import { currentFolder, visibleFolders } from "../../lib/state/folders.svelte";
  import { emptyMessage, visibleAccounts } from "../../lib/state/accounts.svelte";
  import AccountCard from "../accounts/AccountCard.svelte";
  import FolderCard from "../folders/FolderCard.svelte";
  import { ArrowLeft } from "@lucide/svelte";

  const entries = $derived(visibleAccounts());
  const folders = $derived(visibleFolders());
  const insideFolder = $derived(app.folderId !== null && !app.search.trim());
  const empty = $derived(entries.length === 0 && folders.length === 0 && !insideFolder);
  const locked = $derived(Boolean(app.switchingId));
</script>

<div
  class="grid grid-cols-[repeat(auto-fill,minmax(168px,1fr))] auto-rows-fr items-start gap-2.5 pb-2"
>
  {#each folders as folder (folder.id)}
    <FolderCard {folder} />
  {/each}

  {#if insideFolder}
    <button
      type="button"
      class={cn(
        "group relative flex h-full min-h-[146px] cursor-pointer flex-col items-center justify-center gap-1.5 overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--base-alt)] px-2.5 pt-2 pb-1 text-center text-inherit transition-[transform,border-color,background-color] duration-[120ms] ease-out hover:z-[2] hover:border-[var(--border-strong)] hover:bg-[var(--surface-hover)] [&_img]:pointer-events-none [&_img]:[-webkit-user-drag:none] [&_img]:select-none [&_svg]:pointer-events-none [&_svg]:select-none",
        locked && "opacity-[0.45] pointer-events-none",
      )}
      title={t.folder_leave()}
      onclick={() => (app.folderId = null)}
    >
      <span
        class="flex size-[62px] shrink-0 items-center justify-center rounded-full bg-[var(--avatar)] text-[var(--text)]"
        aria-hidden="true"><ArrowLeft size={24} /></span
      >
      <span class="w-full truncate text-center font-semibold text-[var(--text-strong)]"
        >{t.common_back()}</span
      >
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
