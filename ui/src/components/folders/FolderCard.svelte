<script lang="ts">
  import { t, folderLabel } from "../../lib/i18n/i18n.svelte";

  import { cn } from "$lib/utils";
  import FolderContextMenu from "./FolderContextMenu.svelte";
  import { app } from "../../lib/state/state.svelte";
  import SteamGuardIcon from "../steam-guard/SteamGuardIcon.svelte";
  import { Lock } from "@lucide/svelte";
  import { folderIcon } from "../../lib/folders/folder-style";
  import type { FolderDto } from "../../lib/bindings/FolderDto";

  type FolderCardPropsType = { folder: FolderDto };

  let { folder }: FolderCardPropsType = $props();
  const Glyph = $derived(folderIcon(folder.icon));
  const iconStyle = $derived(
    folder.color ? `color:${folder.color};background:${folder.color}33` : "",
  );
  const locked = $derived(Boolean(app.switchingId));
</script>

<FolderContextMenu {folder}>
  {#snippet children({ props })}
    <button
      {...props}
      type="button"
      class={cn(
        "group relative flex h-full min-h-[146px] cursor-pointer flex-col items-center gap-1.5 overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--base-alt)] px-2.5 pt-2 pb-1 text-center text-inherit transition-[transform,border-color,background-color] duration-[120ms] ease-out hover:z-[2] hover:border-[var(--border-strong)] hover:bg-[var(--surface-hover)] [&_img]:pointer-events-none [&_img]:[-webkit-user-drag:none] [&_img]:select-none [&_svg]:pointer-events-none [&_svg]:select-none",
        locked && "opacity-[0.45] pointer-events-none",
      )}
      title={folder.system ? t.guard_saved_folder_open() : t.folder_card_tooltip()}
      onclick={() => (app.folderId = folder.id)}
    >
      {#if folder.system}
        <span
          class="absolute top-2 right-2 inline-flex size-[26px] items-center justify-center rounded-full border border-[var(--border-strong)] bg-[var(--panel)] text-[var(--muted)]"
          title={t.folder_locked_tooltip()}><Lock size={14} /></span
        >
      {/if}
      <span
        class="flex size-[62px] shrink-0 items-center justify-center rounded-full bg-[var(--avatar)] text-[var(--accent)]"
        style={folder.system ? "" : iconStyle}
        aria-hidden="true"
      >
        {#if folder.system}
          <SteamGuardIcon size={32} />
        {:else}
          <Glyph size={32} />
        {/if}
      </span>
      <span
        class="w-full overflow-hidden text-center font-semibold text-clip whitespace-normal text-[var(--text-strong)] [overflow-wrap:anywhere]"
        >{folderLabel(folder)}</span
      >
      <div
        class="mt-auto flex h-[30px] w-full items-center justify-center gap-1.5 text-[11px] text-[var(--muted)]"
      >
        <span>{t.accounts_count({ count: folder.account_count })}</span>
      </div>
    </button>
  {/snippet}
</FolderContextMenu>
