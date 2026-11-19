<script lang="ts">
  import { Switch as SwitchPrimitive } from "bits-ui";
  import { cn, type WithoutChildrenOrChild } from "$lib/utils.js";

  let {
    ref = $bindable(null),
    class: className,
    checked = $bindable(false),
    ...restProps
  }: WithoutChildrenOrChild<SwitchPrimitive.RootProps> = $props();
</script>

<SwitchPrimitive.Root
  bind:ref
  bind:checked
  data-slot="switch"
  class={cn(
    "peer group/switch relative inline-flex h-[18.4px] w-[32px] shrink-0 items-center rounded-full border border-transparent outline-hidden transition-all focus-visible:outline-solid focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring data-checked:bg-primary data-unchecked:bg-input data-disabled:cursor-not-allowed data-disabled:opacity-50",
    className,
  )}
  {...restProps}
>
  <SwitchPrimitive.Thumb
    data-slot="switch-thumb"
    class="pointer-events-none block size-4 rounded-full bg-background ring-0 transition-transform group-data-checked/switch:translate-x-[calc(100%_-_2px)] group-data-unchecked/switch:translate-x-0"
  />
</SwitchPrimitive.Root>
