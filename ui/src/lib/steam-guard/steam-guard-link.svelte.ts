import { t, rich } from "../i18n/i18n.svelte";
import { GuardAction, SteamGuardScreen, SteamGuardField } from "./steam-guard.constants";
import type { GuardSessionType } from "./steam-guard-session.svelte";
import { askConfirm } from "../state/dialogs.svelte";
import { showToast } from "../state/state.svelte";
import { ToastKind } from "../state/state.constants";
import { invoke } from "../api/tauri";
import type { AccountDto } from "../bindings/AccountDto";
import type { LinkStart } from "../bindings/LinkStart";

export type LinkDepsType = {
  session: GuardSessionType;
  entry: () => AccountDto | null;
};

export const createLinkController = (deps: LinkDepsType) => {
  const { session } = deps;

  let link = $state<LinkStart | null>(null);
  let linkCode = $state("");
  let transferStarted = $state(false);
  let revocationInput = $state("");

  const reset = (): void => {
    link = null;
    linkCode = "";
    transferStarted = false;
    revocationInput = "";
  };

  const start = async (): Promise<void> => {
    const entry = deps.entry();
    if (!entry) {
      return;
    }
    const started = await session.guard(
      () => invoke("steam_guard_link_start", { steamId: entry.steam_id }),
      GuardAction.LINK,
    );
    if (started) {
      link = started;
      session.setScreen(SteamGuardScreen.LINK);
      linkCode = "";
      transferStarted = false;
      revocationInput = "";
    }
  };

  const finalizeLink = async (): Promise<void> => {
    const current = link;
    if (!current) {
      return;
    }
    const result = await session.guard(
      () =>
        invoke("steam_guard_link_finalize", {
          sessionId: current.session_id,
          confirmCode: linkCode,
        }),
      GuardAction.FINALIZE,
    );
    if (result) {
      session.applyResult(result);
      link = null;
      session.setScreen(SteamGuardScreen.HOME);
      showToast(ToastKind.SUCCESS, t.guard_link_success(), { account: deps.entry() });
    }
  };

  const startTransfer = async (): Promise<void> => {
    const current = link;
    if (!current) {
      return;
    }
    const done = await session.guardVoid(
      () => invoke("steam_guard_link_transfer_start", { sessionId: current.session_id }),
      GuardAction.TRANSFER_START,
    );
    if (done !== null) {
      transferStarted = true;
    }
  };

  const finishTransfer = async (): Promise<void> => {
    const current = link;
    if (!current) {
      return;
    }
    const result = await session.guard(
      () =>
        invoke("steam_guard_link_transfer_finish", {
          sessionId: current.session_id,
          smsCode: linkCode,
        }),
      GuardAction.TRANSFER_FINISH,
    );
    if (result) {
      session.applyResult(result);
      link = null;
      session.setScreen(SteamGuardScreen.HOME);
      showToast(ToastKind.SUCCESS, t.guard_transfer_success(), { account: deps.entry() });
    }
  };

  const revokeDuringLink = (): void => {
    const entry = deps.entry();
    if (!entry) return;
    const current = link;
    if (!current || !revocationInput) {
      session.fail("REVOCATION_CODE_REQUIRED", SteamGuardField.REVOCATION);
      return;
    }
    askConfirm({
      title: t.guard_revoke_title(),
      steamId: entry.steam_id,
      text: rich(t.guard_revoke_description, { account: entry.display_name }),
      confirmLabel: t.guard_revoke_confirm_action(),
      busyLabel: t.common_removing(),
      danger: true,
      action: async () => {
        const done = await session.guardVoid(
          () =>
            invoke("steam_guard_link_revoke", {
              sessionId: current.session_id,
              revocationCode: revocationInput,
            }),
          GuardAction.REVOKE,
        );
        if (done !== null) {
          link = null;
          session.setScreen(SteamGuardScreen.HOME);
          showToast(ToastKind.SUCCESS, t.guard_revoke_success(), { account: entry });
        }
        return done !== null;
      },
    });
  };

  const cancelLink = async (): Promise<void> => {
    if (!link) return;
    const sessionId = link.session_id;
    const done = await session.guardVoid(
      () => invoke("steam_guard_link_cancel", { sessionId }),
      GuardAction.CANCEL_LINK,
    );
    if (done) {
      link = null;
      session.setScreen(SteamGuardScreen.HOME);
    }
  };

  const clearRevocation = (): void => {
    revocationInput = "";
  };

  return {
    reset,
    start,
    finalizeLink,
    startTransfer,
    finishTransfer,
    revokeDuringLink,
    cancelLink,
    clearRevocation,
    get link() {
      return link;
    },
    get linkCode() {
      return linkCode;
    },
    set linkCode(value: string) {
      linkCode = value;
      session.clearFieldError(SteamGuardField.LINK_CODE);
    },
    get transferStarted() {
      return transferStarted;
    },
    get revocationInput() {
      return revocationInput;
    },
    set revocationInput(value: string) {
      revocationInput = value;
      session.clearFieldError(SteamGuardField.REVOCATION);
    },
  };
};

export type LinkControllerType = ReturnType<typeof createLinkController>;
