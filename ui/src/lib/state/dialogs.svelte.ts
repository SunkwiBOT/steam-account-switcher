import { t } from "../i18n/i18n.svelte";
import { ConfirmKind } from "./dialogs.constants";
import { errorMessage } from "../errors/errors.svelte";

import type { MessageType } from "../i18n/messages";

export type ConfirmAnswerType = {
  cleanup: boolean;
};

export type ConfirmStateType = {
  open: boolean;
  title: string;
  content: MessageType;
  checkboxLabel: string;
  checkboxRequired: boolean;
  confirmLabel: string;
  busyLabel: string;
  danger: boolean;
  steamId: string | null;
  kind: ConfirmKind;
  busy: boolean;
  error: string;
};

export const confirmDialog: ConfirmStateType = $state({
  open: false,
  title: "",
  content: "",
  checkboxLabel: "",
  checkboxRequired: false,
  confirmLabel: t.common_yes(),
  busyLabel: "",
  danger: false,
  steamId: null,
  kind: ConfirmKind.CONFIRM,
  busy: false,
  error: "",
});

let confirmAction: ((answer: ConfirmAnswerType) => Promise<boolean>) | null = null;
let confirmCancel: (() => void) | null = null;
let confirmGeneration = 0;

export const askConfirm = ({
  title,
  text,
  checkboxLabel = "",
  checkboxRequired = false,
  confirmLabel = t.common_yes(),
  busyLabel = t.common_saving(),
  danger = false,
  steamId = null,
  kind = ConfirmKind.CONFIRM,
  action,
  onCancel = null,
}: {
  title: string;
  text: MessageType;
  checkboxLabel?: string;
  checkboxRequired?: boolean;
  confirmLabel?: string;
  busyLabel?: string;
  danger?: boolean;
  steamId?: string | null;
  kind?: ConfirmStateType["kind"];
  action: (answer: ConfirmAnswerType) => Promise<boolean>;
  onCancel?: (() => void) | null;
}): void => {
  Object.assign(confirmDialog, {
    open: true,
    title,
    content: text,
    checkboxLabel,
    checkboxRequired,
    confirmLabel,
    busyLabel,
    danger,
    steamId,
    kind,
    busy: false,
    error: "",
  });
  confirmAction = action;
  confirmCancel = onCancel;
  confirmGeneration += 1;
};

export const runConfirmAction = async (answer: ConfirmAnswerType): Promise<void> => {
  if (confirmDialog.busy || !confirmAction) return;
  const action = confirmAction;
  const generation = confirmGeneration;
  confirmDialog.busy = true;
  confirmDialog.error = "";
  try {
    const ok = await action(answer);
    if (ok && generation === confirmGeneration) {
      confirmDialog.open = false;
      confirmAction = null;
    }
  } catch (cause: unknown) {
    confirmDialog.error = errorMessage(cause);
  } finally {
    if (generation === confirmGeneration) {
      confirmDialog.busy = false;
    }
  }
};

export const cancelConfirm = (): void => {
  if (confirmDialog.busy) return;
  confirmDialog.open = false;
  confirmAction = null;
  const onCancel = confirmCancel;
  confirmCancel = null;
  onCancel?.();
};

export type FolderDialogStateType = {
  open: boolean;
  title: string;
  value: string;
  confirmLabel: string;
};

export const folderDialog: FolderDialogStateType = $state({
  open: false,
  title: t.folder_create_title(),
  value: "",
  confirmLabel: t.common_create(),
});

let folderResolve: ((value: string | null) => void) | null = null;

export const askFolderName = ({
  title = t.folder_create_title(),
  value = "",
  confirmLabel = t.common_create(),
}: { title?: string; value?: string; confirmLabel?: string } = {}): Promise<string | null> => {
  Object.assign(folderDialog, { open: true, title, value, confirmLabel });
  return new Promise((resolve) => {
    folderResolve = resolve;
  });
};

export const answerFolderName = (value: string | null): void => {
  folderDialog.open = false;
  const resolve = folderResolve;
  folderResolve = null;
  resolve?.(value);
};

export const folderEditor: { open: boolean; folderId: string } = $state({
  open: false,
  folderId: "",
});

export const openFolderEditor = (folderId: string): void => {
  Object.assign(folderEditor, { open: true, folderId });
};

export const closeFolderEditor = (): void => {
  folderEditor.open = false;
  folderEditor.folderId = "";
};
