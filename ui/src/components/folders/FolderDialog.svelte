<script lang="ts">
  import DialogHeader from "../dialogs/DialogHeader.svelte";
  import { focusTrap } from "../../lib/actions/focus-trap";
  import { t } from "../../lib/i18n/i18n.svelte";

  import { answerFolderName, folderDialog } from "../../lib/state/dialogs.svelte";

  import { Check, Folder, X } from "@lucide/svelte";

  const confirm = () => {
    answerFolderName(folderDialog.value.trim() || null);
  };
  const uid = $props.id();
</script>

{#if folderDialog.open}
  <div class="modal">
    <div class="modal-backdrop"></div>
    <div
      class="dialog small"
      role="dialog"
      aria-modal="true"
      aria-labelledby={`${uid}-title`}
      tabindex="-1"
      use:focusTrap={{ onEscape: () => answerFolderName(null) }}
    >
      <DialogHeader id={`${uid}-title`} title={folderDialog.title} icon={Folder} />
      <!-- svelte-ignore a11y_autofocus -->
      <input
        class="input full"
        type="text"
        placeholder={t.folder_name()}
        spellcheck="false"
        autofocus
        bind:value={folderDialog.value}
        onkeydown={(event) => event.key === "Enter" && (event.preventDefault(), confirm())}
      />
      <footer class="dialog-foot">
        <button class="button" type="button" onclick={() => answerFolderName(null)}
          ><X size={15} /> {t.common_cancel()}</button
        >
        <button class="button primary" type="button" onclick={confirm}>
          <Check size={15} />
          {folderDialog.confirmLabel}
        </button>
      </footer>
    </div>
  </div>
{/if}
