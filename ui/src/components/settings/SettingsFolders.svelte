<script lang="ts">
  import { t, rich, folderLabel } from "../../lib/i18n/i18n.svelte";

  import { FolderAction } from "./SettingsFolders.constants";
  import ActionButton from "../ui/ActionButton.svelte";
  import { folders, run, showToast } from "../../lib/state/state.svelte";
  import { askConfirm, openFolderEditor } from "../../lib/state/dialogs.svelte";
  import { ConfirmKind } from "../../lib/state/dialogs.constants";
  import { Folder, Lock, Plus, SquarePen, Trash2 } from "@lucide/svelte";
  import { folderIcon, randomFolderStyle } from "../../lib/folders/folder-style";
  import SteamGuardIcon from "../steam-guard/SteamGuardIcon.svelte";
  import type { FolderType } from "../../lib/types";

  let name = $state("");
  let creating = $state(false);
  let pending = $state<Partial<Record<string, FolderAction>>>({});
  const list = $derived(folders());

  const create = async () => {
    const folderName = name.trim();
    if (!folderName || creating) {
      return;
    }
    creating = true;
    const style = randomFolderStyle();
    const result = await run("create_folder", {
      name: folderName,
      color: style.color,
      icon: style.icon,
    });
    creating = false;
    if (result) {
      const created = folders().find((item) => item.name === folderName);
      showToast(rich(t.folder_create_success, { folder: folderName }), true, null, created ?? null);
      if (name.trim() === folderName) name = "";
    }
  };

  const remove = async (folder: FolderType): Promise<void> => {
    const { id, name } = folder;
    if (pending[id]) return;
    const answer = await askConfirm({
      title: t.folder_delete_title(),
      text: rich(t.folder_delete_confirm, { folder: name }),
      confirmLabel: t.common_delete(),
      danger: true,
      kind: ConfirmKind.REMOVE,
    });
    if (!answer) return;
    pending[id] = FolderAction.DELETE;
    const result = await run("delete_folder", { id });
    delete pending[id];
    if (result) showToast(rich(t.folder_delete_success, { folder: name }), true, null, folder);
  };
</script>

<div class="settings-cards">
  <section class="settings-card wide">
    <h3 class="settings-card-title"><Folder size={15} /> {t.settings_folders_title()}</h3>
    <p class="settings-note">
      {t.folder_settings_description()}
    </p>
    <div class="settings-list">
      {#if list.length === 0}
        <p class="empty-tab">{t.folder_empty()}</p>
      {:else}
        {#each list as folder (folder.id)}
          {@const Glyph = folderIcon(folder.icon)}
          <div class="settings-row">
            <span
              class="folder-row-icon"
              style={folder.system || !folder.color
                ? ""
                : `color:${folder.color};background:${folder.color}33`}
              aria-hidden="true"
            >
              {#if folder.system}<SteamGuardIcon size={16} />{:else}<Glyph size={16} />{/if}
            </span>
            <span class="settings-label">{folderLabel(folder)}</span>
            <span class="tag">{t.accounts_count({ count: folder.account_count })}</span>
            {#if folder.system}
              <span class="settings-note"><Lock size={14} /> {t.folder_managed()}</span>
            {:else}
              <button
                class="button"
                type="button"
                disabled={Boolean(pending[folder.id])}
                onclick={() => openFolderEditor(folder.id)}
                ><SquarePen size={15} /> {t.folder_edit_action()}</button
              >
              <ActionButton
                icon={Trash2}
                label={t.common_delete()}
                busyLabel={t.common_deleting()}
                busy={pending[folder.id] === FolderAction.DELETE}
                disabled={Boolean(pending[folder.id])}
                className="button danger"
                onclick={() => remove(folder)}
              />
            {/if}
          </div>
        {/each}
      {/if}
    </div>
    <div class="settings-add">
      <input
        class="input"
        type="text"
        placeholder={t.folder_create_placeholder()}
        bind:value={name}
      />
      <ActionButton
        icon={Plus}
        label={t.common_add()}
        busyLabel={t.common_adding()}
        busy={creating}
        disabled={!name.trim()}
        className="button primary"
        onclick={create}
      />
    </div>
  </section>
</div>
