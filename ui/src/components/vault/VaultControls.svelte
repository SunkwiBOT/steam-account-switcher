<script lang="ts">
  import DialogHeader from "../dialogs/DialogHeader.svelte";
  import AppDialog from "../dialogs/AppDialog.svelte";
  import { DialogContent } from "$lib/components/ui/dialog/index.js";
  import { errorMessage, resolveErrorCode, vaultErrorHint } from "../../lib/errors/errors.svelte";

  import { t } from "../../lib/i18n/i18n.svelte";

  import { VaultOperation, VaultEditor, VaultPasswordField } from "./VaultControls.constants";
  import { AlertTriangle, KeyRound, Lock, ShieldOff, Unlock, X } from "@lucide/svelte";
  import { app, applySnapshot, showToast } from "../../lib/state/state.svelte";
  import { ToastKind } from "../../lib/state/state.constants";
  import type { VaultLockInterval } from "../../lib/bindings/VaultLockInterval";
  import SelectSetting from "../ui/SelectSetting.svelte";
  import { invoke } from "../../lib/api/tauri";
  import type { SnapshotDto } from "../../lib/bindings/SnapshotDto";
  import { vaultCountdown } from "../../lib/vault/vault-lock";
  import AsyncIcon from "../ui/AsyncIcon.svelte";
  import CopyButton from "../ui/CopyButton.svelte";
  import PasswordInput from "../ui/PasswordInput.svelte";
  import FormError from "../ui/FormError.svelte";
  import { settingsDraft } from "../../lib/state/settings-draft.svelte";
  import UnsavedBar from "../ui/UnsavedBar.svelte";
  import { vaultEditorDirty } from "../../lib/vault/vault-editor";

  type VaultControlsPropsType = {
    onready?: () => void;
    onbusy?: (busy: boolean) => void;
    showSettings?: boolean;
  };

  let { onready, onbusy, showSettings = false }: VaultControlsPropsType = $props();
  const vault = $derived(app.snapshot?.vault);
  const uid = $props.id();
  let busy = $state(false);
  let operation = $state<VaultOperation | null>(null);
  let error = $state("");
  let passphrase = $state("");
  let editor = $state<VaultEditor | null>(null);
  let currentPassword = $state("");
  let newPassword = $state("");
  let repeated = $state("");
  let fieldError = $state<VaultPasswordField | null>(null);
  const lockIntervals = $derived<{ value: VaultLockInterval; label: string }[]>([
    { value: "STARTUP", label: t.vault_lock_startup() },
    { value: "FIVE_MINUTES", label: t.vault_lock_5_minutes() },
    { value: "TEN_MINUTES", label: t.vault_lock_10_minutes() },
    { value: "FIFTEEN_MINUTES", label: t.vault_lock_15_minutes() },
    { value: "TWENTY_MINUTES", label: t.vault_lock_20_minutes() },
    { value: "THIRTY_MINUTES", label: t.vault_lock_30_minutes() },
    { value: "ONE_HOUR", label: t.vault_lock_1_hour() },
    { value: "TWO_HOURS", label: t.vault_lock_2_hours() },
    { value: "FOUR_HOURS", label: t.vault_lock_4_hours() },
  ]);
  const lockDraft = $derived(settingsDraft.value.vaultLockInterval);
  const lockLabel = $derived(
    lockIntervals.find((option) => option.value === lockDraft)?.label ?? t.vault_lock_startup(),
  );

  const lockInSeconds = $derived(app.snapshot?.vault.lock_in_seconds ?? null);
  let remaining = $state<number | null>(null);
  // ! Anchored on a deadline, not decremented, so a throttled background timer cannot desync it.
  let lockAt = 0;
  $effect(() => {
    const seconds = lockInSeconds;
    if (seconds === null) {
      remaining = null;
      return;
    }
    lockAt = Date.now() + seconds * 1000;
    const sync = (): void => {
      remaining = Math.max(0, Math.ceil((lockAt - Date.now()) / 1000));
    };
    sync();
    const timer = setInterval(sync, 1000);
    window.addEventListener("focus", sync);
    document.addEventListener("visibilitychange", sync);
    return () => {
      clearInterval(timer);
      window.removeEventListener("focus", sync);
      document.removeEventListener("visibilitychange", sync);
    };
  });
  const remainingLabel = $derived.by(() => {
    if (remaining === null) return "";
    const parts = vaultCountdown(remaining);
    if ("hours" in parts) {
      return t.vault_lock_remaining_hours({
        hours: parts.hours,
        minutes: parts.minutes,
        seconds: parts.seconds,
      });
    }
    if ("minutes" in parts) {
      return t.vault_lock_remaining_minutes({ minutes: parts.minutes, seconds: parts.seconds });
    }
    return t.vault_lock_remaining_seconds({ seconds: parts.seconds });
  });

  const selectLockInterval = (value: string): void => {
    const interval = lockIntervals.find((option) => option.value === value)?.value;
    if (interval) settingsDraft.set("vaultLockInterval", interval);
  };

  const editorTitle = $derived(
    editor === VaultEditor.DISABLE
      ? t.vault_password_disable_title()
      : editor === VaultEditor.CHANGE
        ? t.vault_password_change_title()
        : t.vault_password_enable_title(),
  );

  const openEditor = (kind: NonNullable<typeof editor>): void => {
    if (busy) return;
    error = "";
    fieldError = null;
    currentPassword = newPassword = repeated = "";
    editor = kind;
  };

  const closeEditor = (): void => {
    if (busy) return;
    editor = null;
    currentPassword = newPassword = repeated = "";
    error = "";
    fieldError = null;
  };

  const perform = async (
    action: () => Promise<SnapshotDto>,
    title: string,
    kind: NonNullable<typeof operation>,
    ready = false,
  ): Promise<boolean> => {
    if (busy) return false;
    busy = true;
    operation = kind;
    onbusy?.(true);
    error = "";
    try {
      applySnapshot(await action());
      passphrase = "";
      showToast(ToastKind.SUCCESS, title);
      if (ready) onready?.();
      return true;
    } catch (cause: unknown) {
      error = resolveErrorCode(cause);
      return false;
    } finally {
      busy = false;
      operation = null;
      onbusy?.(false);
    }
  };

  const savePassword = async (event: SubmitEvent): Promise<void> => {
    event.preventDefault();
    if (busy || !editor) return;
    error = "";
    fieldError = null;
    if (editor === VaultEditor.DISABLE || editor === VaultEditor.CHANGE) {
      if (!currentPassword) {
        error = "CURRENT_PASSWORD_REQUIRED";
        fieldError = VaultPasswordField.CURRENT;
        return;
      }
    }
    if (editor !== VaultEditor.DISABLE) {
      if (!newPassword.trim()) {
        error = "VAULT_PASSWORD_REQUIRED";
        fieldError = VaultPasswordField.NEW;
        return;
      }
      if (newPassword !== repeated) {
        error = "PASSWORD_MISMATCH";
        fieldError = VaultPasswordField.REPEAT;
        return;
      }
    }
    const removing = editor === VaultEditor.DISABLE;
    const saved = await perform(
      () =>
        invoke("authenticator_set_passphrase", {
          passphrase: removing ? null : newPassword,
          currentPassword: editor === VaultEditor.ENABLE ? null : currentPassword,
        }),
      removing ? t.vault_password_removed() : t.vault_password_saved(),
      VaultOperation.SAVE,
    );
    if (saved) {
      closeEditor();
      if (!removing) onready?.();
    } else if (error === "WRONG_VAULT_PASSWORD" || error === "CURRENT_PASSWORD_REQUIRED")
      fieldError = VaultPasswordField.CURRENT;
  };

  const unlock = (): Promise<boolean> => {
    return perform(
      () => invoke("authenticator_unlock", { passphrase }),
      t.vault_unlock_success(),
      VaultOperation.UNLOCK,
      true,
    );
  };

  const dirty = $derived(
    vaultEditorDirty(
      { currentPassword, newPassword, repeated },
      { disabling: editor === VaultEditor.DISABLE, requiresCurrent: editor === VaultEditor.CHANGE },
    ),
  );
</script>

{#if vault?.error}
  <p class="steamguard-error" role="alert">
    <AlertTriangle size={15} />
    {errorMessage(vault.error)}
  </p>
  <p class="steamguard-note">
    {vaultErrorHint(vault.error)}
  </p>
{:else if vault?.locked}
  <label class="field-label" for={`${uid}-unlock`}>{t.vault_password_label()}</label>
  <div class="steamguard-row">
    <PasswordInput
      id={`${uid}-unlock`}
      label={t.vault_password_label()}
      autocomplete="current-password"
      bind:value={passphrase}
      disabled={busy}
      invalid={Boolean(error)}
      describedby={error ? `${uid}-unlock-error` : undefined}
      onkeydown={(event) => event.key === "Enter" && void unlock()}
    />
    <button class="button primary" type="button" disabled={busy} onclick={unlock}>
      <AsyncIcon icon={Unlock} size={15} busy={operation === VaultOperation.UNLOCK} />
      {operation === VaultOperation.UNLOCK ? t.common_unlocking() : t.vault_unlock_action()}
    </button>
  </div>
  <FormError id={`${uid}-unlock-error`} message={error ? errorMessage(error) : ""} />
{:else}
  <div class="steamguard-actions">
    {#if vault?.encrypted}
      <button
        class="button"
        type="button"
        disabled={busy}
        onclick={() =>
          perform(() => invoke("authenticator_lock"), t.vault_lock_success(), VaultOperation.LOCK)}
        ><AsyncIcon icon={Lock} size={15} busy={operation === VaultOperation.LOCK} />
        {operation === VaultOperation.LOCK ? t.common_locking() : t.vault_lock_action()}</button
      >
      <button
        class="button"
        type="button"
        disabled={busy}
        onclick={() => openEditor(VaultEditor.CHANGE)}
        ><KeyRound size={15} /> {t.vault_password_change_action()}</button
      >
      <button
        class="button"
        type="button"
        disabled={busy}
        onclick={() => openEditor(VaultEditor.DISABLE)}
        ><ShieldOff size={15} /> {t.vault_password_disable_action()}</button
      >
    {:else}
      <button
        class="button primary"
        type="button"
        disabled={busy}
        onclick={() => openEditor(VaultEditor.ENABLE)}
        ><KeyRound size={15} /> {t.vault_password_enable_action()}</button
      >
    {/if}
  </div>
  {#if error && !editor}<p class="steamguard-error vault-field-error" role="alert">
      <AlertTriangle size={15} />
      {errorMessage(error)}
    </p>{/if}
{/if}

{#if showSettings && vault?.encrypted && !vault?.locked}
  <div class="settings-update-frequency vault-lock">
    <span class="field-label">{t.vault_lock_label()}</span>
    <div class="vault-lock-row">
      <SelectSetting
        value={lockDraft}
        options={lockIntervals}
        label={lockLabel}
        ariaLabel={t.vault_lock_label()}
        disabled={settingsDraft.saving}
        onselect={selectLockInterval}
      />
      {#if remainingLabel}<span class="steamguard-note vault-lock-remaining">{remainingLabel}</span
        >{/if}
    </div>
    <p class="steamguard-note">
      {lockDraft === "STARTUP" ? t.vault_lock_startup_description() : t.vault_lock_description()}
    </p>
  </div>
{/if}

{#if showSettings && vault?.path}
  <div class="vault-path">
    <label class="field-label" for={`${uid}-path`}>{t.vault_file_label()}</label>
    <div class="vault-path-row">
      <input id={`${uid}-path`} class="input" readonly value={vault.path} title={vault.path} />
      <CopyButton
        valueLabel={t.vault_file_label()}
        copyKey={`vault-path:${vault.path}`}
        label={t.vault_path_copy()}
        read={() => vault?.path ?? ""}
      />
    </div>
    <p class="steamguard-note">
      {t.vault_storage_description()}
    </p>
  </div>
{/if}

<AppDialog open={Boolean(editor)} onclose={closeEditor}>
  <DialogContent
    showCloseButton={false}
    class="sm:max-w-md"
    role={editor === VaultEditor.DISABLE ? "alertdialog" : "dialog"}
    aria-labelledby={`${uid}-editor-title`}
    aria-describedby={editor === VaultEditor.DISABLE ? `${uid}-disable-warning` : undefined}
  >
    <DialogHeader
      id={`${uid}-editor-title`}
      title={editorTitle}
      icon={editor === VaultEditor.DISABLE ? AlertTriangle : KeyRound}
      onclose={closeEditor}
      closeLabel={t.vault_password_close()}
      closeDisabled={busy}
    />
    <form class="vault-password-form" onsubmit={savePassword}>
      {#if editor === VaultEditor.DISABLE}
        <p id={`${uid}-disable-warning`} class="dialog-text">
          {t.vault_password_disable_warning()}
        </p>
        <label class="field-label" for={`${uid}-current`}>{t.vault_password_current_label()}</label>
        <PasswordInput
          id={`${uid}-current`}
          label={t.vault_password_current_label()}
          autocomplete="current-password"
          bind:value={currentPassword}
          disabled={busy}
          invalid={fieldError === VaultPasswordField.CURRENT}
          describedby={fieldError === VaultPasswordField.CURRENT
            ? `${uid}-current-error`
            : undefined}
        />
        <FormError
          id={`${uid}-current-error`}
          message={fieldError === VaultPasswordField.CURRENT ? errorMessage(error) : ""}
        />
      {:else}
        {#if editor === VaultEditor.CHANGE}
          <label class="field-label" for={`${uid}-current`}
            >{t.vault_password_current_label()}</label
          >
          <PasswordInput
            id={`${uid}-current`}
            label={t.vault_password_current_label()}
            autocomplete="current-password"
            bind:value={currentPassword}
            disabled={busy}
            invalid={fieldError === VaultPasswordField.CURRENT}
            describedby={fieldError === VaultPasswordField.CURRENT
              ? `${uid}-current-error`
              : undefined}
          />
          <FormError
            id={`${uid}-current-error`}
            message={fieldError === VaultPasswordField.CURRENT ? errorMessage(error) : ""}
          />
        {/if}
        <label class="field-label" for={`${uid}-new`}
          >{editor === VaultEditor.CHANGE
            ? t.vault_password_new_label()
            : t.vault_password_label()}</label
        >
        <PasswordInput
          id={`${uid}-new`}
          label={editor === VaultEditor.CHANGE
            ? t.vault_password_new_label()
            : t.vault_password_label()}
          autocomplete="new-password"
          bind:value={newPassword}
          disabled={busy}
          invalid={fieldError === VaultPasswordField.NEW}
          describedby={fieldError === VaultPasswordField.NEW ? `${uid}-new-error` : undefined}
        />
        <FormError
          id={`${uid}-new-error`}
          message={fieldError === VaultPasswordField.NEW ? errorMessage(error) : ""}
        />
        <label class="field-label" for={`${uid}-repeat`}>{t.vault_password_repeat_label()}</label>
        <PasswordInput
          id={`${uid}-repeat`}
          label={t.vault_password_repeat_label()}
          autocomplete="new-password"
          bind:value={repeated}
          disabled={busy}
          invalid={fieldError === VaultPasswordField.REPEAT}
          describedby={fieldError === VaultPasswordField.REPEAT ? `${uid}-repeat-error` : undefined}
        />
        <FormError
          id={`${uid}-repeat-error`}
          message={fieldError === VaultPasswordField.REPEAT ? errorMessage(error) : ""}
        />
      {/if}
      <FormError message={error && !fieldError ? errorMessage(error) : ""} />
      {#if !dirty}
        <footer class="dialog-foot">
          <button class="button" type="button" disabled={busy} onclick={closeEditor}
            ><X size={15} /> {t.common_cancel()}</button
          >
          <button
            class="button"
            class:danger={editor === VaultEditor.DISABLE}
            class:primary={editor !== VaultEditor.DISABLE}
            type="submit"
            disabled={busy}
            ><AsyncIcon
              icon={editor === VaultEditor.DISABLE ? ShieldOff : KeyRound}
              busy={operation === VaultOperation.SAVE}
              size={15}
            />{operation === VaultOperation.SAVE
              ? editor === VaultEditor.DISABLE
                ? t.common_disabling()
                : t.common_saving()
              : editor === VaultEditor.DISABLE
                ? t.vault_password_disable_action()
                : t.vault_password_save()}</button
          >
        </footer>
      {:else}
        <UnsavedBar
          type="submit"
          saving={busy}
          oncancel={closeEditor}
          saveIcon={editor === VaultEditor.DISABLE ? ShieldOff : KeyRound}
          saveLabel={editor === VaultEditor.DISABLE
            ? t.vault_password_disable_action()
            : t.vault_password_save()}
          savingLabel={editor === VaultEditor.DISABLE ? t.common_disabling() : t.common_saving()}
          danger={editor === VaultEditor.DISABLE}
        />
      {/if}
    </form>
  </DialogContent>
</AppDialog>
