<script lang="ts">
  import { Download, Upload, FileSearch, X } from "@lucide/svelte";
  import {
    BackupMode,
    BackupProtection,
    BackupField,
    BACKUP_EXTENSION,
  } from "../../lib/vault/backup.constants";
  import { ErrorApiMessage } from "../../lib/errors/errors.constants";
  import { errorMessage, resolveErrorCode } from "../../lib/errors/errors.svelte";
  import { app, accountById, applySnapshot, showToast } from "../../lib/state/state.svelte";
  import { t, rich } from "../../lib/i18n/i18n.svelte";
  import { invoke, invokePlugin } from "../../lib/api/tauri";
  import DialogHeader from "../dialogs/DialogHeader.svelte";
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
  let dialog = $state<HTMLDialogElement>();
  let password = $state("");
  let repeated = $state("");
  let path = $state("");
  let protection = $state(BackupProtection.PASSWORD);
  let busy = $state(false);
  let browsing = $state(false);
  let vaultBusy = $state(false);
  let error = $state<ErrorApiMessage | null>(null);
  let errorField = $state(BackupField.GENERAL);
  const pending = $derived(busy || browsing || vaultBusy);
  const fieldError = (field: BackupField): string =>
    error && errorField === field
      ? exporting && error === ErrorApiMessage.BACKUP_PASSWORD_REQUIRED
        ? t.backup_password_required()
        : errorMessage(error)
      : "";

  $effect(() => {
    const current = request;
    if (!dialog) return;
    if (current) {
      password = repeated = path = "";
      error = null;
      protection = BackupProtection.PASSWORD;
      if (!dialog.open) {
        dialog.showModal();
        dialog.querySelector("input")?.focus();
      }
    } else if (dialog.open) dialog.close();
  });

  const close = (): void => {
    if (pending) return;
    app.backup = null;
    password = repeated = path = "";
    error = null;
  };

  const chooseProtection = (value: BackupProtection): void => {
    if (pending) return;
    protection = value;
    password = repeated = "";
    error = null;
  };

  const pickFile = async (): Promise<void> => {
    if (pending) return;
    const current = request;
    browsing = true;
    error = null;
    try {
      const chosen = await invokePlugin("plugin:dialog|open", {
        options: {
          title: t.backup_import_title(),
          multiple: false,
          directory: false,
          filters: [{ name: t.backup_file_filter(), extensions: [BACKUP_EXTENSION] }],
        },
      });
      if (app.backup === current && typeof chosen === "string") path = chosen;
    } catch (cause) {
      if (app.backup !== current) return;
      error = resolveErrorCode(cause);
      errorField = BackupField.FILE;
    } finally {
      browsing = false;
    }
  };

  const submit = async (event: SubmitEvent): Promise<void> => {
    event.preventDefault();
    if (pending || !request || !ready) return;
    const current = request;
    const account = entry;
    error = null;
    if (!exporting && !path) {
      error = ErrorApiMessage.BACKUP_FILE_REQUIRED;
      errorField = BackupField.FILE;
      return;
    }
    if (exporting && protection === BackupProtection.PASSWORD) {
      if (!password.trim()) {
        error = ErrorApiMessage.BACKUP_PASSWORD_REQUIRED;
        errorField = BackupField.PASSWORD;
        return;
      }
      if (password !== repeated) {
        error = ErrorApiMessage.PASSWORD_MISMATCH;
        errorField = BackupField.REPEAT;
        return;
      }
    }
    busy = true;
    try {
      if (exporting) {
        //! The backend opens the "save as" dialog itself, so the page never
        //! decides where secrets are written.
        const count = await invoke("steam_guard_export_backup", {
          steamId: current.steamId,
          password: protection === BackupProtection.PASSWORD ? password : null,
        });
        if (app.backup !== current || count === null) return;
        showToast(
          account
            ? rich(t.backup_export_account_success, { account: account.display_name })
            : rich(t.backup_export_success, { accounts: t.accounts_count({ count }) }),
          true,
          account,
        );
      } else {
        const result = await invoke("steam_guard_import_backup", {
          path,
          password: password || null,
        });
        applySnapshot(result.snapshot);
        showToast(
          rich(t.backup_import_success, {
            imported: t.accounts_count({ count: result.imported }),
            skipped: t.accounts_count({ count: result.skipped }),
          }),
          true,
        );
      }
      if (app.backup === current) {
        app.backup = null;
        password = repeated = path = "";
      }
    } catch (cause) {
      if (app.backup !== current) return;
      error = resolveErrorCode(cause);
      errorField =
        error === ErrorApiMessage.BACKUP_PASSWORD_REQUIRED ||
        error === ErrorApiMessage.BACKUP_DECRYPTION_FAILED
          ? BackupField.PASSWORD
          : exporting
            ? BackupField.GENERAL
            : BackupField.FILE;
    } finally {
      busy = false;
    }
  };
</script>

{#if request}
  <dialog
    class="dialog small backup-dialog"
    bind:this={dialog}
    aria-labelledby={`${uid}-title`}
    aria-modal="true"
    oncancel={(event) => {
      event.preventDefault();
      close();
    }}
    onkeydown={(event) => {
      if (event.key === "Escape") {
        event.preventDefault();
        event.stopPropagation();
        close();
      }
    }}
  >
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
              : rich(t.backup_import_description, { format: `.${BACKUP_EXTENSION}` }, ["format"])}
            account={entry}
          />
        </p>
        {#if exporting}
          <label class="setting-line">
            <span>{t.backup_protection_label()}</span>
            <span class="setting-control">
              <input
                class="switch"
                type="checkbox"
                checked={protection === BackupProtection.PASSWORD}
                disabled={pending}
                onchange={(event) =>
                  chooseProtection(
                    event.currentTarget.checked ? BackupProtection.PASSWORD : BackupProtection.NONE,
                  )}
              />
            </span>
          </label>
          {#if protection === BackupProtection.NONE}<p class="history-notice">
              {t.backup_unprotected_warning()}
            </p>{/if}
        {:else}
          <label class="field-label" for={`${uid}-file`}>{t.backup_file_label()}</label>
          <div class="steamguard-row">
            <div class="steamguard-field">
              <input
                id={`${uid}-file`}
                class="input"
                readonly
                value={path}
                placeholder={t.common_file_empty()}
                disabled={pending}
                aria-invalid={Boolean(fieldError(BackupField.FILE))}
                aria-describedby={fieldError(BackupField.FILE) ? `${uid}-file-error` : undefined}
              />
            </div>
            <ActionButton
              icon={FileSearch}
              label={t.common_browse()}
              busyLabel={t.common_opening()}
              busy={browsing}
              disabled={pending}
              onclick={pickFile}
            />
          </div>
          <FormError id={`${uid}-file-error`} message={fieldError(BackupField.FILE)} />
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
  </dialog>
{/if}
