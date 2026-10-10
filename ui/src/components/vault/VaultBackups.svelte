<script lang="ts">
  import { Download, Upload } from "@lucide/svelte";
  import { app } from "../../lib/state/state.svelte";
  import { BackupMode, BACKUP_EXTENSION } from "../../lib/vault/backup.constants";
  import { t, rich } from "../../lib/i18n/i18n.svelte";
  import MessageText from "../ui/MessageText.svelte";
</script>

<section class="settings-card">
  <h3 class="settings-card-title"><Download size={15} /> {t.backup_settings_title()}</h3>
  <p class="settings-note">
    <MessageText
      content={rich(t.backup_settings_description, { format: `.${BACKUP_EXTENSION}` }, ["format"])}
    />
  </p>
  <div class="settings-actions">
    <button
      class="button"
      type="button"
      disabled={app.snapshot?.vault.count === 0 || Boolean(app.snapshot?.vault.error)}
      onclick={() => (app.backup = { mode: BackupMode.EXPORT, steamId: null })}
      ><Download size={15} /> {t.backup_export_all_action()}</button
    >
    <button
      class="button"
      type="button"
      disabled={Boolean(app.snapshot?.vault.error)}
      onclick={() => (app.backup = { mode: BackupMode.IMPORT, steamId: null })}
      ><Upload size={15} /> {t.backup_import_action()}</button
    >
  </div>
</section>
