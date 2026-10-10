<script lang="ts">
  import { FOLDER_ICONS } from "../../lib/folders/folder-style";

  type FolderIconPickerPropsType = {
    value: string | null | undefined;
    color?: string | null;
    className?: string;
    onselect: (icon: string) => unknown;
  };

  let {
    value,
    color = null,
    className = "icon-grid",
    onselect,
  }: FolderIconPickerPropsType = $props();
</script>

<div class={className} style={`color:${color || "var(--muted)"}`}>
  {#each FOLDER_ICONS as entry (entry.id)}
    <button
      class="icon-choice"
      class:active={(value ?? "folder") === entry.id}
      type="button"
      title={entry.label()}
      aria-label={entry.label()}
      aria-pressed={(value ?? "folder") === entry.id}
      onclick={() => onselect(entry.id)}
    >
      <entry.icon size={18} />
    </button>
  {/each}
</div>
