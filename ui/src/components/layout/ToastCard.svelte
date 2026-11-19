<script lang="ts">
  import { CircleCheck, OctagonX, X } from "@lucide/svelte";

  import { t } from "../../lib/i18n/i18n.svelte";
  import { ToastKind } from "../../lib/state/state.constants";
  import type { ToastCardPropsType } from "../../lib/state/toasts.svelte";
  import MessageText from "../ui/MessageText.svelte";

  let { kind, title, description, account, folder, onclose }: ToastCardPropsType = $props();
</script>

<div
  class="relative flex w-toast max-w-[calc(100vw-2rem)] items-start gap-3 rounded-lg border bg-popover p-3 pr-10 text-sm text-popover-foreground shadow-lg"
  role={kind === ToastKind.ERROR ? "alert" : "status"}
>
  {#if kind === ToastKind.ERROR}
    <OctagonX class="mt-0.5 size-4 shrink-0 text-destructive-emphasis" />
  {:else}
    <CircleCheck class="mt-0.5 size-4 shrink-0 text-success-foreground" />
  {/if}
  <div class="flex min-w-0 flex-1 flex-col gap-1">
    <p class="m-0 font-semibold break-words">{title}</p>
    {#if description || account || folder}
      <p class="m-0 break-words"><MessageText content={description} {account} {folder} /></p>
    {/if}
  </div>
  <button
    class="icon-button absolute top-1.5 right-1.5"
    type="button"
    aria-label={t.common_close()}
    onclick={onclose}><X size={16} /></button
  >
</div>
