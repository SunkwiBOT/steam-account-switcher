<script lang="ts">
  import { Checkbox as CheckboxPrimitive } from "bits-ui";
  import { Check } from "@lucide/svelte";
  import { cn, type WithoutChildrenOrChild } from "$lib/utils.js";

  let {
    ref = $bindable(null),
    checked = $bindable(false),
    class: className,
    ...restProps
  }: WithoutChildrenOrChild<CheckboxPrimitive.RootProps> = $props();
</script>

<CheckboxPrimitive.Root
  bind:ref
  bind:checked
  data-slot="checkbox"
  class={cn(
    "peer relative flex size-4 shrink-0 items-center justify-center rounded-[4px] border border-input outline-hidden transition-colors focus-visible:outline-solid focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring data-checked:border-primary data-checked:bg-primary data-checked:text-primary-foreground data-disabled:cursor-not-allowed data-disabled:opacity-50",
    className,
  )}
  {...restProps}
>
  {#snippet children({ checked })}
    <div
      data-slot="checkbox-indicator"
      class="grid place-content-center text-current [&>svg]:size-3.5"
    >
      {#if checked}
        <Check />
      {/if}
    </div>
  {/snippet}
</CheckboxPrimitive.Root>
