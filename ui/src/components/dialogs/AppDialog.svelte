<script lang="ts">
  import { Dialog } from "$lib/components/ui/dialog/index.js";
  import type { Snippet } from "svelte";

  // ! Mounted only while open, since bits-ui's Portal appends to `body` at mount time, not at `open`.
  type AppDialogPropsType = { open: boolean; onclose: () => void; children: Snippet };

  let { open, onclose, children }: AppDialogPropsType = $props();
</script>

{#if open}
  <Dialog
    bind:open={
      () => open,
      (next) => {
        if (!next) onclose();
      }
    }
  >
    {@render children()}
  </Dialog>
{/if}
