<script lang="ts">
  import DialogHeader from "../dialogs/DialogHeader.svelte";
  import { focusTrap } from "../../lib/actions/focus-trap";
  import { t } from "../../lib/i18n/i18n.svelte";

  import { Lock, X } from "@lucide/svelte";
  import { app } from "../../lib/state/state.svelte";
  import VaultControls from "./VaultControls.svelte";
  import { copySteamGuardCode, removeAccount } from "../../lib/actions/actions.svelte";
  let busy = $state(false);
  const close = (): void => {
    if (!busy) {
      app.vaultOpen = false;
      app.vaultCopySteamId = null;
      app.vaultRemoveSteamId = null;
    }
  };

  const ready = async (): Promise<void> => {
    const steamId = app.vaultCopySteamId;
    const removeSteamId = app.vaultRemoveSteamId;
    app.vaultOpen = false;
    app.vaultCopySteamId = null;
    app.vaultRemoveSteamId = null;
    if (steamId) await copySteamGuardCode(steamId);
    else if (removeSteamId) await removeAccount(removeSteamId);
  };
  const uid = $props.id();
</script>

{#if app.vaultOpen}
  <div class="modal vault-modal">
    <div class="modal-backdrop"></div>
    <div
      class="dialog small"
      role="dialog"
      aria-modal="true"
      aria-labelledby={`${uid}-title`}
      tabindex="-1"
      use:focusTrap={{ onEscape: close }}
    >
      <DialogHeader
        id={`${uid}-title`}
        title={app.snapshot?.vault.locked ? t.vault_unlock_title() : t.vault_title()}
        icon={Lock}
        onclose={close}
        closeLabel={t.vault_close()}
        closeDisabled={busy}
      />
      <VaultControls onready={ready} onbusy={(value) => (busy = value)} />
      <footer class="dialog-foot">
        <button class="button" type="button" disabled={busy} onclick={close}
          ><X size={15} /> {t.common_cancel()}</button
        >
      </footer>
    </div>
  </div>
{/if}
