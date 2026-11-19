<script lang="ts">
  import { folderLabel } from "../../lib/i18n/i18n.svelte";
  import { folderIcon } from "../../lib/folders/folder-style";
  import SteamGuardIcon from "../steam-guard/SteamGuardIcon.svelte";
  import type { FolderDto } from "../../lib/bindings/FolderDto";

  type FolderCaptionPropsType = {
    folder: Pick<FolderDto, "name" | "color" | "icon" | "system">;
    size?: number;
    inline?: boolean;
    bold?: boolean;
  };

  let { folder, size = 28, inline = false, bold = false }: FolderCaptionPropsType = $props();
  const Glyph = $derived(folderIcon(folder.icon));
  const name = $derived(folderLabel(folder));
  const glyphSize = $derived(Math.round(size * 0.6));
  const badgeStyle = $derived(
    folder.color && !folder.system
      ? `--folder-size:${size}px;color:${folder.color};background:${folder.color}33`
      : `--folder-size:${size}px`,
  );
  const rootClasses = $derived(
    inline
      ? "inline max-w-full align-baseline [font-size:inherit] [color:inherit]"
      : "flex items-center gap-1 mt-[9px] text-[13px] text-[var(--muted)]",
  );
  const badgeClasses = $derived(
    inline
      ? "inline-block mr-1 size-[var(--folder-size)] shrink-0 rounded-full bg-[var(--avatar)] text-center align-[-0.16em] text-[var(--accent)] [line-height:var(--folder-size)]"
      : "inline-flex items-center justify-center size-[var(--folder-size)] shrink-0 rounded-full bg-[var(--avatar)] text-center text-[var(--accent)] [line-height:var(--folder-size)]",
  );
</script>

<span class={rootClasses}
  ><span class={badgeClasses} style={badgeStyle} aria-hidden="true"
    >{#if folder.system}<SteamGuardIcon size={glyphSize} />{:else}<Glyph
        size={glyphSize}
      />{/if}</span
  ><span title={name}
    >{#if bold}<strong>{name}</strong>{:else}{name}{/if}</span
  ></span
>
