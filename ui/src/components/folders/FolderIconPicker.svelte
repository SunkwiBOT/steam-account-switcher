<script lang="ts">
  import { FOLDER_ICONS } from "../../lib/folders/folder-style";
  import { t } from "../../lib/i18n/i18n.svelte";
  import { ContextMenu as ContextMenuPrimitive } from "bits-ui";

  type FolderIconPickerPropsType = {
    value: string | null | undefined;
    color?: string | null;
    className?: string;
    asMenuItem?: boolean;
    groupLabel?: string;
    onselect: (icon: string) => unknown;
  };

  let {
    value,
    color = null,
    className = "icon-grid",
    asMenuItem = false,
    groupLabel,
    onselect,
  }: FolderIconPickerPropsType = $props();
</script>

{#if asMenuItem}
  <ContextMenuPrimitive.RadioGroup
    class={className}
    style={`color:${color || "var(--muted)"}`}
    aria-label={groupLabel ?? t.folder_icon()}
    value={value ?? "folder"}
    onValueChange={onselect}
  >
    {#each FOLDER_ICONS as entry (entry.id)}
      <ContextMenuPrimitive.RadioItem value={entry.id} closeOnSelect={false}>
        {#snippet child({ props, checked })}
          <button
            {...props}
            class="icon-choice icon-menu-item"
            class:active={checked}
            type="button"
            title={entry.label()}
            aria-label={entry.label()}
          >
            <entry.icon size={18} />
          </button>
        {/snippet}
      </ContextMenuPrimitive.RadioItem>
    {/each}
  </ContextMenuPrimitive.RadioGroup>
{:else}
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
{/if}
