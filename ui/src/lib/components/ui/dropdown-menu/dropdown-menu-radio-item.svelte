<script lang="ts">
  import { DropdownMenu as DropdownMenuPrimitive } from "bits-ui";
  import CheckIcon from "@lucide/svelte/icons/check";
  import { cn, type WithoutChild } from "$lib/utils.js";

  let {
    ref = $bindable(null),
    class: className,
    children: childrenProp,
    ...restProps
  }: WithoutChild<DropdownMenuPrimitive.RadioItemProps> = $props();
</script>

<DropdownMenuPrimitive.RadioItem
  bind:ref
  data-slot="dropdown-menu-radio-item"
  class={cn(
    "data-highlighted:bg-accent data-highlighted:text-accent-foreground data-disabled:pointer-events-none data-disabled:opacity-50 relative flex cursor-pointer items-center gap-2 rounded-sm py-1.5 pr-2 pl-2 text-sm outline-hidden select-none [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
    className,
  )}
  {...restProps}
>
  {#snippet children({ checked })}
    {@render childrenProp?.({ checked })}
    <span class="ml-auto flex size-4 items-center justify-center">
      {#if checked}<CheckIcon class="text-primary size-4" />{/if}
    </span>
  {/snippet}
</DropdownMenuPrimitive.RadioItem>
