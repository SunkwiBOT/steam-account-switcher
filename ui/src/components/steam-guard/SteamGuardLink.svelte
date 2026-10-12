<script lang="ts">
  import MessageText from "../ui/MessageText.svelte";
  import { t, rich } from "../../lib/i18n/i18n.svelte";

  import ActionButton from "../ui/ActionButton.svelte";
  import type { SteamGuardControllerType } from "../../lib/steam-guard/steam-guard.svelte";
  import type { AccountType } from "../../lib/types";
  import { AlertTriangle, ArrowRightLeft, ShieldOff, Check } from "@lucide/svelte";
  import PasswordInput from "../ui/PasswordInput.svelte";
  import SteamGuardIcon from "./SteamGuardIcon.svelte";
  import FormError from "../ui/FormError.svelte";
  import {
    GuardAction,
    LinkConfirmKind,
    LinkStatus,
    SteamGuardField,
  } from "../../lib/steam-guard/steam-guard.constants";
  type SteamGuardLinkPropsType = { controller: SteamGuardControllerType; entry: AccountType };

  const linkMessages: Partial<Record<LinkStatus, () => string>> = {
    [LinkStatus.RESUME]: () => t.guard_link_resume(),
    [LinkStatus.NEEDS_PHONE]: () => t.guard_link_phone_required(),
    [LinkStatus.NEEDS_EMAIL]: () => t.guard_link_email_required(),
  };

  const linkMessage = (status: LinkStatus): string =>
    (linkMessages[status] ?? (() => t.common_unknown()))();
  let { controller, entry }: SteamGuardLinkPropsType = $props();
  const uid = $props.id();
</script>

{#if controller.link}
  <section class="steamguard-card">
    <h3 class="block-title"><SteamGuardIcon size={17} /> {t.guard_link_title()}</h3>
    <p class="steamguard-note">
      <MessageText
        content={rich(t.guard_link_description, { account: entry.display_name })}
        account={entry}
      />
    </p>
    {#if controller.link.status === LinkStatus.LINKED || controller.link.status === LinkStatus.RESUME}
      <p class="steamguard-warning">
        <AlertTriangle size={15} />
        {t.guard_link_recovery_hint()}
        <strong>{controller.link.revocation_code}</strong>
      </p>
      <p class="steamguard-note">
        {#if controller.link.status === LinkStatus.RESUME}
          {linkMessage(controller.link.status)}
        {:else if controller.link.confirm_kind === LinkConfirmKind.EMAIL}
          {t.guard_link_email_sent()}
        {:else}
          <MessageText
            content={rich(t.guard_link_sms_sent, { hint: controller.link.phone_hint || "…" })}
          />
        {/if}
      </p>
      <div class="steamguard-row">
        <div class="steamguard-field">
          <input
            class="input"
            placeholder={t.guard_link_code_label()}
            aria-label={t.guard_link_code_label()}
            aria-invalid={Boolean(controller.errorFor(SteamGuardField.LINK_CODE))}
            aria-describedby={controller.errorFor(SteamGuardField.LINK_CODE)
              ? `${uid}-code-error`
              : undefined}
            bind:value={controller.linkCode}
            disabled={controller.busy}
            onkeydown={(event) => event.key === "Enter" && controller.finalizeLink()}
          />
          <FormError
            id={`${uid}-code-error`}
            message={controller.errorFor(SteamGuardField.LINK_CODE)}
          />
        </div>
        <ActionButton
          icon={Check}
          label={t.common_finish()}
          busyLabel={t.common_finishing()}
          busy={controller.busyAction === GuardAction.FINALIZE}
          disabled={controller.busy}
          className="button primary"
          onclick={controller.finalizeLink}
        />
      </div>
    {:else if controller.link.status === LinkStatus.AUTHENTICATOR_PRESENT}
      <p class="steamguard-note">
        {t.guard_link_existing_description()}
      </p>
      {#if !controller.transferStarted}
        <div class="steamguard-row">
          <ActionButton
            icon={ArrowRightLeft}
            label={t.guard_transfer_start_action()}
            busyLabel={t.guard_transfer_start_pending()}
            busy={controller.busyAction === GuardAction.TRANSFER_START}
            disabled={controller.busy}
            className="button primary"
            onclick={controller.startTransfer}
          />
        </div>
        <FormError message={controller.errorFor(SteamGuardField.TRANSFER)} />
      {:else}
        <div class="steamguard-row">
          <div class="steamguard-field">
            <input
              class="input"
              placeholder={t.guard_transfer_code_label()}
              aria-label={t.guard_transfer_code_label()}
              aria-invalid={Boolean(controller.errorFor(SteamGuardField.LINK_CODE))}
              aria-describedby={controller.errorFor(SteamGuardField.LINK_CODE)
                ? `${uid}-code-error`
                : undefined}
              bind:value={controller.linkCode}
              disabled={controller.busy}
              onkeydown={(event) => event.key === "Enter" && controller.finishTransfer()}
            />
            <FormError
              id={`${uid}-code-error`}
              message={controller.errorFor(SteamGuardField.LINK_CODE)}
            />
          </div>
          <ActionButton
            icon={ArrowRightLeft}
            label={t.guard_transfer_action()}
            busyLabel={t.guard_transfer_pending()}
            busy={controller.busyAction === GuardAction.TRANSFER_FINISH}
            disabled={controller.busy}
            className="button primary"
            onclick={controller.finishTransfer}
          />
        </div>
      {/if}
      <div class="steamguard-row">
        <div class="steamguard-field">
          <PasswordInput
            label={t.guard_revocation_label()}
            placeholder={t.guard_revocation_placeholder()}
            bind:value={controller.revocationInput}
            disabled={controller.busy}
            invalid={Boolean(controller.errorFor(SteamGuardField.REVOCATION))}
            describedby={controller.errorFor(SteamGuardField.REVOCATION)
              ? `${uid}-revocation-error`
              : undefined}
          />
          <FormError
            id={`${uid}-revocation-error`}
            message={controller.errorFor(SteamGuardField.REVOCATION)}
          />
        </div>
        <ActionButton
          icon={ShieldOff}
          label={t.guard_revoke_action()}
          busyLabel={t.guard_revoke_pending()}
          busy={controller.busyAction === GuardAction.REVOKE}
          disabled={controller.busy}
          className="button danger"
          onclick={controller.revokeDuringLink}
        />
      </div>
    {:else}
      <p class="steamguard-note">{linkMessage(controller.link.status)}</p>
    {/if}
    <FormError message={controller.errorFor(SteamGuardField.LINK)} />
  </section>
{/if}
