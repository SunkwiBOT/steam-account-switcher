<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";

  import { X } from "@lucide/svelte";
  import { CARD_COLORS } from "../../lib/api/api";
  import { ContextMenu as ContextMenuPrimitive } from "bits-ui";

  type ColorSwatchesPropsType = {
    value: string | null | undefined;
    className?: string;
    asMenuItem?: boolean;
    groupLabel?: string;
    onselect: (color: string) => unknown;
  };

  let {
    value,
    className = "color-grid",
    asMenuItem = false,
    groupLabel,
    onselect,
  }: ColorSwatchesPropsType = $props();
</script>

{#if asMenuItem}
  <ContextMenuPrimitive.RadioGroup
    class={className}
    aria-label={groupLabel ?? t.account_colour()}
    value={value ?? ""}
    onValueChange={onselect}
  >
    {#each CARD_COLORS as color (color)}
      <ContextMenuPrimitive.RadioItem value={color} closeOnSelect={false}>
        {#snippet child({ props, checked })}
          <button
            {...props}
            class="color-swatch swatch-menu-item"
            class:none={!color}
            class:active={checked}
            type="button"
            title={color || t.colour_none()}
            aria-label={color || t.colour_none()}
            style={color ? `background:${color}` : ""}
          >
            {#if !color}<X size={14} />{/if}
          </button>
        {/snippet}
      </ContextMenuPrimitive.RadioItem>
    {/each}
  </ContextMenuPrimitive.RadioGroup>
{:else}
  <div class={className}>
    {#each CARD_COLORS as color (color)}
      <button
        class="color-swatch"
        class:none={!color}
        class:active={(value ?? "") === color}
        type="button"
        title={color || t.colour_none()}
        aria-label={color || t.colour_none()}
        aria-pressed={(value ?? "") === color}
        style={color ? `background:${color}` : ""}
        onclick={() => onselect(color)}
      >
        {#if !color}<X size={14} />{/if}
      </button>
    {/each}
  </div>
{/if}
