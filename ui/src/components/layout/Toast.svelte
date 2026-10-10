<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";

  import AccountCaption from "../accounts/AccountCaption.svelte";
  import FolderCaption from "../folders/FolderCaption.svelte";
  import { containsAccountName } from "../../lib/i18n/messages";
  import MessageText from "../ui/MessageText.svelte";
  import { app } from "../../lib/state/state.svelte";
  const showAccountName = $derived(
    app.toast.account && !containsAccountName(app.toast.parts, app.toast.account.display_name),
  );
  const showFolderName = $derived(
    app.toast.folder && !containsAccountName(app.toast.parts, app.toast.folder.name),
  );
</script>

{#if app.toast.visible}
  <div
    class="toast"
    class:ok={app.toast.ok}
    role="status"
    aria-live="polite"
    aria-label={app.toast.account
      ? t.account_notification({ account: app.toast.account.display_name, message: app.toast.text })
      : app.toast.folder
        ? t.folder_notification({ folder: app.toast.folder.name, message: app.toast.text })
        : undefined}
  >
    <span class="toast-message">
      {#if showAccountName && app.toast.account}<AccountCaption
          entry={app.toast.account}
          inline
          bold
        />{" · "}{/if}
      {#if showFolderName && app.toast.folder}<FolderCaption
          folder={app.toast.folder}
          size={22}
          inline
          bold
        />{" · "}{/if}
      <span class="toast-content"
        ><MessageText
          content={app.toast.parts}
          account={app.toast.account}
          folder={app.toast.folder}
        /></span
      >
    </span>
  </div>
{/if}
