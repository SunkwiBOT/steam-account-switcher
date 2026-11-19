<script lang="ts">
  import DialogHeader from "../dialogs/DialogHeader.svelte";
  import AppDialog from "../dialogs/AppDialog.svelte";
  import { DialogContent } from "$lib/components/ui/dialog/index.js";
  import ColorSwatches from "./ColorSwatches.svelte";
  import FolderIconPicker from "./FolderIconPicker.svelte";
  import ActionButton from "../ui/ActionButton.svelte";
  import UnsavedBar from "../ui/UnsavedBar.svelte";
  import { t } from "../../lib/i18n/i18n.svelte";
  import { Check, SquarePen, X } from "@lucide/svelte";
  import { folderDraftDirty } from "../../lib/folders/folder-editor";
  import { untrack } from "svelte";
  import type { FolderDto } from "../../lib/bindings/FolderDto";

  type FolderEditorValueType = { name: string; color: string | null; icon: string | null };
  type FolderEditorPropsType = {
    folder: FolderDto;
    onsave: (value: FolderEditorValueType) => Promise<boolean>;
    onclose: () => void;
  };

  let { folder, onsave, onclose }: FolderEditorPropsType = $props();
  let name = $state(untrack(() => folder.name));
  let color = $state(untrack(() => folder.color ?? ""));
  let icon = $state(untrack(() => folder.icon ?? "folder"));
  const baseline = untrack(() => ({
    name: folder.name,
    color: folder.color,
    icon: folder.icon ?? "folder",
  }));
  let saving = $state(false);
  const uid = $props.id();

  const close = (): void => {
    if (saving) return;
    onclose();
  };

  const save = async (): Promise<void> => {
    if (saving || !name.trim()) return;
    saving = true;
    try {
      if (await onsave({ name: name.trim(), color: color || null, icon })) onclose();
    } finally {
      saving = false;
    }
  };
  const dirty = $derived(folderDraftDirty({ name, color, icon }, baseline));
</script>

<AppDialog open={true} onclose={close}>
  <DialogContent showCloseButton={false} class="sm:max-w-md" aria-labelledby={`${uid}-title`}>
    <DialogHeader
      id={`${uid}-title`}
      title={t.folder_edit_title()}
      icon={SquarePen}
      onclose={close}
      closeDisabled={saving}
    />
    <div class="folder-editor-name">
      <label class="field-label" for={`${uid}-name`}>{t.folder_name()}</label>
      <input
        id={`${uid}-name`}
        class="input full"
        type="text"
        spellcheck="false"
        bind:value={name}
        disabled={saving}
        onkeydown={(event) => event.key === "Enter" && (event.preventDefault(), void save())}
      />
    </div>
    <div class="folder-editor-fields">
      <div class="folder-editor-field">
        <span class="field-label">{t.folder_colour()}</span>
        <ColorSwatches value={color} onselect={(value) => (color = value)} />
      </div>
      <div class="folder-editor-field">
        <span class="field-label">{t.folder_icon()}</span>
        <FolderIconPicker value={icon} {color} onselect={(value) => (icon = value)} />
      </div>
    </div>
    {#if !dirty}
      <footer class="dialog-foot">
        <button class="button" type="button" disabled={saving} onclick={close}
          ><X size={15} /> {t.common_cancel()}</button
        >
        <ActionButton
          icon={Check}
          label={t.common_save()}
          busyLabel={t.common_saving()}
          busy={saving}
          disabled={saving || !name.trim()}
          className="button primary"
          onclick={save}
        />
      </footer>
    {:else}
      <UnsavedBar
        type="button"
        {saving}
        oncancel={close}
        onsave={save}
        saveDisabled={saving || !name.trim()}
      />
    {/if}
  </DialogContent>
</AppDialog>
