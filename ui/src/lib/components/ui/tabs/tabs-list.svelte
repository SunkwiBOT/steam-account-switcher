<script lang="ts">
  import { Tabs as TabsPrimitive } from "bits-ui";
  import { cn } from "$lib/utils.js";

  let { ref = $bindable(null), class: className, ...restProps }: TabsPrimitive.ListProps = $props();
</script>

<TabsPrimitive.List
  bind:ref
  data-slot="tabs-list"
  class={cn(
    "flex flex-col gap-0.5 rounded-sm border border-border bg-background p-1 group-data-horizontal/tabs:h-8 group-data-horizontal/tabs:flex-row group-data-horizontal/tabs:items-center overflow-auto",
    className,
  )}
  {...restProps}
/>
