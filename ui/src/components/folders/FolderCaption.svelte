<script lang="ts">
  import { folderLabel } from "../../lib/i18n/i18n.svelte";
  import { folderIcon } from "../../lib/folders/folder-style";
  import SteamGuardIcon from "../steam-guard/SteamGuardIcon.svelte";
  import type { FolderType } from "../../lib/types";

  type FolderCaptionPropsType = {
    folder: Pick<FolderType, "name" | "color" | "icon" | "system">;
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
</script>

<span class="account-caption folder-caption" class:inline
  ><span class="folder-badge" style={badgeStyle} aria-hidden="true"
    >{#if folder.system}<SteamGuardIcon size={glyphSize} />{:else}<Glyph
        size={glyphSize}
      />{/if}</span
  ><span title={name}
    >{#if bold}<strong>{name}</strong>{:else}{name}{/if}</span
  ></span
>
