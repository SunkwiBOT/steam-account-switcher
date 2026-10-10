<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";

  import { X } from "@lucide/svelte";
  import { CARD_COLORS } from "../../lib/api/api";

  type ColorSwatchesPropsType = {
    value: string | null | undefined;
    className?: string;
    onselect: (color: string) => unknown;
  };

  let { value, className = "color-grid", onselect }: ColorSwatchesPropsType = $props();
</script>

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
