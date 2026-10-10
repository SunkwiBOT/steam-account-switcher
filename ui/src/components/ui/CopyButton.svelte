<script lang="ts">
  import { errorMessage } from "../../lib/errors/errors.svelte";

  import { t } from "../../lib/i18n/i18n.svelte";

  import { copyFeedback, copyValue } from "../../lib/state/clipboard.svelte";
  import { accountById, showToast } from "../../lib/state/state.svelte";
  import CopyIndicator from "./CopyIndicator.svelte";

  type CopyButtonPropsType = {
    steamId?: string;
    copyKey: string;
    label: string;
    valueLabel?: string;
    read?: () => string | Promise<string>;
    action?: () => Promise<boolean>;
    disabled?: boolean;
    text?: string;
    className?: string;
    guard?: boolean;
    title?: string;
  };

  let {
    steamId,
    copyKey,
    label,
    valueLabel = t.clipboard_value_label(),
    read,
    action,
    disabled = false,
    text = "",
    className = "copy-button",
    guard = false,
    title = label,
  }: CopyButtonPropsType = $props();
  let pending = $state(false);

  const copy = async (event: MouseEvent): Promise<void> => {
    event.stopPropagation();
    if (pending || disabled) return;
    const key = copyKey;
    const account = steamId ? accountById(steamId) : null;
    pending = true;
    try {
      if (action) await action();
      else if (read) {
        const value = await read();
        if (key === copyKey) await copyValue(value, key, valueLabel, account);
      }
    } catch (cause) {
      showToast(errorMessage(cause), false, account);
    } finally {
      pending = false;
    }
  };
</script>

<button
  class={className}
  type="button"
  title={pending ? t.clipboard_copy_pending() : title}
  aria-label={pending ? t.clipboard_copy_pending() : label}
  aria-busy={pending}
  disabled={disabled || pending}
  onclick={copy}
  ondblclick={(event) => event.stopPropagation()}
>
  <CopyIndicator copied={copyFeedback[copyKey] === true} {pending} {guard} />{text && pending
    ? t.clipboard_copy_pending()
    : text}
  {#if copyFeedback[copyKey]}<span class="sr-only" role="status">{t.clipboard_copied()}</span>{/if}
</button>
