<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";

  import { Eye, EyeOff } from "@lucide/svelte";

  type PasswordInputPropsType = {
    value?: string;
    id?: string;
    label: string;
    placeholder?: string;
    autocomplete?: "off" | "current-password" | "new-password";
    disabled?: boolean;
    invalid?: boolean;
    describedby?: string;
    onkeydown?: (event: KeyboardEvent) => void;
  };

  let {
    value = $bindable(""),
    id,
    label,
    placeholder = "",
    autocomplete = "off",
    disabled = false,
    invalid = false,
    describedby,
    onkeydown,
  }: PasswordInputPropsType = $props();
  let visible = $state(false);
  const toggleLabel = $derived(
    visible
      ? t.common_hide_value({ value: label.toLocaleLowerCase() })
      : t.common_show_value({ value: label.toLocaleLowerCase() }),
  );
</script>

<div class="password-input">
  <input
    class="input"
    {id}
    type={visible ? "text" : "password"}
    bind:value
    {placeholder}
    {autocomplete}
    {disabled}
    aria-label={label}
    aria-invalid={invalid}
    aria-describedby={describedby}
    {onkeydown}
  />
  <button
    class="password-toggle"
    type="button"
    title={toggleLabel}
    aria-label={toggleLabel}
    aria-pressed={visible}
    {disabled}
    onmousedown={(event) => event.preventDefault()}
    onclick={() => (visible = !visible)}
  >
    {#if visible}<EyeOff size={15} />{:else}<Eye size={15} />{/if}
  </button>
</div>
