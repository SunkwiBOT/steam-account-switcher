<script lang="ts">
  import { t, rich } from "../../lib/i18n/i18n.svelte";
  import MessageText from "../ui/MessageText.svelte";

  import {
    GuardAction,
    LoginPurpose,
    SteamGuardField,
    GuardKind,
  } from "../../lib/steam-guard/steam-guard.constants";
  import ActionButton from "../ui/ActionButton.svelte";
  import type { SteamGuardControllerType } from "../../lib/steam-guard/steam-guard.svelte";
  import type { AccountType } from "../../lib/types";
  import { ArrowRight, LogIn, QrCode, X } from "@lucide/svelte";
  import AccountIdentity from "../accounts/AccountIdentity.svelte";
  import PasswordInput from "../ui/PasswordInput.svelte";
  import SteamGuardCode from "./SteamGuardCode.svelte";
  import FormError from "../ui/FormError.svelte";
  type SteamGuardLoginPropsType = { controller: SteamGuardControllerType; entry: AccountType };

  let { controller, entry }: SteamGuardLoginPropsType = $props();
  const uid = $props.id();
</script>

<section class="steamguard-card">
  <h3 class="block-title">
    <LogIn size={15} />
    {t.guard_login_title()}
  </h3>
  {#if !controller.loginBegin}
    <p class="steamguard-note">
      {#if controller.loginPurpose === LoginPurpose.LINK}
        <MessageText
          content={rich(t.guard_login_link_description, { account: entry.display_name })}
          account={entry}
        />
      {:else}
        <MessageText
          content={rich(t.guard_login_confirmations_description, { account: entry.display_name })}
          account={entry}
        />
      {/if}
    </p>
    <div class="steamguard-row">
      <span class="field-label">{t.guard_login_name_label()}</span>
      <AccountIdentity {entry} />
    </div>
    <div class="steamguard-row">
      <div class="steamguard-field">
        <PasswordInput
          label={t.guard_login_password_label()}
          placeholder={t.common_password()}
          autocomplete="current-password"
          bind:value={controller.password}
          disabled={controller.busy}
          invalid={Boolean(controller.errorFor(SteamGuardField.PASSWORD))}
          describedby={controller.errorFor(SteamGuardField.PASSWORD)
            ? `${uid}-password-error`
            : undefined}
          onkeydown={(event) => event.key === "Enter" && controller.startPassword()}
        />
        <FormError
          id={`${uid}-password-error`}
          message={controller.errorFor(SteamGuardField.PASSWORD)}
        />
      </div>
      <ActionButton
        icon={LogIn}
        label={t.guard_login_action()}
        busyLabel={t.guard_login_pending()}
        busy={controller.busyAction === GuardAction.PASSWORD}
        disabled={controller.busy}
        className="button primary"
        onclick={controller.startPassword}
      />
    </div>
    <div class="steamguard-actions">
      <ActionButton
        icon={QrCode}
        label={t.guard_login_qr_action()}
        busyLabel={t.guard_login_qr_pending()}
        busy={controller.busyAction === GuardAction.QR}
        disabled={controller.busy}
        className="button"
        onclick={controller.startQr}
      />
    </div>
  {:else if controller.loginBegin.qr_svg}
    <p class="steamguard-note">
      <MessageText
        content={rich(t.guard_login_finish_description, { account: entry.display_name })}
        account={entry}
      />
    </p>
    <p class="steamguard-note">
      {t.guard_login_qr_description()}
    </p>
    <div class="steamguard-qr">{@html controller.loginBegin.qr_svg}</div>
    {#if controller.waiting}
      <p class="steamguard-note">{t.guard_login_approval_pending()}</p>
    {:else}
      <p class="steamguard-note">{t.guard_login_qr_expired()}</p>
    {/if}
    <ActionButton
      icon={X}
      label={t.guard_login_cancel()}
      busyLabel={t.common_cancelling()}
      busy={controller.busyAction === GuardAction.CANCEL_LOGIN}
      onclick={controller.cancelLogin}
    />
  {:else}
    <p class="steamguard-note">
      <MessageText
        content={rich(t.guard_login_finish_description, { account: entry.display_name })}
        account={entry}
      />
    </p>
    <p class="steamguard-note">
      {#if controller.needsExternalConfirmation()}
        {controller.selectedGuard === GuardKind.EMAIL_CONFIRMATION
          ? t.guard_login_email_approval_hint()
          : t.guard_login_device_approval_hint()}
      {:else if controller.guardNeedsCode}
        {controller.selectedGuard === GuardKind.EMAIL_CODE
          ? t.guard_login_email_code_hint()
          : t.guard_login_device_code_hint()}
      {:else}
        {t.guard_login_continue_hint()}
      {/if}
    </p>
    {#if controller.selectedGuard === GuardKind.DEVICE_CODE && entry.has_authenticator}
      <SteamGuardCode steamId={entry.steam_id} />
      <p class="steamguard-note">
        {t.guard_login_saved_code_hint()}
      </p>
    {/if}
    {#if controller.guardNeedsCode}
      <div class="steamguard-row">
        <div class="steamguard-field">
          <input
            class="input"
            placeholder={t.guard_code_label()}
            aria-label={t.guard_code_label()}
            aria-invalid={Boolean(controller.errorFor(SteamGuardField.LOGIN_CODE))}
            aria-describedby={controller.errorFor(SteamGuardField.LOGIN_CODE)
              ? `${uid}-code-error`
              : undefined}
            bind:value={controller.loginCode}
            disabled={controller.busy || controller.waiting}
            onkeydown={(event) => event.key === "Enter" && controller.submitPasswordCode()}
          />
          <FormError
            id={`${uid}-code-error`}
            message={controller.errorFor(SteamGuardField.LOGIN_CODE)}
          />
        </div>
      </div>
    {/if}
    <div class="steamguard-actions">
      <ActionButton
        icon={X}
        label={t.common_cancel()}
        busyLabel={t.common_cancelling()}
        busy={controller.busyAction === GuardAction.CANCEL_LOGIN}
        onclick={controller.cancelLogin}
      />
      <ActionButton
        icon={ArrowRight}
        label={t.common_continue()}
        busyLabel={controller.busyAction === GuardAction.LINK
          ? t.guard_link_pending()
          : controller.busyAction === GuardAction.LOGIN_CODE
            ? t.guard_code_generate_pending()
            : t.guard_login_pending()}
        busy={controller.busyAction === GuardAction.LOGIN ||
          controller.busyAction === GuardAction.LINK ||
          controller.busyAction === GuardAction.LOGIN_CODE ||
          controller.waiting}
        disabled={controller.busy || controller.waiting}
        className="button primary"
        onclick={controller.submitPasswordCode}
      />
    </div>
  {/if}
  <FormError message={controller.errorFor(SteamGuardField.LOGIN)} />
</section>
