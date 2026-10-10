<script lang="ts">
  import DialogHeader from "../dialogs/DialogHeader.svelte";
  import { errorMessage } from "../../lib/errors/errors.svelte";
  import { t, rich } from "../../lib/i18n/i18n.svelte";

  import {
    GuardAction,
    SteamGuardScreen,
    SteamGuardField,
  } from "../../lib/steam-guard/steam-guard.constants";
  import { Settings, X, KeyRound, Upload, Smartphone, Lock } from "@lucide/svelte";
  import { app } from "../../lib/state/state.svelte";
  import { createSteamGuardController } from "../../lib/steam-guard/steam-guard.svelte";
  import AsyncIcon from "../ui/AsyncIcon.svelte";
  import MessageText from "../ui/MessageText.svelte";
  import FormError from "../ui/FormError.svelte";
  import SteamGuardCode from "./SteamGuardCode.svelte";
  import SteamGuardIcon from "./SteamGuardIcon.svelte";
  import VaultControls from "../vault/VaultControls.svelte";
  import SteamGuardConfirmations from "./SteamGuardConfirmations.svelte";
  import SteamGuardSecurity from "./SteamGuardSecurity.svelte";
  import SteamGuardImport from "./SteamGuardImport.svelte";
  import SteamGuardLogin from "./SteamGuardLogin.svelte";
  import SteamGuardLink from "./SteamGuardLink.svelte";

  const controller = createSteamGuardController();
  const entry = $derived(controller.entry);
  const uid = $props.id();
</script>

{#if app.steamGuardOpen && entry}
  <div class="modal" inert={app.vaultOpen}>
    <div class="modal-backdrop"></div>
    <div
      class="dialog steamguard-dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby={`${uid}-title`}
      tabindex="-1"
    >
      <DialogHeader
        id={`${uid}-title`}
        title={t.guard_title()}
        icon={SteamGuardIcon}
        onclose={controller.close}
        closeDisabled={controller.busy && !controller.waiting}
      />

      <div class="dialog-body">
        {#if controller.vault.error}
          <p class="steamguard-note">
            <MessageText
              content={rich(t.guard_vault_unavailable_description, { account: entry.display_name })}
              account={entry}
            />
          </p>
          <p class="steamguard-error">
            {errorMessage(controller.vault.error)}
            {t.vault_restore_hint()}
          </p>
        {:else if controller.vault.locked}
          <section class="steamguard-card">
            <h3 class="block-title">
              <Lock size={15} />
              {t.vault_unlock_title()}
            </h3>
            <p class="steamguard-note">
              <MessageText
                content={rich(t.guard_vault_unlock_description, { account: entry.display_name })}
                account={entry}
              />
            </p>
            <VaultControls onbusy={(value) => (controller.busy = value)} />
          </section>
        {:else if controller.screen === SteamGuardScreen.HOME}
          {#if entry.has_authenticator}
            <section class="steamguard-card">
              <h3 class="block-title"><KeyRound size={15} /> {t.guard_code_label()}</h3>
              <p class="steamguard-note">
                <MessageText
                  content={rich(t.guard_code_description, { account: entry.display_name })}
                  account={entry}
                />
              </p>
              <SteamGuardCode steamId={entry.steam_id} />
            </section>

            <SteamGuardConfirmations {controller} {entry} />
            <SteamGuardSecurity {controller} {entry} />
          {:else}
            <section class="steamguard-card">
              <h3 class="block-title"><SteamGuardIcon size={17} /> {t.guard_empty_title()}</h3>
              <p class="steamguard-note">
                <MessageText
                  content={rich(t.guard_empty_description, { account: entry.display_name })}
                  account={entry}
                />
              </p>
              <div class="steamguard-actions">
                <button
                  class="button primary"
                  type="button"
                  disabled={controller.busy || !controller.vaultReady}
                  onclick={controller.beginLink}
                >
                  <Smartphone size={15} />
                  {t.guard_add_action()}
                </button>
                <button
                  class="button"
                  type="button"
                  disabled={controller.busy || !controller.vaultReady}
                  onclick={() => (controller.screen = SteamGuardScreen.IMPORT)}
                >
                  {#if controller.busyAction === GuardAction.IMPORT}<AsyncIcon
                      size={15}
                    />{:else}<Upload size={15} />{/if}
                  <MessageText
                    content={rich(t.guard_import_action, { format: ".maFile" }, ["format"])}
                  />
                </button>
              </div>
            </section>
          {/if}
        {:else if controller.screen === SteamGuardScreen.IMPORT}
          <SteamGuardImport {controller} {entry} />
        {:else if controller.screen === SteamGuardScreen.LOGIN}
          <SteamGuardLogin {controller} {entry} />
        {:else if controller.screen === SteamGuardScreen.LINK && controller.link}
          <SteamGuardLink {controller} {entry} />
        {/if}
        <FormError message={controller.errorFor(SteamGuardField.GENERAL)} />
      </div>

      <footer class="dialog-foot">
        <button
          class="button"
          type="button"
          disabled={controller.busy || controller.waiting}
          onclick={controller.openVaultSettings}
          ><Settings size={15} /> {t.vault_settings_action()}</button
        >
        <button
          class="button"
          type="button"
          disabled={controller.busy && !controller.waiting}
          onclick={controller.close}><X size={15} /> {t.common_close()}</button
        >
      </footer>
    </div>
  </div>
{/if}
