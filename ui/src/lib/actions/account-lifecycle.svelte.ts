import { errorMessage } from "../errors/errors.svelte";
import { t, rich } from "../i18n/i18n.svelte";
import { ConfirmKind } from "../state/dialogs.constants";

import { askConfirm, confirmDialog } from "../state/dialogs.svelte";
import { accountById, app, applySnapshot, run, showToast } from "../state/state.svelte";
import { ToastKind } from "../state/state.constants";
import { invoke } from "../api/tauri";

let switchPending = false;

export const switchAccount = (steamId: string): void => {
  if (app.busy.active || app.switchingId || switchPending || confirmDialog.open) return;
  const entry = accountById(steamId);
  if (!entry) {
    return;
  }
  if (!entry.is_local) {
    showToast(ToastKind.ERROR, t.toast_sign_in_required_title(), {
      description: t.account_switch_sign_in_required(),
      account: entry,
    });
    return;
  }
  app.openAccountId = null;

  if (app.switchingId) {
    showToast(ToastKind.ERROR, t.toast_switch_busy_title(), {
      description: t.account_switch_busy(),
    });
    return;
  }

  const steamRunning = app.snapshot?.steam_running === true;
  if (entry.is_current && steamRunning) {
    showToast(ToastKind.ERROR, t.toast_already_active_title(), {
      description: rich(t.account_switch_already_active, { account: entry.display_name }),
      account: entry,
    });
    return;
  }

  const installationRoot = app.snapshot?.installations[app.snapshot.selected]?.root;
  const switchInterrupted = (): boolean =>
    !switchPending ||
    app.busy.active ||
    app.switchingId !== "" ||
    app.snapshot?.installations[app.snapshot.selected]?.root !== installationRoot;

  const performSwitch = async (): Promise<boolean> => {
    app.switchingId = steamId;
    try {
      const result = await invoke("switch_account", { steamId });
      applySnapshot(result.snapshot);
      if (result.outcome === "ALREADY_ACTIVE") {
        showToast(ToastKind.ERROR, t.toast_already_active_title(), {
          description: rich(t.account_switch_already_active, { account: entry.display_name }),
          account: entry,
        });
      } else if (result.outcome === "STARTED") {
        showToast(ToastKind.SUCCESS, t.toast_account_started_title(), {
          description: rich(t.account_start_success, { account: entry.display_name }),
          account: entry,
        });
      } else {
        showToast(ToastKind.SUCCESS, t.toast_account_switched_title(), {
          description: rich(t.account_switch_success, { account: entry.display_name }),
          account: entry,
        });
      }
      return true;
    } catch (error: unknown) {
      showToast(ToastKind.ERROR, t.toast_error_title(), {
        description: errorMessage(error),
        account: entry,
      });
      return false;
    } finally {
      app.switchingId = "";
    }
  };

  switchPending = true;
  askConfirm({
    title: steamRunning ? t.account_switch_title() : t.account_start_action(),
    steamId,
    kind: steamRunning ? ConfirmKind.SWITCH : ConfirmKind.START,
    text: steamRunning
      ? rich(t.account_switch_confirm, { account: entry.display_name })
      : rich(t.account_start_confirm, { account: entry.display_name }),
    confirmLabel: steamRunning ? t.account_switch_action() : t.account_start_action(),
    onCancel: () => {
      switchPending = false;
    },
    action: async () => {
      if (switchInterrupted()) {
        switchPending = false;
        return true;
      }
      const games = await run("running_steam_games", undefined, {
        busy: t.account_switch_games_pending(),
        account: entry,
      });
      if (!games) {
        switchPending = false;
        return false;
      }
      if (switchInterrupted()) {
        switchPending = false;
        return true;
      }
      if (games.length) {
        askConfirm({
          title: t.account_switch_games_title({ count: games.length }),
          steamId,
          kind: ConfirmKind.RUNNING_GAME,
          text: rich(t.account_switch_games_warning, {
            games: games.join(", "),
            account: entry.display_name,
          }),
          confirmLabel: t.account_switch_games_action(),
          busyLabel: t.common_saving(),
          danger: true,
          onCancel: () => {
            switchPending = false;
          },
          action: async () => {
            if (switchInterrupted()) {
              switchPending = false;
              return true;
            }
            const ok = await performSwitch();
            switchPending = false;
            return ok;
          },
        });
        return true;
      }
      const ok = await performSwitch();
      switchPending = false;
      return ok;
    },
  });
};

export const removeAccount = (steamId: string): void => {
  if (app.busy.active || app.switchingId || switchPending || confirmDialog.open) return;
  const entry = accountById(steamId);
  if (!entry) {
    return;
  }
  app.openAccountId = null;

  if (!entry.is_local) {
    if (app.snapshot?.vault.locked) {
      app.vaultCopySteamId = null;
      app.vaultRemoveSteamId = steamId;
      app.vaultOpen = true;
      return;
    }
    askConfirm({
      title: t.guard_saved_delete(),
      kind: ConfirmKind.REMOVE,
      steamId,
      text: rich(t.guard_saved_delete_warning, {
        account: entry.display_name,
      }),
      checkboxLabel: t.guard_saved_delete_acknowledgement(),
      checkboxRequired: true,
      confirmLabel: t.guard_saved_delete(),
      busyLabel: t.common_removing(),
      danger: true,
      action: async (answer) => {
        if (!answer.cleanup) return false;
        const result = await run(
          "forget_saved_account",
          { steamId },
          {
            busy: t.guard_saved_delete_pending(),
            successTitle: t.toast_guard_saved_removed_title(),
            success: rich(t.guard_saved_delete_success, {
              account: entry.display_name,
            }),
            account: entry,
          },
        );
        return result !== null;
      },
    });
    return;
  }

  askConfirm({
    title: t.account_remove_title(),
    kind: ConfirmKind.REMOVE,
    steamId,
    text: entry.has_authenticator
      ? rich(t.account_remove_guard_confirm, {
          account: entry.display_name,
        })
      : rich(t.account_remove_confirm, {
          account: entry.display_name,
        }),
    checkboxLabel: t.account_remove_data_checkbox(),
    confirmLabel: t.common_remove(),
    busyLabel: t.common_removing(),
    danger: true,
    action: async (answer) => {
      const result = await run(
        "remove_account",
        { steamId, cleanupUserdata: Boolean(answer.cleanup) },
        {
          busy: t.account_remove_pending(),
          successTitle: t.toast_account_removed_title(),
          success: entry.has_authenticator
            ? rich(t.account_remove_guard_success, {
                account: entry.display_name,
              })
            : rich(t.account_remove_success, { account: entry.display_name }),
          account: entry,
        },
      );
      return result !== null;
    },
  });
};

export const addAccount = (): void => {
  if (app.busy.active || app.switchingId || switchPending || confirmDialog.open) return;
  askConfirm({
    title: t.account_add_title(),
    kind: ConfirmKind.ADD,
    text: t.account_add_description(),
    confirmLabel: t.account_add_action(),
    action: async () => {
      const result = await run("add_steam_account", undefined, {
        busy: t.account_add_pending(),
        successTitle: t.account_add_success(),
      });
      return result !== null;
    },
  });
};
