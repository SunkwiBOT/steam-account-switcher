<script lang="ts">
  import { t, rich } from "../../lib/i18n/i18n.svelte";

  import ColorSwatches from "../folders/ColorSwatches.svelte";
  import FolderIconPicker from "../folders/FolderIconPicker.svelte";
  import FolderCaption from "../folders/FolderCaption.svelte";
  import { randomFolderStyle } from "../../lib/folders/folder-style";
  import { nextMenuIndex } from "../../lib/state/keyboard";
  import { copySteamId, removeAccount, switchAccount } from "../../lib/actions/actions.svelte";
  import { askConfirm, askFolderName, openFolderEditor } from "../../lib/state/dialogs.svelte";
  import { ConfirmKind } from "../../lib/state/dialogs.constants";
  import {
    closeContextMenu,
    contextMenu,
    dropdown,
    openDropdown,
  } from "../../lib/state/menus.svelte";
  import { accountById, app, folders, run, showToast } from "../../lib/state/state.svelte";
  import {
    ArrowRight,
    ArrowRightLeft,
    ChevronDown,
    Copy,
    FolderPlus,
    SquarePen,
    Trash2,
  } from "@lucide/svelte";

  import SteamGuardIcon from "../steam-guard/SteamGuardIcon.svelte";

  let element = $state<HTMLDivElement | null>(null);
  let position = $state({ left: 0, top: 0 });
  let folderButton = $state<HTMLButtonElement | null>(null);

  const entry = $derived(contextMenu.steamId ? accountById(contextMenu.steamId) : null);
  const folder = $derived(folders().find((item) => item.id === contextMenu.folderId) ?? null);
  const folderName = $derived(
    folders().find((item) => item.id === entry?.folder_id)?.name ?? t.folder_none(),
  );

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

  $effect(() => {
    if (!contextMenu.open || !element) {
      return;
    }
    element.querySelector<HTMLElement>("[role='menuitem']")?.focus();
  });

  const moveFocus = (event: KeyboardEvent): void => {
    if (!element) return;
    const items = [...element.querySelectorAll<HTMLElement>("[role='menuitem']")];
    const next = nextMenuIndex(
      items.findIndex((item) => item === document.activeElement),
      items.length,
      event.key,
    );
    if (next === null) return;
    event.preventDefault();
    items[next]?.focus();
  };

  const switchTo = (steamId: string): void => {
    closeContextMenu();
    switchAccount(steamId);
  };

  const openDetails = (steamId: string): void => {
    closeContextMenu();
    app.openAccountId = steamId;
  };

  const copyId = (steamId: string): void => {
    closeContextMenu();
    copySteamId(steamId);
  };

  const openSteamGuard = (steamId: string): void => {
    closeContextMenu();
    app.steamGuardAccountId = steamId;
    app.steamGuardOpen = true;
  };

  const moveTo = (steamId: string, folderId: string | null): void => {
    closeContextMenu();
    run("assign_folder", { steamId, folderId });
  };

  const pickFolder = (): void => {
    if (!entry || !folderButton) return;
    const account = entry;
    openDropdown({
      anchor: folderButton,
      value: account.folder_id ?? "",
      options: [
        { value: "", label: t.folder_none() },
        ...folders()
          .filter((item) => !item.system)
          .map((item) => ({ value: item.id, label: item.name })),
      ],
      onSelect: (value) => moveTo(account.steam_id, value || null),
    });
  };

  const setColor = (steamId: string, color: string): void => {
    closeContextMenu();
    run("set_account_color", { steamId, color: color || null });
  };

  const setFolderColor = (id: string, color: string): void => {
    closeContextMenu();
    run("set_folder_color", { id, color: color || null });
  };

  const setFolderIcon = (id: string, icon: string): void => {
    closeContextMenu();
    run("set_folder_icon", { id, icon });
  };

  const remove = (steamId: string): void => {
    closeContextMenu();
    removeAccount(steamId);
  };

  const openFolder = (folderId: string): void => {
    closeContextMenu();
    app.folderId = folderId;
  };

  const createFolder = async (steamId: string): Promise<void> => {
    closeContextMenu();
    const name = await askFolderName({
      title: t.folder_create_title(),
      confirmLabel: t.common_create(),
    });
    if (!name) {
      return;
    }
    const style = randomFolderStyle();
    const known = new Set(folders().map((item) => item.id));
    const result = await run("create_folder", { name, color: style.color, icon: style.icon });
    if (!result) return;
    // The id of the new folder, never the first folder that shares its name.
    const created = folders().find((item) => !known.has(item.id));
    if (created) {
      await run(
        "assign_folder",
        { steamId, folderId: created.id },
        {
          success: rich(t.folder_move_success, { folder: created.name }),
        },
      );
    }
  };

  const deleteFolder = async (id: string): Promise<void> => {
    const folder = folders().find((item) => item.id === id);
    const name = folder?.name ?? "";
    closeContextMenu();
    const answer = await askConfirm({
      title: t.folder_delete_title(),
      text: rich(t.folder_delete_confirm, { folder: name }),
      confirmLabel: t.common_delete(),
      danger: true,
      kind: ConfirmKind.REMOVE,
    });
    if (!answer) return;
    const result = await run("delete_folder", { id });
    if (result && folder)
      showToast(rich(t.folder_delete_success, { folder: name }), true, null, folder);
    if (result && app.folderId === id) {
      app.folderId = null;
    }
  };
</script>

{#if contextMenu.open}
  <div
    class="context-menu"
    role="menu"
    tabindex="-1"
    bind:this={element}
    style="left:{position.left}px;top:{position.top}px"
    onkeydown={moveFocus}
  >
    {#if entry}
      {#if entry.is_local}
        <button
          role="menuitem"
          class="menu-item accent"
          type="button"
          onclick={() => switchTo(entry.steam_id)}
        >
          <span class="menu-icon"><ArrowRightLeft size={16} /></span>
          {t.account_switch_action()}
        </button>
      {/if}
      <button
        role="menuitem"
        class="menu-item"
        type="button"
        onclick={() => copyId(entry.steam_id)}
      >
        <span class="menu-icon"><Copy size={16} /></span>
        {t.steam_ids_copy_64()}
      </button>
      <button
        role="menuitem"
        class="menu-item"
        type="button"
        onclick={() => openSteamGuard(entry.steam_id)}
      >
        <span class="menu-icon"><SteamGuardIcon size={17} /></span>
        {entry.has_authenticator ? t.guard_open_action() : t.guard_setup()}
      </button>
      {#if entry.is_local}
        <hr />
        <p class="menu-heading">{t.folder_move_label()}</p>
        <div class="menu-folder-picker">
          <button
            class="select-button"
            type="button"
            aria-label={t.folder_move_label()}
            aria-haspopup="listbox"
            aria-expanded={dropdown.open && dropdown.anchor === folderButton}
            bind:this={folderButton}
            onclick={pickFolder}
          >
            <span>{folderName}</span>
            <span class="picker-caret"><ChevronDown size={14} /></span>
          </button>
        </div>
        <button
          role="menuitem"
          class="menu-item"
          type="button"
          onclick={() => createFolder(entry.steam_id)}
        >
          <span class="menu-icon"><FolderPlus size={16} /></span>
          {t.folder_create_menu()}
        </button>
      {/if}
      <hr />
      <p class="menu-heading">{t.account_colour()}</p>
      <ColorSwatches
        className="menu-swatches"
        value={entry.color}
        onselect={(color) => setColor(entry.steam_id, color)}
      />
      <hr />
      <button
        role="menuitem"
        class="menu-item"
        type="button"
        onclick={() => openDetails(entry.steam_id)}
      >
        <span class="menu-icon"><SquarePen size={16} /></span>
        {t.account_details_menu()}
      </button>
      <button
        role="menuitem"
        class="menu-item danger"
        type="button"
        onclick={() => remove(entry.steam_id)}
      >
        <span class="menu-icon"><Trash2 size={16} /></span>
        {entry.is_local ? t.account_remove_action() : t.guard_saved_delete()}
      </button>
    {:else if folder}
      <p class="menu-heading folder-heading">
        <FolderCaption {folder} size={18} inline bold />
      </p>
      <button role="menuitem" class="menu-item" type="button" onclick={() => openFolder(folder.id)}>
        <span class="menu-icon"><ArrowRight size={16} /></span>
        {t.common_open()}
      </button>
      {#if !folder.system}
        <button
          role="menuitem"
          class="menu-item"
          type="button"
          onclick={() => {
            const id = folder.id;
            closeContextMenu();
            openFolderEditor(id);
          }}
        >
          <span class="menu-icon"><SquarePen size={16} /></span>
          {t.folder_edit_menu()}
        </button>
        <hr />
        <p class="menu-heading">{t.folder_colour()}</p>
        <ColorSwatches
          className="menu-swatches"
          value={folder.color}
          onselect={(color) => setFolderColor(folder.id, color)}
        />
        <p class="menu-heading">{t.folder_icon()}</p>
        <FolderIconPicker
          className="menu-icons"
          value={folder.icon}
          color={folder.color}
          onselect={(icon) => setFolderIcon(folder.id, icon)}
        />
        <hr />
        <button
          role="menuitem"
          class="menu-item danger"
          type="button"
          onclick={() => deleteFolder(folder.id)}
        >
          <span class="menu-icon"><Trash2 size={16} /></span>
          {t.folder_delete_action()}
        </button>
      {/if}
    {/if}
  </div>
{/if}
