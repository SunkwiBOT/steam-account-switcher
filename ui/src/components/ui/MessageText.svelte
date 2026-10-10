<script lang="ts">
  import { messageParts, type MessageType } from "../../lib/i18n/messages";
  import type { ToastAccountType, ToastFolderType } from "../../lib/types";
  import AccountCaption from "../accounts/AccountCaption.svelte";
  import FolderCaption from "../folders/FolderCaption.svelte";
  type MessageTextPropsType = {
    content: MessageType;
    account?: ToastAccountType | null;
    folder?: ToastFolderType | null;
  };

  let { content, account = null, folder = null }: MessageTextPropsType = $props();
  const parts = $derived(messageParts(content));
</script>

{#each parts as part}{#if part.code}<code class="inline-code">{part.text}</code
    >{:else if account && part.bold && part.text === account.display_name}<AccountCaption
      entry={account}
      inline
      bold
    />{:else if folder && part.bold && part.text === folder.name}<FolderCaption
      {folder}
      size={22}
      inline
      bold
    />{:else if part.bold}<strong>{part.text}</strong>{:else}{part.text}{/if}{/each}
