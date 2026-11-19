<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";

  import AsyncIcon from "../ui/AsyncIcon.svelte";
  import { Switch } from "$lib/components/ui/switch/index.js";

  type SettingTogglePropsType = {
    label: string;
    checked: boolean;
    disabled?: boolean;
    onchange: (enabled: boolean) => Promise<unknown>;
  };

  let { label, checked, disabled = false, onchange }: SettingTogglePropsType = $props();
  let busy = $state(false);

  const change = async (next: boolean): Promise<void> => {
    if (busy || disabled) return;
    busy = true;
    try {
      await onchange(next);
    } finally {
      busy = false;
    }
  };
</script>

<label class="setting-line">
  <span>{label}</span>
  <span class="setting-control">
    {#if busy}<AsyncIcon size={16} /><span class="sr-only" role="status">{t.common_saving()}</span
      >{/if}
    <Switch
      bind:checked={() => checked, (next) => void change(next)}
      hidden={busy}
      disabled={disabled || busy}
      aria-busy={busy}
    />
  </span>
</label>
