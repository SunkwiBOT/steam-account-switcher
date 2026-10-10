<script lang="ts">
  import FormError from "../ui/FormError.svelte";
  import { GuardAction, SteamGuardField } from "../../lib/steam-guard/steam-guard.constants";
  import { t } from "../../lib/i18n/i18n.svelte";

  import { IdentityField } from "../accounts/AccountIdentity.constants";
  import ActionButton from "../ui/ActionButton.svelte";
  import type { SteamGuardControllerType } from "../../lib/steam-guard/steam-guard.svelte";
  import type { AccountType } from "../../lib/types";
  import { ShieldOff, Lock, Download } from "@lucide/svelte";
  import { app } from "../../lib/state/state.svelte";
  import { BackupMode } from "../../lib/vault/backup.constants";
  import AccountIdentity from "../accounts/AccountIdentity.svelte";
  type SteamGuardSecurityPropsType = { controller: SteamGuardControllerType; entry: AccountType };

  let { controller, entry }: SteamGuardSecurityPropsType = $props();
</script>

<section class="steamguard-card">
  <h3 class="block-title"><Lock size={15} /> {t.guard_security_title()}</h3>
  <p class="steamguard-note">
    {t.guard_security_recovery_hint()}
  </p>
  <div class="fact revocation-fact">
    <span class="fact-label">{t.guard_revocation_label()}</span>
    <AccountIdentity {entry} field={IdentityField.REVOCATION} />
  </div>
  <div class="steamguard-row">
    <button
      class="button"
      type="button"
      disabled={controller.busy}
      onclick={() => (app.backup = { mode: BackupMode.EXPORT, steamId: entry.steam_id })}
      ><Download size={15} /> {t.backup_export_account_action()}</button
    >
  </div>
  <p class="steamguard-note">
    {t.guard_security_remove_warning()}
  </p>
  <div class="steamguard-row">
    <ActionButton
      icon={ShieldOff}
      label={t.guard_remove_action()}
      busyLabel={t.common_removing()}
      busy={controller.busyAction === GuardAction.REMOVE}
      disabled={controller.busy}
      className="button danger"
      onclick={controller.removeAuthenticator}
    />
  </div>
  <FormError message={controller.errorFor(SteamGuardField.SECURITY)} />
</section>
