<script lang="ts">
  import DialogHeader from "./DialogHeader.svelte";
  import { t, rich } from "../../lib/i18n/i18n.svelte";

  import { ConfirmKind } from "../../lib/state/dialogs.constants";
  import { answerConfirm, confirmDialog } from "../../lib/state/dialogs.svelte";
  import { accountById } from "../../lib/state/state.svelte";

  import MessageText from "../ui/MessageText.svelte";
  import { containsAccountName } from "../../lib/i18n/messages";
  const entry = $derived(accountById(confirmDialog.steamId ?? ""));
  const inlineAccount = $derived(
    entry && containsAccountName(confirmDialog.content, entry.display_name),
  );

  import { AlertTriangle, ArrowRightLeft, Check, Play, Trash2, UserPlus, X } from "@lucide/svelte";
  import { untrack } from "svelte";
  import { cycleFocusIndex } from "../../lib/state/keyboard";

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

  let dialog = $state<HTMLDivElement | null>(null);
  let cancelButton = $state<HTMLButtonElement | null>(null);

  // Focus starts on the safe choice, stays inside the dialog while open, and
  // returns to the control that opened it when the dialog closes.
  $effect(() => {
    const element = dialog;
    if (!element) return;
    const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    untrack(() => cancelButton)?.focus();
    const trapFocus = (event: KeyboardEvent): void => {
      if (event.key !== "Tab") return;
      const focusable = [
        ...element.querySelectorAll<HTMLElement>("button:not(:disabled), input:not(:disabled)"),
      ];
      const next = cycleFocusIndex(
        focusable.findIndex((item) => item === document.activeElement),
        focusable.length,
        event.shiftKey,
      );
      if (next < 0) return;
      event.preventDefault();
      focusable[next]?.focus();
    };
    element.addEventListener("keydown", trapFocus);
    return () => {
      element.removeEventListener("keydown", trapFocus);
      previous?.focus();
    };
  });
</script>

{#if confirmDialog.open}
  <div class="modal">
    <div class="modal-backdrop"></div>
    <div
      class="dialog small"
      role="dialog"
      aria-modal="true"
      aria-labelledby={`${uid}-title`}
      tabindex="-1"
      bind:this={dialog}
    >
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
          <input type="checkbox" bind:checked />
          <span>{confirmDialog.checkboxLabel}</span>
        </label>
      {/if}
      <footer class="dialog-foot">
        <button
          class="button"
          type="button"
          bind:this={cancelButton}
          onclick={() => answerConfirm(null)}><X size={15} /> {t.common_cancel()}</button
        >
        <button
          class="button"
          class:primary={!confirmDialog.danger}
          class:danger={confirmDialog.danger}
          type="button"
          disabled={confirmDialog.checkboxRequired && !checked}
          onclick={() => answerConfirm({ cleanup: checked })}
        >
          <ActionIcon size={15} />
          {confirmDialog.confirmLabel}
        </button>
      </footer>
    </div>
  </div>
{/if}
