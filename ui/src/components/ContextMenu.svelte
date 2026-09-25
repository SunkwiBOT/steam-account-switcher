<script lang="ts">
  import { CARD_COLORS } from "../lib/api";
  import { copySteamId, removeAccount, switchAccount } from "../lib/actions.svelte";
  import { askFolderName } from "../lib/dialogs.svelte";
  import { closeContextMenu, contextMenu } from "../lib/menus.svelte";
  import { accountById, app, folders, run } from "../lib/state.svelte";
  import { ArrowRight, Check, Copy, FolderPlus, SquarePen, Trash2, X } from "@lucide/svelte";

  let element = $state<HTMLDivElement | null>(null);
  let position = $state({ left: 0, top: 0 });

  const entry = $derived(contextMenu.steamId ? accountById(contextMenu.steamId) : null);
  const folder = $derived(folders().find((item) => item.id === contextMenu.folderId) ?? null);

  $effect(() => {
    if (!contextMenu.open || !element) {
      return;
    }
    const width = element.offsetWidth;
    const height = element.offsetHeight;
    position = {
      left: Math.max(8, Math.min(contextMenu.x, window.innerWidth - width - 8)),
      top: Math.max(8, Math.min(contextMenu.y, window.innerHeight - height - 8)),
    };
  });

  function switchTo(steamId: string): void {
    closeContextMenu();
    switchAccount(steamId);
  }

  function openDetails(steamId: string): void {
    closeContextMenu();
    app.openAccountId = steamId;
  }

  function copyId(steamId: string): void {
    closeContextMenu();
    copySteamId(steamId);
  }

  function moveTo(steamId: string, folderId: string | null): void {
    closeContextMenu();
    run("assign_folder", { steamId, folderId });
  }

  function setColor(steamId: string, color: string): void {
    closeContextMenu();
    run("set_account_color", { steamId, color: color || null });
  }

  function remove(steamId: string): void {
    closeContextMenu();
    removeAccount(steamId);
  }

  function openFolder(folderId: string): void {
    closeContextMenu();
    app.folderId = folderId;
  }

  async function createFolder(steamId: string): Promise<void> {
    const name = await askFolderName({ title: "New folder", confirmLabel: "Create" });
    if (!name) {
      return;
    }
    await run("create_folder", { name });
    const created = folders().find((item) => item.name === name.trim());
    if (created) {
      await run("assign_folder", { steamId, folderId: created.id });
    }
  }

  async function renameFolder(id: string, name: string): Promise<void> {
    const next = await askFolderName({
      title: "Rename folder",
      value: name,
      confirmLabel: "Rename",
    });
    if (next) {
      await run("rename_folder", { id, name: next });
    }
  }

  async function deleteFolder(id: string): Promise<void> {
    closeContextMenu();
    await run("delete_folder", { id }, { success: "Folder deleted." });
    if (app.folderId === id) {
      app.folderId = null;
    }
  }
</script>

{#if contextMenu.open}
  <div class="context-menu" bind:this={element} style="left:{position.left}px;top:{position.top}px">
    {#if entry}
      <button class="menu-item accent" type="button" onclick={() => switchTo(entry.steam_id)}>
        <span class="menu-icon"><ArrowRight size={16} /></span> Switch to this account
      </button>
      <button class="menu-item" type="button" onclick={() => copyId(entry.steam_id)}>
        <span class="menu-icon"><Copy size={16} /></span> Copy Steam ID 64
      </button>
      <hr />
      <p class="menu-heading">Move to folder</p>
      <button
        class="menu-item"
        class:checked={!entry.folder_id}
        type="button"
        onclick={() => moveTo(entry.steam_id, null)}
      >
        <span class="menu-check">
          {#if !entry.folder_id}<Check size={15} />{/if}
        </span>
        No folder
      </button>
      {#each folders() as item (item.id)}
        <button
          class="menu-item"
          class:checked={entry.folder_id === item.id}
          type="button"
          onclick={() => moveTo(entry.steam_id, item.id)}
        >
          <span class="menu-check">
            {#if entry.folder_id === item.id}<Check size={15} />{/if}
          </span>
          {item.name}
        </button>
      {/each}
      <button class="menu-item" type="button" onclick={() => createFolder(entry.steam_id)}>
        <span class="menu-icon"><FolderPlus size={16} /></span> New folder…
      </button>
      <hr />
      <p class="menu-heading">Card colour</p>
      <div class="menu-swatches">
        {#each CARD_COLORS as color (color)}
          <button
            class="color-swatch"
            class:none={!color}
            class:active={(entry.color ?? "") === color}
            type="button"
            style={color ? `background:${color}` : ""}
            title={color || "No colour"}
            onclick={() => setColor(entry.steam_id, color)}
          >
            {#if !color}<X size={14} />{/if}
          </button>
        {/each}
      </div>
      <hr />
      <button class="menu-item" type="button" onclick={() => openDetails(entry.steam_id)}>
        <span class="menu-icon"><SquarePen size={16} /></span> View / edit details…
      </button>
      <button class="menu-item danger" type="button" onclick={() => remove(entry.steam_id)}>
        <span class="menu-icon"><Trash2 size={16} /></span> Remove account
      </button>
    {:else if folder}
      <p class="menu-heading">{folder.name}</p>
      <button class="menu-item" type="button" onclick={() => openFolder(folder.id)}>
        <span class="menu-icon"><ArrowRight size={16} /></span> Open
      </button>
      <button class="menu-item" type="button" onclick={() => renameFolder(folder.id, folder.name)}>
        <span class="menu-icon"><SquarePen size={16} /></span> Rename…
      </button>
      <hr />
      <button class="menu-item danger" type="button" onclick={() => deleteFolder(folder.id)}>
        <span class="menu-icon"><Trash2 size={16} /></span> Delete folder
      </button>
    {/if}
  </div>
{/if}
