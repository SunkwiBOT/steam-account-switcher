<script lang="ts">
  import { X } from "@lucide/svelte";
  import type { Component } from "svelte";
  import { t } from "../../lib/i18n/i18n.svelte";
  import type { MessageType } from "../../lib/i18n/messages";
  import AsyncIcon from "../ui/AsyncIcon.svelte";
  import MessageText from "../ui/MessageText.svelte";

  type DialogHeaderPropsType = {
    id: string;
    title: MessageType;
    icon: Component<{ size?: number }>;
    busy?: boolean;
    onclose?: () => void;
    closeDisabled?: boolean;
    closeLabel?: string;
  };

  let {
    id,
    title,
    icon: Icon,
    busy = false,
    onclose,
    closeDisabled = false,
    closeLabel,
  }: DialogHeaderPropsType = $props();
</script>

<header class="dialog-head">
  <h2 {id} class="dialog-title">
    {#if busy}<AsyncIcon size={18} />{:else}<span class="action-icon" aria-hidden="true"
        ><Icon size={18} /></span
      >{/if}
    <span class="dialog-title-text"><MessageText content={title} /></span>
  </h2>
  {#if onclose}
    <button
      class="icon-button"
      type="button"
      aria-label={closeLabel ?? t.common_close()}
      disabled={closeDisabled}
      onclick={onclose}><X size={16} /></button
    >
  {/if}
</header>
