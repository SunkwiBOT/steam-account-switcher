import { errorMessage } from "../errors/errors.svelte";
import { t } from "../i18n/i18n.svelte";
import { copyValue } from "../state/clipboard.svelte";
import { accountById, app, showToast } from "../state/state.svelte";
import { ToastKind } from "../state/state.constants";
import { invoke } from "../api/tauri";

export const copySteamId = async (steamId: string): Promise<void> => {
  try {
    await copyValue(steamId, `id:${steamId}:Steam ID 64`, t.steam_ids_64(), accountById(steamId));
  } catch (e: unknown) {
    showToast(ToastKind.ERROR, t.toast_copy_error_title(), {
      description: t.steam_ids_copy_error(),
      account: accountById(steamId),
    });
  }
};

export const copySteamGuardCode = async (steamId: string): Promise<boolean> => {
  if (app.busy.active || app.switchingId) return false;
  const entry = accountById(steamId);
  if (app.snapshot?.vault.locked) {
    app.vaultRemoveSteamId = null;
    app.vaultCopySteamId = steamId;
    app.vaultOpen = true;
    return false;
  }
  try {
    const { code } = await invoke("steam_guard_code", { steamId });
    return await copyValue(code, `guard:${steamId}`, t.guard_code_label(), entry);
  } catch (cause: unknown) {
    showToast(ToastKind.ERROR, t.toast_error_title(), {
      description: errorMessage(cause),
      account: entry,
    });
    return false;
  }
};
