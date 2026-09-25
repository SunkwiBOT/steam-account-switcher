<script lang="ts">
  import { answerConfirm, confirmDialog } from "../lib/dialogs.svelte";

  let checked = $state(false);

  $effect(() => {
    if (confirmDialog.open) {
      checked = false;
    }
  });
</script>

{#if confirmDialog.open}
  <div class="modal">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="modal-backdrop" onclick={() => answerConfirm(null)}></div>
    <section class="dialog small" role="dialog" aria-modal="true">
      <h2 class="dialog-title">{confirmDialog.title}</h2>
      <p class="dialog-text">{@html confirmDialog.html}</p>
      {#if confirmDialog.checkboxLabel}
        <label class="checkbox">
          <input type="checkbox" bind:checked />
          <span>{confirmDialog.checkboxLabel}</span>
        </label>
      {/if}
      <footer class="dialog-foot">
        <button class="button" type="button" onclick={() => answerConfirm(null)}>Cancel</button>
        <button
          class="button"
          class:primary={!confirmDialog.danger}
          class:danger={confirmDialog.danger}
          type="button"
          onclick={() => answerConfirm({ cleanup: checked })}
        >
          {confirmDialog.confirmLabel}
        </button>
      </footer>
    </section>
  </div>
{/if}
