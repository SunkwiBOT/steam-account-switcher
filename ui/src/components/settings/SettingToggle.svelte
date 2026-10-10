<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";

  import AsyncIcon from "../ui/AsyncIcon.svelte";

  type SettingTogglePropsType = {
    label: string;
    checked: boolean;
    disabled?: boolean;
    onchange: (enabled: boolean) => Promise<unknown>;
  };

  let { label, checked, disabled = false, onchange }: SettingTogglePropsType = $props();
  let busy = $state(false);

  const change = async (event: Event): Promise<void> => {
    const input = event.currentTarget;
    if (!(input instanceof HTMLInputElement)) return;
    if (busy || disabled) {
      input.checked = checked;
      return;
    }
    busy = true;
    try {
      await onchange(input.checked);
    } finally {
      input.checked = checked;
      busy = false;
    }
  };
</script>

<label class="setting-line">
  <span>{label}</span>
  <span class="setting-control">
    {#if busy}<AsyncIcon size={16} /><span class="sr-only" role="status">{t.common_saving()}</span
      >{/if}
    <input
      class="switch"
      type="checkbox"
      {checked}
      hidden={busy}
      disabled={disabled || busy}
      aria-busy={busy}
      onchange={change}
    />
  </span>
</label>
