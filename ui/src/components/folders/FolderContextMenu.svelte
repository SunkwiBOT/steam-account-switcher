<script lang="ts">
  import { t, rich } from "../../lib/i18n/i18n.svelte";

  import ColorSwatches from "./ColorSwatches.svelte";
  import FolderCaption from "./FolderCaption.svelte";
  import FolderIconPicker from "./FolderIconPicker.svelte";
  import { askConfirm, openFolderEditor } from "../../lib/state/dialogs.svelte";
  import { ConfirmKind } from "../../lib/state/dialogs.constants";
  import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuSeparator,
    ContextMenuTrigger,
  } from "$lib/components/ui/context-menu/index.js";
  import { app, run, showToast } from "../../lib/state/state.svelte";
  import { folders } from "../../lib/state/folders.svelte";
  import { ToastKind } from "../../lib/state/state.constants";
  import { ArrowRight, SquarePen, Trash2 } from "@lucide/svelte";
  import type { Snippet } from "svelte";
  import type { FolderDto } from "../../lib/bindings/FolderDto";

  type FolderContextMenuPropsType = {
    folder: FolderDto;
    children: Snippet<[{ props: Record<string, unknown> }]>;
  };

  let { folder, children }: FolderContextMenuPropsType = $props();
  let open = $state(false);

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

  const pickSwatch = <T,>(action: (id: string, value: T) => unknown, value: T): void => {
    open = false;
    pick(action, folder.id, value);
  };

  const openFolder = (id: string): void => {
    app.folderId = id;
  };

  const setFolderColor = (id: string, color: string): void => {
    void run("set_folder_color", { id, color: color || null });
  };

  const setFolderIcon = (id: string, icon: string): void => {
    void run("set_folder_icon", { id, icon });
  };

  const deleteFolder = (id: string): void => {
    const name = folders().find((item) => item.id === id)?.name ?? "";
    askConfirm({
      title: t.folder_delete_title(),
      text: rich(t.folder_delete_confirm, { folder: name }),
      confirmLabel: t.common_delete(),
      busyLabel: t.common_removing(),
      danger: true,
      kind: ConfirmKind.REMOVE,
      action: async () => {
        const result = await run("delete_folder", { id });
        if (result) {
          showToast(ToastKind.SUCCESS, t.toast_folder_deleted_title(), {
            description: rich(t.folder_delete_success, { folder: name }),
            folder,
          });
          if (app.folderId === id) {
            app.folderId = null;
          }
        }
        return result !== null;
      },
    });
  };
</script>

<ContextMenu bind:open>
  <ContextMenuTrigger>
    {#snippet child({ props })}
      {@render children({ props })}
    {/snippet}
  </ContextMenuTrigger>
  <ContextMenuContent {onCloseAutoFocus}>
    <p class="text-muted-foreground px-2 py-1.5 text-sm font-semibold">
      <FolderCaption {folder} size={18} inline bold />
    </p>
    <ContextMenuItem onSelect={() => pick(openFolder, folder.id)}>
      <ArrowRight size={16} />
      {t.common_open()}
    </ContextMenuItem>
    {#if !folder.system}
      <ContextMenuItem onSelect={() => pick(openFolderEditor, folder.id, undefined, true)}>
        <SquarePen size={16} />
        {t.folder_edit_menu()}
      </ContextMenuItem>
      <ContextMenuSeparator />
      <p class="text-muted-foreground px-2 py-1.5 text-xs font-semibold tracking-wide uppercase">
        {t.folder_colour()}
      </p>
      <ColorSwatches
        className="menu-swatches"
        value={folder.color}
        asMenuItem
        groupLabel={t.folder_colour()}
        onselect={(color) => pickSwatch(setFolderColor, color)}
      />
      <p class="text-muted-foreground px-2 py-1.5 text-xs font-semibold tracking-wide uppercase">
        {t.folder_icon()}
      </p>
      <FolderIconPicker
        className="menu-icons"
        value={folder.icon}
        color={folder.color}
        asMenuItem
        groupLabel={t.folder_icon()}
        onselect={(icon) => pickSwatch(setFolderIcon, icon)}
      />
      <ContextMenuSeparator />
      <ContextMenuItem
        variant="destructive"
        onSelect={() => pick(deleteFolder, folder.id, undefined, true)}
      >
        <Trash2 size={16} />
        {t.folder_delete_action()}
      </ContextMenuItem>
    {/if}
  </ContextMenuContent>
</ContextMenu>
