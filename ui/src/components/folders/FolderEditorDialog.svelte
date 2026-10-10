<script lang="ts">
  import FolderEditor from "./FolderEditor.svelte";
  import { t, rich } from "../../lib/i18n/i18n.svelte";
  import { closeFolderEditor, folderEditor } from "../../lib/state/dialogs.svelte";
  import { folders, run, showToast } from "../../lib/state/state.svelte";

  const folder = $derived(folders().find((item) => item.id === folderEditor.folderId) ?? null);

  const save = async (value: {
    name: string;
    color: string | null;
    icon: string | null;
  }): Promise<boolean> => {
    const current = folder;
    if (!current) return false;
    const renamed = value.name !== current.name;
    const result = await run("update_folder", {
      id: current.id,
      name: value.name,
      color: value.color,
      icon: value.icon,
    });
    if (result && renamed)
      showToast(rich(t.folder_rename_success, { folder: value.name }), true, null, {
        ...current,
        name: value.name,
      });
    return Boolean(result);
  };
</script>

{#if folderEditor.open && folder}
  <FolderEditor {folder} onsave={save} onclose={closeFolderEditor} />
{/if}
