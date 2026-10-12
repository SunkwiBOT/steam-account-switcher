import { cycleFocusIndex } from "../state/keyboard";

/**
 * `onEscape` runs when Escape is pressed while this dialog is the top-most
 * one; each dialog applies its own "busy" rule inside it.
 */
export type FocusTrapOptionsType = {
  onEscape?: () => void;
};

type TrappedDialogType = {
  element: HTMLElement;
  options: FocusTrapOptionsType;
};

const FOCUSABLE_SELECTOR = [
  "a[href]",
  "button:not(:disabled)",
  "input:not(:disabled)",
  "select:not(:disabled)",
  "textarea:not(:disabled)",
  "summary",
  '[tabindex]:not([tabindex="-1"]):not(:disabled)',
].join(", ");

/**
 * Dialogs in mount order: the last one opened is the one on top, which is also
 * the one Escape must close. Native `<dialog>` elements handle Escape
 * themselves and never register here.
 */
const stack: TrappedDialogType[] = [];

const isVisible = (element: HTMLElement): boolean =>
  element.getClientRects().length > 0 && !element.closest("[inert]");

/** Focusable descendants of `root`, in document order, skipping hidden ones. */
export const focusableWithin = (root: HTMLElement): HTMLElement[] =>
  [...root.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR)].filter(isVisible);

const initialTarget = (root: HTMLElement): HTMLElement => {
  const preferred = root.querySelector<HTMLElement>("[autofocus]");
  if (preferred && isVisible(preferred) && !preferred.matches(":disabled")) return preferred;
  return focusableWithin(root)[0] ?? root;
};

export const openDialogCount = (): number => stack.length;

export const closeTopDialog = (): boolean => {
  const top = stack[stack.length - 1];
  if (!top) return false;
  top.options.onEscape?.();
  return true;
};

/**
 * Initial focus goes to the `[autofocus]` element, else the first focusable
 * one, else the container; focus returns to the opening control on unmount.
 */
export const focusTrap = (
  node: HTMLElement,
  options: FocusTrapOptionsType = {},
): { update: (next: FocusTrapOptionsType) => void; destroy: () => void } => {
  const entry: TrappedDialogType = { element: node, options };
  stack.push(entry);

  const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  initialTarget(node).focus();

  const onKeydown = (event: KeyboardEvent): void => {
    if (event.key !== "Tab") return;
    const target = event.target instanceof Element ? event.target : null;
    //! A native modal <dialog> nested inside already contains focus on its own.
    if (target?.closest("dialog[open]")) return;
    const focusable = focusableWithin(node);
    const next = cycleFocusIndex(
      focusable.findIndex((item) => item === document.activeElement),
      focusable.length,
      event.shiftKey,
    );
    event.preventDefault();
    if (next >= 0) focusable[next]?.focus();
    else node.focus();
  };
  node.addEventListener("keydown", onKeydown);

  return {
    update(next: FocusTrapOptionsType) {
      entry.options = next;
    },
    destroy() {
      node.removeEventListener("keydown", onKeydown);
      const index = stack.indexOf(entry);
      if (index >= 0) stack.splice(index, 1);
      if (previous?.isConnected && !previous.closest("[inert]")) previous.focus();
    },
  };
};
