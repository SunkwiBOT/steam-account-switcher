<script lang="ts">
  import FolderEditor from "./FolderEditor.svelte";
  import { t, rich } from "../../lib/i18n/i18n.svelte";
  import { closeFolderEditor, folderEditor } from "../../lib/state/dialogs.svelte";
  import { run, showToast } from "../../lib/state/state.svelte";
  import { folders } from "../../lib/state/folders.svelte";
  import { ToastKind } from "../../lib/state/state.constants";

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
      showToast(ToastKind.SUCCESS, t.toast_folder_renamed_title(), {
        description: rich(t.folder_rename_success, { folder: value.name }),
        folder: { ...current, name: value.name },
      });
    return Boolean(result);
  };
</script>

{#if folderEditor.open && folder}
  <FolderEditor {folder} onsave={save} onclose={closeFolderEditor} />
{/if}
