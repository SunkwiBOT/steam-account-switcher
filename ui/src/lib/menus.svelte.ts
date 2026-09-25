/* Dropdown and right-click menus: position plus the pending selection. */

import { answerConfirm, answerFolderName } from "./dialogs.svelte";
import { app } from "./state.svelte";

export interface DropdownOption {
  value: string;
  label: string;
  icon?: string;
}

export interface DropdownState {
  open: boolean;
  x: number;
  y: number;
  anchorTop: number;
  options: DropdownOption[];
  value: string;
  anchor: HTMLElement | null;
}

export const dropdown: DropdownState = $state({
  open: false,
  x: 0,
  y: 0,
  anchorTop: 0,
  options: [],
  value: "",
  anchor: null,
});

let dropdownSelect: ((value: string) => void) | null = null;

export function openDropdown({
  anchor,
  options,
  value,
  onSelect,
}: {
  anchor: HTMLElement;
  options: DropdownOption[];
  value: string;
  onSelect: (value: string) => void;
}): void {
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
}

export function closeDropdown(): void {
  dropdown.open = false;
  dropdown.anchor = null;
  dropdownSelect = null;
}

export function selectDropdown(value: string): void {
  const onSelect = dropdownSelect;
  closeDropdown();
  onSelect?.(value);
}

export interface ContextMenuState {
  open: boolean;
  x: number;
  y: number;
  steamId: string;
  folderId: string;
}

export const contextMenu: ContextMenuState = $state({
  open: false,
  x: 0,
  y: 0,
  steamId: "",
  folderId: "",
});

export function openAccountMenu(event: MouseEvent, steamId: string): void {
  event.preventDefault();
  Object.assign(contextMenu, {
    open: true,
    x: event.clientX,
    y: event.clientY,
    steamId,
    folderId: "",
  });
}

export function openFolderMenu(event: MouseEvent, folderId: string): void {
  event.preventDefault();
  Object.assign(contextMenu, {
    open: true,
    x: event.clientX,
    y: event.clientY,
    steamId: "",
    folderId,
  });
}

export function closeContextMenu(): void {
  if (!contextMenu.open) {
    return;
  }
  contextMenu.open = false;
  contextMenu.steamId = "";
  contextMenu.folderId = "";
}

/** Closes the menus when the user clicks elsewhere or presses Escape. */
export function wireMenuDismissal(): void {
  document.addEventListener("click", (event) => {
    const target = event.target as HTMLElement | null;
    if (dropdown.open && !target?.closest?.(".dropdown-menu")) {
      // The control that opened the menu toggles it itself.
      if (!dropdown.anchor?.contains(target)) {
        closeDropdown();
      }
    }
    if (contextMenu.open && !target?.closest?.(".context-menu")) {
      closeContextMenu();
    }
  });

  document.addEventListener("keydown", (event) => {
    if (event.key !== "Escape") {
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
}
