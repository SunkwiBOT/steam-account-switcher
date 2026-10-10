import { t } from "../i18n/i18n.svelte";
import { ConfirmKind } from "./dialogs.constants";

import type { MessageType } from "../i18n/messages";

export type ConfirmStateType = {
  open: boolean;
  title: string;
  content: MessageType;
  checkboxLabel: string;
  checkboxRequired: boolean;
  confirmLabel: string;
  danger: boolean;
  steamId: string | null;
  kind: ConfirmKind;
};

export const confirmDialog: ConfirmStateType = $state({
  open: false,
  title: "",
  content: "",
  checkboxLabel: "",
  checkboxRequired: false,
  confirmLabel: t.common_yes(),
  danger: false,
  steamId: null,
  kind: ConfirmKind.CONFIRM,
});

export type ConfirmAnswerType = {
  cleanup: boolean;
};

let confirmResolve: ((value: ConfirmAnswerType | null) => void) | null = null;

export const askConfirm = ({
  title,
  text,
  checkboxLabel = "",
  checkboxRequired = false,
  confirmLabel = t.common_yes(),
  danger = false,
  steamId = null,
  kind = ConfirmKind.CONFIRM,
}: {
  title: string;
  text: MessageType;
  checkboxLabel?: string;
  checkboxRequired?: boolean;
  confirmLabel?: string;
  danger?: boolean;
  steamId?: string | null;
  kind?: ConfirmStateType["kind"];
}): Promise<ConfirmAnswerType | null> => {
  Object.assign(confirmDialog, {
    open: true,
    title,
    content: text,
    checkboxLabel,
    checkboxRequired,
    confirmLabel,
    danger,
    steamId,
    kind,
  });
  return new Promise((resolve) => {
    confirmResolve = resolve;
  });
};

export const answerConfirm = (value: ConfirmAnswerType | null): void => {
  confirmDialog.open = false;
  const resolve = confirmResolve;
  confirmResolve = null;
  resolve?.(value);
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
