<script lang="ts">
  import { answerFolderName, folderDialog } from "../lib/dialogs.svelte";

  function confirm() {
    answerFolderName(folderDialog.value.trim() || null);
  }
</script>

{#if folderDialog.open}
  <div class="modal">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="modal-backdrop" onclick={() => answerFolderName(null)}></div>
    <section class="dialog small" role="dialog" aria-modal="true">
      <h2 class="dialog-title">{folderDialog.title}</h2>
      <!-- svelte-ignore a11y_autofocus -->
      <input
        class="input full"
        type="text"
        placeholder="Folder name"
        spellcheck="false"
        autofocus
        bind:value={folderDialog.value}
        onkeydown={(event) => event.key === "Enter" && (event.preventDefault(), confirm())}
      />
      <footer class="dialog-foot">
        <button class="button" type="button" onclick={() => answerFolderName(null)}>Cancel</button>
        <button class="button primary" type="button" onclick={confirm}>
          {folderDialog.confirmLabel}
        </button>
      </footer>
    </section>
  </div>
{/if}
