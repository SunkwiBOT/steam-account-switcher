import { errorMessage } from "../errors/errors.svelte";
import { t } from "../i18n/i18n.svelte";
import { app, showToast } from "../state/state.svelte";
import { ToastKind } from "../state/state.constants";
import { invoke } from "../api/tauri";

export const repositoryUrl = (): string => app.info.repository;

export const openRepository = async (): Promise<void> => {
  try {
    await invoke("open_repository");
  } catch (error: unknown) {
    showToast(ToastKind.ERROR, t.toast_error_title(), { description: errorMessage(error) });
  }
};
