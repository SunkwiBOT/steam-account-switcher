<script lang="ts">
  import { Download, Upload, X } from "@lucide/svelte";
  import { BackupMode, BackupProtection, BackupField } from "../../lib/vault/backup.constants";
  import { errorMessage, resolveErrorCode, type UiErrorCode } from "../../lib/errors/errors.svelte";
  import { app, accountById, applySnapshot, showToast } from "../../lib/state/state.svelte";
  import { ToastKind } from "../../lib/state/state.constants";
  import { t, rich } from "../../lib/i18n/i18n.svelte";
  import { invoke } from "../../lib/api/tauri";
  import DialogHeader from "../dialogs/DialogHeader.svelte";
  import AppDialog from "../dialogs/AppDialog.svelte";
  import { DialogContent } from "$lib/components/ui/dialog/index.js";
  import { Switch } from "$lib/components/ui/switch/index.js";
  import MessageText from "../ui/MessageText.svelte";
  import ActionButton from "../ui/ActionButton.svelte";
  import FormError from "../ui/FormError.svelte";
  import PasswordInput from "../ui/PasswordInput.svelte";
  import VaultControls from "./VaultControls.svelte";

  const request = $derived(app.backup);
  const exporting = $derived(request?.mode === BackupMode.EXPORT);
  const entry = $derived(accountById(request?.steamId ?? ""));
  const vault = $derived(app.snapshot?.vault);
  const ready = $derived(!vault?.locked && !vault?.error);
  const title = $derived(exporting ? t.backup_export_title() : t.backup_import_title());
  const uid = $props.id();
  let password = $state("");
  let repeated = $state("");
  let protection = $state(BackupProtection.PASSWORD);
  let busy = $state(false);
  let vaultBusy = $state(false);
  let error = $state<UiErrorCode | null>(null);
  let errorField = $state(BackupField.GENERAL);
  const pending = $derived(busy || vaultBusy);
  const fieldError = (field: BackupField): string =>
    error && errorField === field
      ? exporting && error === "BACKUP_PASSWORD_REQUIRED"
        ? t.backup_password_required()
        : errorMessage(error)
      : "";

  $effect(() => {
    if (!request) return;
    password = repeated = "";
    error = null;
    protection = BackupProtection.PASSWORD;
  });

  const close = (): void => {
    if (pending) return;
    app.backup = null;
    password = repeated = "";
    error = null;
  };

  const chooseProtection = (value: BackupProtection): void => {
    if (pending) return;
    protection = value;
    password = repeated = "";
    error = null;
  };

  const submit = async (event: SubmitEvent): Promise<void> => {
    event.preventDefault();
    if (pending || !request || !ready) return;
    const current = request;
    const account = entry;
    error = null;
    if (exporting && protection === BackupProtection.PASSWORD) {
      if (!password.trim()) {
        error = "BACKUP_PASSWORD_REQUIRED";
        errorField = BackupField.PASSWORD;
        return;
      }
      if (password !== repeated) {
        error = "PASSWORD_MISMATCH";
        errorField = BackupField.REPEAT;
        return;
      }
    }
    busy = true;
    try {
      if (exporting) {
        // ! The backend opens the "save as" dialog itself, so the page never decides where secrets are written.
        const count = await invoke("steam_guard_export_backup", {
          steamId: current.steamId,
          password: protection === BackupProtection.PASSWORD ? password : null,
        });
        if (app.backup !== current || count === null) return;
        showToast(ToastKind.SUCCESS, t.toast_backup_export_title(), {
          description: account
            ? rich(t.backup_export_account_success, { account: account.display_name })
            : rich(t.backup_export_success, { accounts: t.accounts_count({ count }) }),
          account,
        });
      } else {
        // ! The source is chosen in a native dialog opened by the backend, so the page never picks which file is read.
        const result = await invoke("steam_guard_import_backup", {
          password: password || null,
        });
        if (app.backup !== current || result === null) return;
        applySnapshot(result.snapshot);
        showToast(ToastKind.SUCCESS, t.toast_backup_import_title(), {
          description: rich(t.backup_import_success, {
            imported: t.accounts_count({ count: result.imported }),
            skipped: t.accounts_count({ count: result.skipped }),
          }),
        });
      }
      if (app.backup === current) {
        app.backup = null;
        password = repeated = "";
      }
    } catch (cause: unknown) {
      if (app.backup !== current) return;
      error = resolveErrorCode(cause);
      errorField =
        error === "BACKUP_PASSWORD_REQUIRED" || error === "BACKUP_DECRYPTION_FAILED"
          ? BackupField.PASSWORD
          : BackupField.GENERAL;
    } finally {
      busy = false;
    }
  };
</script>

<AppDialog open={Boolean(request)} onclose={close}>
  <DialogContent showCloseButton={false} class="sm:max-w-md" aria-labelledby={`${uid}-title`}>
    <DialogHeader
      id={`${uid}-title`}
      {title}
      icon={exporting ? Download : Upload}
      onclose={close}
      closeDisabled={pending}
    />
    {#if !ready}
      {#if !vault?.error}
        <p class="steamguard-note">{t.backup_vault_locked()}</p>
      {/if}
      <VaultControls onbusy={(value) => (vaultBusy = value)} />
      <footer class="dialog-foot">
        <button class="button" type="button" disabled={pending} onclick={close}
          ><X size={15} /> {t.common_cancel()}</button
        >
      </footer>
    {:else}
      <form onsubmit={submit}>
        <p class="steamguard-note">
          <MessageText
            content={exporting
              ? entry
                ? rich(t.backup_export_account_description, { account: entry.display_name })
                : rich(t.backup_export_description, {
                    accounts: t.accounts_count({ count: vault?.count ?? 0 }),
                  })
              : rich(t.backup_import_description, { format: `.${app.info.backup_extension}` }, [
                  "format",
                ])}
            account={entry}
          />
        </p>
        {#if exporting}
          <label class="setting-line">
            <span>{t.backup_protection_label()}</span>
            <span class="setting-control">
              <Switch
                checked={protection === BackupProtection.PASSWORD}
                disabled={pending}
                onCheckedChange={(next) =>
                  chooseProtection(next ? BackupProtection.PASSWORD : BackupProtection.NONE)}
              />
            </span>
          </label>
          {#if protection === BackupProtection.NONE}<p class="history-notice">
              {t.backup_unprotected_warning()}
            </p>{/if}
        {/if}
        {#if !exporting || protection === BackupProtection.PASSWORD}
          {#if exporting}<p class="steamguard-note">{t.backup_password_description()}</p>{/if}
          <label class="field-label" for={`${uid}-password`}
            >{t.guard_import_password_label()}</label
          >
          <PasswordInput
            id={`${uid}-password`}
            label={t.guard_import_password_label()}
            placeholder={exporting ? "" : t.guard_import_password_placeholder()}
            autocomplete={exporting ? "new-password" : "off"}
            bind:value={password}
            disabled={pending}
            invalid={Boolean(fieldError(BackupField.PASSWORD))}
            describedby={fieldError(BackupField.PASSWORD) ? `${uid}-password-error` : undefined}
          />
          <FormError id={`${uid}-password-error`} message={fieldError(BackupField.PASSWORD)} />
          {#if exporting}
            <label class="field-label" for={`${uid}-repeat`}>{t.backup_password_repeat()}</label>
            <PasswordInput
              id={`${uid}-repeat`}
              label={t.backup_password_repeat()}
              autocomplete="new-password"
              bind:value={repeated}
              disabled={pending}
              invalid={Boolean(fieldError(BackupField.REPEAT))}
              describedby={fieldError(BackupField.REPEAT) ? `${uid}-repeat-error` : undefined}
            />
            <FormError id={`${uid}-repeat-error`} message={fieldError(BackupField.REPEAT)} />
          {/if}
        {/if}
        <FormError message={fieldError(BackupField.GENERAL)} />
        <footer class="dialog-foot">
          <button class="button" type="button" disabled={pending} onclick={close}
            ><X size={15} /> {t.common_cancel()}</button
          >
          <ActionButton
            icon={exporting ? Download : Upload}
            label={exporting ? t.backup_export_action() : t.common_import()}
            busyLabel={exporting ? t.backup_export_pending() : t.common_importing()}
            {busy}
            disabled={pending}
            type="submit"
            className="button primary"
          />
        </footer>
      </form>
    {/if}
  </DialogContent>
</AppDialog>
