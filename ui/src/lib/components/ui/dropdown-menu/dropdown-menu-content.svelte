<script lang="ts">
  import { DropdownMenu as DropdownMenuPrimitive } from "bits-ui";
  import { cn, type WithoutChild, type WithoutChildrenOrChild } from "$lib/utils.js";
  import type { ComponentProps } from "svelte";

  let {
    ref = $bindable(null),
    sideOffset = 4,
    portalProps,
    class: className,
    ...restProps
  }: WithoutChild<DropdownMenuPrimitive.ContentProps> & {
    portalProps?: WithoutChildrenOrChild<ComponentProps<typeof DropdownMenuPrimitive.Portal>>;
  } = $props();
</script>

<DropdownMenuPrimitive.Portal {...portalProps}>
  <DropdownMenuPrimitive.Content
    bind:ref
    data-slot="dropdown-menu-content"
    {sideOffset}
    class={cn(
      "bg-popover text-popover-foreground data-open:animate-in data-closed:animate-out data-closed:fade-out-0 data-open:fade-in-0 data-closed:zoom-out-95 data-open:zoom-in-95 ring-foreground/10 z-50 min-w-56 max-h-(--bits-dropdown-menu-content-available-height) overflow-x-hidden overflow-y-auto rounded-md p-1 text-sm ring-1 duration-100",
      className,
    )}
    {...restProps}
  />
</DropdownMenuPrimitive.Portal>
