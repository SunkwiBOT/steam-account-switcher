import { errorMessage } from "../errors/errors.svelte";
import { t, rich } from "../i18n/i18n.svelte";
import { showToast } from "./state.svelte";
import { ToastKind } from "./state.constants";
import { invoke } from "../api/tauri";
import type { ToastAccountType } from "../types";

export const copyFeedback = $state<Record<string, boolean>>({});
const timers = new Map<string, ReturnType<typeof setTimeout>>();

export const copyValue = async (
  value: string,
  key: string,
  label: string,
  account: ToastAccountType | null = null,
): Promise<boolean> => {
  try {
    await invoke("clipboard_write_text", { text: value });
    clearTimeout(timers.get(key));
    copyFeedback[key] = true;
    timers.set(
      key,
      setTimeout(() => {
        delete copyFeedback[key];
        timers.delete(key);
      }, 2000),
    );
    showToast(ToastKind.SUCCESS, t.toast_copied_title(), {
      description: rich(t.clipboard_copy_success, { value: label }),
      account,
    });
    return true;
  } catch (cause: unknown) {
    showToast(ToastKind.ERROR, t.toast_error_title(), {
      description: errorMessage(cause),
      account,
    });
    return false;
  }
};
