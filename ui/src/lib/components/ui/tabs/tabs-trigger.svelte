<script lang="ts">
  import { Tabs as TabsPrimitive } from "bits-ui";
  import { cn } from "$lib/utils.js";

  let {
    ref = $bindable(null),
    class: className,
    ...restProps
  }: TabsPrimitive.TriggerProps = $props();
</script>

<TabsPrimitive.Trigger
  bind:ref
  data-slot="tabs-trigger"
  class={cn(
    "flex items-center gap-2 rounded-sm border border-transparent px-2.5 py-2 text-sm text-muted-foreground text-left transition-colors outline-hidden focus-visible:outline-solid focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-ring not-data-active:hover:bg-accent/50 not-data-active:hover:text-foreground data-active:border-primary data-active:bg-accent data-active:font-semibold data-active:text-foreground group-data-vertical/tabs:w-full disabled:cursor-not-allowed disabled:opacity-50",
    className,
  )}
  {...restProps}
/>
