<script lang="ts">
  import { t, rich } from "../../lib/i18n/i18n.svelte";

  import ColorSwatches from "../folders/ColorSwatches.svelte";
  import { randomFolderStyle } from "../../lib/folders/folder-style";
  import { copySteamId } from "../../lib/actions/account-copy.svelte";
  import { removeAccount, switchAccount } from "../../lib/actions/account-lifecycle.svelte";
  import { askFolderName } from "../../lib/state/dialogs.svelte";
  import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuRadioGroup,
    ContextMenuRadioItem,
    ContextMenuSeparator,
    ContextMenuSub,
    ContextMenuSubContent,
    ContextMenuSubTrigger,
    ContextMenuTrigger,
  } from "$lib/components/ui/context-menu/index.js";
  import { app, run } from "../../lib/state/state.svelte";
  import { folders } from "../../lib/state/folders.svelte";
  import { ArrowRightLeft, Copy, FolderPlus, SquarePen, Trash2 } from "@lucide/svelte";
  import type { Snippet } from "svelte";
  import type { AccountDto } from "../../lib/bindings/AccountDto";

  import SteamGuardIcon from "../steam-guard/SteamGuardIcon.svelte";

  type AccountContextMenuPropsType = {
    entry: AccountDto;
    children: Snippet<[{ props: Record<string, unknown> }]>;
  };

  let { entry, children }: AccountContextMenuPropsType = $props();
  let open = $state(false);

  const folderName = $derived(
    folders().find((item) => item.id === entry.folder_id)?.name ?? t.folder_none(),
  );

  let suppressCloseFocus = false;

  // ! The content's own close-focus restoration would steal focus back from a dialog an action opens.
  const pick = <T,>(
    action: (id: string, value: T) => unknown,
    id: string,
    value?: T,
    opensDialog = false,
  ): void => {
    if (!id) return;
    suppressCloseFocus = opensDialog;
    void action(id, value as T);
  };

  const onCloseAutoFocus = (event: Event): void => {
    if (!suppressCloseFocus) return;
    event.preventDefault();
    suppressCloseFocus = false;
  };

  const pickSwatch = (action: (id: string, value: string) => unknown, color: string): void => {
    open = false;
    pick(action, entry.steam_id, color);
  };

  const openDetails = (steamId: string): void => {
    app.openAccountId = steamId;
  };

  const openSteamGuard = (steamId: string): void => {
    app.steamGuardAccountId = steamId;
    app.steamGuardOpen = true;
  };

  const moveTo = (steamId: string, folderId: string | null): void => {
    void run("assign_folder", { steamId, folderId });
  };

  const setColor = (steamId: string, color: string): void => {
    void run("set_account_color", { steamId, color: color || null });
  };

  const createFolder = async (steamId: string): Promise<void> => {
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
    const created = folders().find((item) => !known.has(item.id));
    if (created) {
      await run(
        "assign_folder",
        { steamId, folderId: created.id },
        {
          successTitle: t.toast_folder_moved_title(),
          success: rich(t.folder_move_success, { folder: created.name }),
        },
      );
    }
  };
</script>

<ContextMenu bind:open>
  <ContextMenuTrigger>
    {#snippet child({ props })}
      {@render children({ props })}
    {/snippet}
  </ContextMenuTrigger>
  <ContextMenuContent {onCloseAutoFocus}>
    {#if entry.is_local}
      <ContextMenuItem variant="accent" onSelect={() => pick(switchAccount, entry.steam_id)}>
        <ArrowRightLeft size={16} />
        {t.account_switch_action()}
      </ContextMenuItem>
    {/if}
    <ContextMenuItem onSelect={() => pick(copySteamId, entry.steam_id)}>
      <Copy size={16} />
      {t.steam_ids_copy_64()}
    </ContextMenuItem>
    <ContextMenuItem onSelect={() => pick(openSteamGuard, entry.steam_id, undefined, true)}>
      <SteamGuardIcon size={17} />
      {entry.has_authenticator ? t.guard_open_action() : t.guard_setup()}
    </ContextMenuItem>
    {#if entry.is_local}
      <ContextMenuSeparator />
      <ContextMenuSub>
        <ContextMenuSubTrigger>
          <FolderPlus size={15} />
          {t.folder_move_label()}
          <span class="ml-auto text-xs text-muted-foreground">{folderName}</span>
        </ContextMenuSubTrigger>
        <ContextMenuSubContent>
          <ContextMenuRadioGroup
            value={entry.folder_id ?? ""}
            onValueChange={(value) => pick(moveTo, entry.steam_id, value || null)}
          >
            <ContextMenuRadioItem value="">{t.folder_none()}</ContextMenuRadioItem>
            {#each folders().filter((item) => !item.system) as folder (folder.id)}
              <ContextMenuRadioItem value={folder.id}>{folder.name}</ContextMenuRadioItem>
            {/each}
          </ContextMenuRadioGroup>
        </ContextMenuSubContent>
      </ContextMenuSub>
      <ContextMenuItem onSelect={() => pick(createFolder, entry.steam_id, undefined, true)}>
        <FolderPlus size={16} />
        {t.folder_create_menu()}
      </ContextMenuItem>
    {/if}
    <ContextMenuSeparator />
    <p class="text-muted-foreground px-2 py-1.5 text-xs font-semibold tracking-wide uppercase">
      {t.account_colour()}
    </p>
    <ColorSwatches
      className="menu-swatches"
      value={entry.color}
      asMenuItem
      onselect={(color) => pickSwatch(setColor, color)}
    />
    <ContextMenuSeparator />
    <ContextMenuItem onSelect={() => pick(openDetails, entry.steam_id, undefined, true)}>
      <SquarePen size={16} />
      {t.account_details_menu()}
    </ContextMenuItem>
    <ContextMenuItem
      variant="destructive"
      onSelect={() => pick(removeAccount, entry.steam_id, undefined, true)}
    >
      <Trash2 size={16} />
      {entry.is_local ? t.account_remove_action() : t.guard_saved_delete()}
    </ContextMenuItem>
  </ContextMenuContent>
</ContextMenu>
