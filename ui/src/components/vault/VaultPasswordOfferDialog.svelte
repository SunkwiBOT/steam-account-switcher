<script lang="ts">
  import DialogHeader from "../dialogs/DialogHeader.svelte";
  import AppDialog from "../dialogs/AppDialog.svelte";
  import { DialogContent } from "$lib/components/ui/dialog/index.js";
  import { t } from "../../lib/i18n/i18n.svelte";
  import { KeyRound, X } from "@lucide/svelte";
  import { app, run } from "../../lib/state/state.svelte";
  import VaultControls from "./VaultControls.svelte";

  let busy = $state(false);

  const dismiss = (): void => {
    if (busy) return;
    app.vaultOfferOpen = false;
  };

  const skip = async (): Promise<void> => {
    if (busy) return;
    dismiss();
    await run("set_vault_offer_dismissed");
  };

  const uid = $props.id();
</script>

<AppDialog open={app.vaultOfferOpen} onclose={dismiss}>
  <DialogContent showCloseButton={false} class="sm:max-w-md" aria-labelledby={`${uid}-title`}>
    <DialogHeader
      id={`${uid}-title`}
      title={t.vault_offer_title()}
      icon={KeyRound}
      onclose={dismiss}
      closeDisabled={busy}
    />
    <p class="dialog-text">{t.vault_offer_description()}</p>
    <VaultControls onready={dismiss} onbusy={(value) => (busy = value)} />
    <p class="steamguard-note">{t.vault_offer_settings_hint()}</p>
    <footer class="dialog-foot">
      <button class="button" type="button" disabled={busy} onclick={skip}
        ><X size={15} /> {t.vault_offer_skip()}</button
      >
    </footer>
  </DialogContent>
</AppDialog>
