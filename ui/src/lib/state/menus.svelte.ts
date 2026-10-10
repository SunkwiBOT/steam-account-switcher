import { answerConfirm, answerFolderName } from "./dialogs.svelte";
import { app } from "./state.svelte";

export type DropdownOptionType = {
  value: string;
  label: string;
  icon?: string;
};

export type DropdownStateType = {
  open: boolean;
  x: number;
  y: number;
  anchorTop: number;
  options: DropdownOptionType[];
  value: string;
  anchor: HTMLElement | null;
};

export const dropdown: DropdownStateType = $state({
  open: false,
  x: 0,
  y: 0,
  anchorTop: 0,
  options: [],
  value: "",
  anchor: null,
});

let dropdownSelect: ((value: string) => void) | null = null;

export const openDropdown = ({
  anchor,
  options,
  value,
  onSelect,
}: {
  anchor: HTMLElement;
  options: DropdownOptionType[];
  value: string;
  onSelect: (value: string) => void;
}): void => {
  dropdownSelect = onSelect;
  const bounds = anchor.getBoundingClientRect();
  Object.assign(dropdown, {
    open: true,
    anchor,
    options,
    value,
    x: bounds.left,
    y: bounds.bottom + 4,
    anchorTop: bounds.top,
  });
};

export const closeDropdown = (): void => {
  dropdown.open = false;
  dropdown.anchor = null;
  dropdownSelect = null;
};

export const selectDropdown = (value: string): void => {
  const onSelect = dropdownSelect;
  closeDropdown();
  onSelect?.(value);
};

export type ContextMenuStateType = {
  open: boolean;
  x: number;
  y: number;
  steamId: string;
  folderId: string;
};

export const contextMenu: ContextMenuStateType = $state({
  open: false,
  x: 0,
  y: 0,
  steamId: "",
  folderId: "",
});

export const openAccountMenu = (event: MouseEvent, steamId: string): void => {
  event.preventDefault();
  Object.assign(contextMenu, {
    open: true,
    x: event.clientX,
    y: event.clientY,
    steamId,
    folderId: "",
  });
};

export const openFolderMenu = (event: MouseEvent, folderId: string): void => {
  event.preventDefault();
  Object.assign(contextMenu, {
    open: true,
    x: event.clientX,
    y: event.clientY,
    steamId: "",
    folderId,
  });
};

export const closeContextMenu = (): void => {
  if (!contextMenu.open) {
    return;
  }
  contextMenu.open = false;
  contextMenu.steamId = "";
  contextMenu.folderId = "";
};

export const wireMenuDismissal = (): void => {
  // Close on press so a control that stops its click event cannot leave the menu open.
  document.addEventListener("pointerdown", (event) => {
    const target = event.target instanceof Element ? event.target : null;
    if (contextMenu.open && !target?.closest(".context-menu")) {
      closeContextMenu();
    }
  });

  document.addEventListener("click", (event) => {
    const target = event.target instanceof Element ? event.target : null;
    if (dropdown.open && !target?.closest(".dropdown-menu")) {
      // The control that opened the menu toggles it itself.
      if (!dropdown.anchor?.contains(target)) {
        closeDropdown();
      }
    }
  });

  document.addEventListener("keydown", (event) => {
    if (event.key !== "Escape") {
      return;
    }
    if (app.busy.active || app.switchingId) {
      event.preventDefault();
      return;
    }
    closeDropdown();
    closeContextMenu();
    answerConfirm(null);
    answerFolderName(null);
    app.openAccountId = null;
    app.settingsOpen = false;
    app.updateDialog = false;
    app.report = null;
  });

  window.addEventListener("blur", () => {
    closeDropdown();
    closeContextMenu();
  });
};
