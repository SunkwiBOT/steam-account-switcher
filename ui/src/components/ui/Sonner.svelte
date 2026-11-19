<script lang="ts">
  import { Toaster as Sonner, type ToasterProps as SonnerProps } from "svelte-sonner";

  let { ...restProps }: SonnerProps = $props();
</script>

<Sonner
  theme="dark"
  class="toaster group"
  style="--normal-bg: var(--color-popover); --normal-text: var(--color-popover-foreground); --normal-border: var(--color-border); font-family: var(--font-sans);"
  toastOptions={{
    classes: {
      toast: "cn-toast",
    },
  }}
  {...restProps}
/>
