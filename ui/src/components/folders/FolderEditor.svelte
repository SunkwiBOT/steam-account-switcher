<script lang="ts">
  import DialogHeader from "../dialogs/DialogHeader.svelte";
  import ColorSwatches from "./ColorSwatches.svelte";
  import FolderIconPicker from "./FolderIconPicker.svelte";
  import ActionButton from "../ui/ActionButton.svelte";
  import { t } from "../../lib/i18n/i18n.svelte";
  import { Check, SquarePen, X } from "@lucide/svelte";
  import { untrack } from "svelte";
  import type { FolderType } from "../../lib/types";

  type FolderEditorValueType = { name: string; color: string | null; icon: string | null };
  type FolderEditorPropsType = {
    folder: FolderType;
    onsave: (value: FolderEditorValueType) => Promise<boolean>;
    onclose: () => void;
  };

  let { folder, onsave, onclose }: FolderEditorPropsType = $props();
  let dialog = $state<HTMLDialogElement>();
  let name = $state(untrack(() => folder.name));
  let color = $state(untrack(() => folder.color ?? ""));
  let icon = $state(untrack(() => folder.icon ?? "folder"));
  let saving = $state(false);
  const uid = $props.id();

  $effect(() => {
    if (dialog && !dialog.open) {
      dialog.showModal();
      dialog.querySelector("input")?.focus();
    }
  });

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
</script>

<dialog
  class="dialog small folder-editor"
  bind:this={dialog}
  aria-labelledby={`${uid}-title`}
  aria-modal="true"
  oncancel={(event) => {
    event.preventDefault();
    close();
  }}
  onkeydown={(event) => {
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      close();
    }
  }}
>
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
</dialog>
