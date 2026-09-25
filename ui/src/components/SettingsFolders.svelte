<script lang="ts">
  import { askFolderName } from "../lib/dialogs.svelte";
  import { folders, run, showToast } from "../lib/state.svelte";
  import { Plus, SquarePen, Trash2 } from "@lucide/svelte";

  let name = $state("");
  let pending = $state("");
  const list = $derived(folders());

  async function create() {
    if (!name.trim()) {
      return;
    }
    pending = "create";
    const result = await run("create_folder", { name: name.trim() });
    pending = "";
    if (result) {
      showToast(`Folder “${name.trim()}” created.`, true);
      name = "";
    }
  }

  async function rename(id: string, current: string): Promise<void> {
    const next = await askFolderName({
      title: "Rename folder",
      value: current,
      confirmLabel: "Rename",
    });
    if (next) {
      pending = id;
      await run("rename_folder", { id, name: next });
      pending = "";
    }
  }

  async function remove(id: string): Promise<void> {
    pending = id;
    await run("delete_folder", { id }, { success: "Folder deleted." });
    pending = "";
  }
</script>

<div class="settings-cards">
  <section class="settings-card wide">
    <h3 class="settings-card-title">Folders</h3>
    <p class="settings-note">
      Folders appear as cards in the account grid. Right click an account to move it into one.
    </p>
    <div class="settings-list">
      {#if list.length === 0}
        <p class="empty-tab">No folder yet.</p>
      {:else}
        {#each list as folder (folder.id)}
          <div class="settings-row">
            <span class="settings-label">{folder.name}</span>
            <span class="tag">{folder.account_count}</span>
            <button
              class="button"
              type="button"
              disabled={pending === folder.id}
              onclick={() => rename(folder.id, folder.name)}
            >
              {#if pending === folder.id}
                <span class="spinner tiny"></span>
              {:else}
                <SquarePen size={14} /> Rename
              {/if}
            </button>
            <button
              class="button danger"
              type="button"
              disabled={pending === folder.id}
              onclick={() => remove(folder.id)}
            >
              <Trash2 size={14} /> Delete
            </button>
          </div>
        {/each}
      {/if}
    </div>
    <div class="settings-add">
      <input class="input" type="text" placeholder="New folder name" bind:value={name} />
      <button class="button primary" type="button" disabled={pending === "create"} onclick={create}>
        {#if pending === "create"}
          <span class="spinner tiny"></span>
        {:else}
          <Plus size={14} /> Add
        {/if}
      </button>
    </div>
  </section>
</div>
