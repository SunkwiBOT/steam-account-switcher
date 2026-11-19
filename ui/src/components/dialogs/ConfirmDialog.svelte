<script lang="ts">
  import DialogHeader from "./DialogHeader.svelte";
  import { t, rich } from "../../lib/i18n/i18n.svelte";

  import { ConfirmKind } from "../../lib/state/dialogs.constants";
  import { cancelConfirm, confirmDialog, runConfirmAction } from "../../lib/state/dialogs.svelte";
  import { accountById } from "../../lib/state/state.svelte";

  import MessageText from "../ui/MessageText.svelte";
  import FormError from "../ui/FormError.svelte";
  import { containsAccountName } from "../../lib/i18n/messages";
  const entry = $derived(accountById(confirmDialog.steamId ?? ""));
  const inlineAccount = $derived(
    entry && containsAccountName(confirmDialog.content, entry.display_name),
  );

  import { AlertTriangle, ArrowRightLeft, Check, Play, Trash2, UserPlus, X } from "@lucide/svelte";
  import AppDialog from "./AppDialog.svelte";
  import { DialogContent } from "$lib/components/ui/dialog/index.js";
  import { Checkbox } from "$lib/components/ui/checkbox/index.js";

  const ActionIcon = $derived(
    {
      [ConfirmKind.SWITCH]: ArrowRightLeft,
      [ConfirmKind.RUNNING_GAME]: AlertTriangle,
      [ConfirmKind.START]: Play,
      [ConfirmKind.REMOVE]: Trash2,
      [ConfirmKind.ADD]: UserPlus,
      [ConfirmKind.CONFIRM]: confirmDialog.danger ? AlertTriangle : Check,
    }[confirmDialog.kind],
  );

  let checked = $state(false);

  $effect(() => {
    if (confirmDialog.open) {
      checked = false;
    }
  });
  const uid = $props.id();
</script>

<!-- ! Mounted only while open: bits-ui's Portal appends to `body` at mount time, so an always-mounted Dialog would paint under the last opened one. -->
<AppDialog
  open={confirmDialog.open}
  onclose={() => {
    if (!confirmDialog.busy) cancelConfirm();
  }}
>
  <DialogContent showCloseButton={false} class="sm:max-w-md" aria-labelledby={`${uid}-title`}>
    <DialogHeader id={`${uid}-title`} title={confirmDialog.title} icon={ActionIcon} />
    {#if entry && !inlineAccount}<p class="dialog-text">
        <MessageText
          content={rich(t.account_details_description, { account: entry.display_name })}
          account={entry}
        />
      </p>{/if}
    <p class="dialog-text"><MessageText content={confirmDialog.content} account={entry} /></p>
    {#if confirmDialog.checkboxLabel}
      <label class="checkbox">
        <Checkbox bind:checked disabled={confirmDialog.busy} />
        <span>{confirmDialog.checkboxLabel}</span>
      </label>
    {/if}
    <FormError id={`${uid}-error`} message={confirmDialog.error} />
    <footer class="dialog-foot">
      <!-- svelte-ignore a11y_autofocus -->
      <button
        class="button"
        type="button"
        autofocus
        disabled={confirmDialog.busy}
        onclick={cancelConfirm}><X size={15} /> {t.common_cancel()}</button
      >
      <button
        class="button"
        class:primary={!confirmDialog.danger}
        class:danger={confirmDialog.danger}
        type="button"
        aria-busy={confirmDialog.busy}
        disabled={confirmDialog.busy || (confirmDialog.checkboxRequired && !checked)}
        onclick={() => void runConfirmAction({ cleanup: checked })}
      >
        {#if confirmDialog.busy}
          {confirmDialog.busyLabel}
        {:else}
          <ActionIcon size={15} />
          {confirmDialog.confirmLabel}
        {/if}
      </button>
    </footer>
  </DialogContent>
</AppDialog>
