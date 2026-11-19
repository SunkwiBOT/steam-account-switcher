<script lang="ts">
  import { ContextMenu as ContextMenuPrimitive } from "bits-ui";
  import { cn } from "$lib/utils.js";

  let {
    ref = $bindable(null),
    class: className,
    ...restProps
  }: ContextMenuPrimitive.SubContentProps = $props();
</script>

<ContextMenuPrimitive.SubContent
  bind:ref
  data-slot="context-menu-sub-content"
  class={cn(
    "bg-popover text-popover-foreground data-open:animate-in data-closed:animate-out data-closed:fade-out-0 data-open:fade-in-0 data-closed:zoom-out-95 data-open:zoom-in-95 ring-foreground/10 z-[70] max-h-[50vh] min-w-48 overflow-x-hidden overflow-y-auto rounded-md p-1 text-sm ring-1 duration-100",
    className,
  )}
  {...restProps}
/>
