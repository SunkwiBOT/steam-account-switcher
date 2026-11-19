<script lang="ts">
  import { DropdownMenu as DropdownMenuPrimitive } from "bits-ui";
  import { cn, type WithoutChild } from "$lib/utils.js";

  let {
    ref = $bindable(null),
    class: className,
    variant = "default",
    ...restProps
  }: WithoutChild<DropdownMenuPrimitive.ItemProps> & {
    variant?: "default" | "destructive" | "accent";
  } = $props();
</script>

<DropdownMenuPrimitive.Item
  bind:ref
  data-slot="dropdown-menu-item"
  data-variant={variant}
  class={cn(
    "data-highlighted:bg-accent data-highlighted:text-accent-foreground data-[variant=destructive]:text-destructive-emphasis data-[variant=accent]:text-success-emphasis data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg:not([class*='text-'])]:text-muted-foreground relative flex cursor-pointer items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-hidden select-none [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
    className,
  )}
  {...restProps}
/>
