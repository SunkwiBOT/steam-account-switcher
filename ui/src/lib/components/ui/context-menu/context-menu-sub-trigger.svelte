<script lang="ts">
  import { ContextMenu as ContextMenuPrimitive } from "bits-ui";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import { cn, type WithoutChild } from "$lib/utils.js";

  let {
    ref = $bindable(null),
    class: className,
    children: childrenProp,
    ...restProps
  }: WithoutChild<ContextMenuPrimitive.SubTriggerProps> = $props();
</script>

<ContextMenuPrimitive.SubTrigger
  bind:ref
  data-slot="context-menu-sub-trigger"
  class={cn(
    "data-highlighted:bg-accent data-highlighted:text-accent-foreground data-[state=open]:bg-accent data-[state=open]:text-accent-foreground flex cursor-pointer items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-hidden select-none [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
    className,
  )}
  {...restProps}
>
  {@render childrenProp?.()}
  <ChevronRightIcon class="ml-auto" />
</ContextMenuPrimitive.SubTrigger>
