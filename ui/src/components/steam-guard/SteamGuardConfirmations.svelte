<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";

  import ActionButton from "../ui/ActionButton.svelte";
  import FormError from "../ui/FormError.svelte";
  import { SteamGuardField } from "../../lib/steam-guard/steam-guard.constants";
  import type { SteamGuardControllerType } from "../../lib/steam-guard/steam-guard.svelte";
  import type { AccountType } from "../../lib/types";
  import { ChevronDown, ListChecks, LogIn, RefreshCw, Check, X } from "@lucide/svelte";
  type SteamGuardConfirmationsPropsType = {
    controller: SteamGuardControllerType;
    entry: AccountType;
  };

  const confirmationLabel = (kind: string): string =>
    (
      ({
        trade: t.guard_confirmation_kind_trade,
        market: t.guard_confirmation_kind_market,
        phone: t.guard_confirmation_kind_phone,
        recovery: t.guard_confirmation_kind_recovery,
        api_key: t.guard_confirmation_kind_api_key,
        feature: t.guard_confirmation_kind_opt_out,
        family: t.guard_confirmation_kind_family,
        test: t.guard_confirmation_kind_test,
      })[kind] ?? t.guard_confirmation_kind_unknown
    )();
  let { controller, entry }: SteamGuardConfirmationsPropsType = $props();
</script>

<section class="steamguard-card">
  <div class="steamguard-card-head">
    <h3 class="block-title"><ListChecks size={15} /> {t.guard_confirmations_title()}</h3>
    <ActionButton
      icon={RefreshCw}
      label={t.common_refresh()}
      busyLabel={t.common_refreshing()}
      busy={controller.confirmationsBusy}
      disabled={controller.busy || !entry.has_authenticator_session}
      className="button"
      onclick={controller.loadConfirmations}
    />
  </div>
  {#if entry.has_authenticator_session}
    <details class="confirmation-session">
      <summary
        ><LogIn size={15} />
        {t.guard_confirmations_session_label()}
        <ChevronDown size={15} /></summary
      >
      <p class="steamguard-note">
        {t.guard_confirmations_session_description()}
      </p>
      <button
        class="button"
        type="button"
        disabled={controller.busy || controller.confirmationsBusy}
        onclick={controller.beginSignIn}
        ><LogIn size={15} /> {t.guard_confirmations_session_renew()}</button
      >
    </details>
  {:else}
    <button
      class="button"
      type="button"
      disabled={controller.busy || controller.confirmationsBusy}
      onclick={controller.beginSignIn}><LogIn size={15} /> {t.guard_confirmations_sign_in()}</button
    >
  {/if}
  <FormError message={controller.confirmationsError} />
  {#if !entry.has_authenticator_session}
    <p class="steamguard-note">{t.guard_confirmations_sign_in_hint()}</p>
  {:else if !controller.confirmationsLoaded}
    <p class="steamguard-note">
      {controller.confirmationsBusy
        ? t.guard_confirmations_loading()
        : !controller.confirmationsAttempted
          ? t.guard_confirmations_inactive()
          : t.guard_confirmations_load_error()}
    </p>
  {:else if controller.confirmations.length === 0}
    <p class="steamguard-note">{t.guard_confirmations_empty()}</p>
  {:else}
    <ul class="confirmation-list">
      {#each controller.confirmations as item (item.id)}
        <li class="confirmation-item">
          <div class="confirmation-text">
            <span class="confirmation-kind">{confirmationLabel(item.kind)}</span>
            <span class="confirmation-headline">{item.headline}</span>
            {#if item.summary}<span class="confirmation-summary">{item.summary}</span>{/if}
          </div>
          <div class="confirmation-actions">
            <ActionButton
              icon={X}
              label={t.common_refuse()}
              busyLabel={t.common_refusing()}
              busy={controller.busyAction === `confirmation:${item.id}:false`}
              disabled={controller.busy}
              className="button danger"
              onclick={() => controller.actOnConfirmation(item, false)}
            />
            <ActionButton
              icon={Check}
              label={t.common_accept()}
              busyLabel={t.common_accepting()}
              busy={controller.busyAction === `confirmation:${item.id}:true`}
              disabled={controller.busy}
              className="button primary"
              onclick={() => controller.actOnConfirmation(item, true)}
            />
          </div>
        </li>
      {/each}
    </ul>
  {/if}
  <FormError message={controller.errorFor(SteamGuardField.CONFIRMATIONS)} />
</section>
