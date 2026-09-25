/* Promise-based confirm and folder-name dialogs. */

export interface ConfirmState {
  open: boolean;
  title: string;
  html: string;
  checkboxLabel: string;
  confirmLabel: string;
  danger: boolean;
}

export const confirmDialog: ConfirmState = $state({
  open: false,
  title: "",
  html: "",
  checkboxLabel: "",
  confirmLabel: "Yes",
  danger: false,
});

export interface ConfirmAnswer {
  cleanup: boolean;
}

let confirmResolve: ((value: ConfirmAnswer | null) => void) | null = null;

/** `text` is markup: build it with strong() so values stay escaped. */
export function askConfirm({
  title,
  text,
  checkboxLabel = "",
  confirmLabel = "Yes",
  danger = false,
}: {
  title: string;
  text: string;
  checkboxLabel?: string;
  confirmLabel?: string;
  danger?: boolean;
}): Promise<ConfirmAnswer | null> {
  Object.assign(confirmDialog, {
    open: true,
    title,
    html: text,
    checkboxLabel,
    confirmLabel,
    danger,
  });
  return new Promise((resolve) => {
    confirmResolve = resolve;
  });
}

export function answerConfirm(value: ConfirmAnswer | null): void {
  confirmDialog.open = false;
  const resolve = confirmResolve;
  confirmResolve = null;
  resolve?.(value);
}

export interface FolderDialogState {
  open: boolean;
  title: string;
  value: string;
  confirmLabel: string;
}

export const folderDialog: FolderDialogState = $state({
  open: false,
  title: "New folder",
  value: "",
  confirmLabel: "Create",
});

let folderResolve: ((value: string | null) => void) | null = null;

export function askFolderName({
  title = "New folder",
  value = "",
  confirmLabel = "Create",
}: { title?: string; value?: string; confirmLabel?: string } = {}): Promise<string | null> {
  Object.assign(folderDialog, { open: true, title, value, confirmLabel });
  return new Promise((resolve) => {
    folderResolve = resolve;
  });
}

export function answerFolderName(value: string | null): void {
  folderDialog.open = false;
  const resolve = folderResolve;
  folderResolve = null;
  resolve?.(value);
}
