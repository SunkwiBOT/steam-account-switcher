<script lang="ts">
  import { t, folderLabel } from "../../lib/i18n/i18n.svelte";

  import { openFolderMenu } from "../../lib/state/menus.svelte";
  import { app } from "../../lib/state/state.svelte";
  import SteamGuardIcon from "../steam-guard/SteamGuardIcon.svelte";
  import { folderIcon } from "../../lib/folders/folder-style";
  import type { FolderType } from "../../lib/types";

  type FolderCardPropsType = { folder: FolderType };

  let { folder }: FolderCardPropsType = $props();
  const Glyph = $derived(folderIcon(folder.icon));
  const iconStyle = $derived(
    folder.color ? `color:${folder.color};background:${folder.color}33` : "",
  );
</script>

<button
  type="button"
  class="card folder-card"
  title={folder.system ? t.guard_saved_folder_open() : t.folder_card_tooltip()}
  onclick={() => (app.folderId = folder.id)}
  oncontextmenu={(event) => openFolderMenu(event, folder.id)}
>
  <span class="folder-icon" style={folder.system ? "" : iconStyle} aria-hidden="true">
    {#if folder.system}
      <SteamGuardIcon size={32} />
    {:else}
      <Glyph size={32} />
    {/if}
  </span>
  <span class="card-name">{folderLabel(folder)}</span>
  <div class="card-meta">
    <span>{t.accounts_count({ count: folder.account_count })}</span>
  </div>
</button>
