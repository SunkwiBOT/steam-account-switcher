<script lang="ts">
  import type { LoaderCircle } from "@lucide/svelte";
  import { AlertTriangle, Check, X } from "@lucide/svelte";
  import { t } from "../../lib/i18n/i18n.svelte";
  import ActionButton from "./ActionButton.svelte";

  type UnsavedBarPropsType = {
    saving: boolean;
    oncancel: () => void;
    onsave?: () => void;
    saveLabel?: string;
    savingLabel?: string;
    saveIcon?: typeof LoaderCircle;
    saveDisabled?: boolean;
    danger?: boolean;
    type?: "button" | "submit";
  };

  let {
    saving,
    oncancel,
    onsave,
    saveLabel,
    savingLabel,
    saveIcon = Check,
    saveDisabled = false,
    danger = false,
    type = "button",
  }: UnsavedBarPropsType = $props();
</script>

<footer class="unsaved-bar">
  <span class="unsaved-bar-note"><AlertTriangle size={14} /> {t.settings_unsaved()}</span>
  <div class="unsaved-bar-actions">
    <button class="button" type="button" disabled={saving} onclick={oncancel}
      ><X size={15} /> {t.common_cancel()}</button
    >
    <ActionButton
      icon={saveIcon}
      label={saveLabel ?? t.common_save()}
      busyLabel={savingLabel ?? t.common_saving()}
      busy={saving}
      disabled={saveDisabled}
      className={danger ? "button danger" : "button primary"}
      {type}
      onclick={type === "button" ? onsave : undefined}
    />
  </div>
</footer>
