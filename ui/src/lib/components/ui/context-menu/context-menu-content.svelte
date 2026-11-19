<script lang="ts">
  import { ContextMenu as ContextMenuPrimitive } from "bits-ui";
  import { cn, type WithoutChild, type WithoutChildrenOrChild } from "$lib/utils.js";
  import type { ComponentProps } from "svelte";

  let {
    ref = $bindable(null),
    portalProps,
    class: className,
    ...restProps
  }: WithoutChild<ContextMenuPrimitive.ContentProps> & {
    portalProps?: WithoutChildrenOrChild<ComponentProps<typeof ContextMenuPrimitive.Portal>>;
  } = $props();
</script>

<ContextMenuPrimitive.Portal {...portalProps}>
  <ContextMenuPrimitive.Content
    bind:ref
    data-slot="context-menu-content"
    class={cn(
      "bg-popover text-popover-foreground data-open:animate-in data-closed:animate-out data-closed:fade-out-0 data-open:fade-in-0 data-closed:zoom-out-95 data-open:zoom-in-95 ring-foreground/10 z-[70] max-h-[70vh] min-w-60 overflow-x-hidden overflow-y-auto rounded-md p-1 text-sm ring-1 duration-100",
      className,
    )}
    {...restProps}
  />
</ContextMenuPrimitive.Portal>
