import { ErrorApiMessage } from "../errors/errors.constants";
import { t, rich } from "../i18n/i18n.svelte";
import { GuardAction, SteamGuardScreen, SteamGuardField } from "./steam-guard.constants";
import type { GuardSessionType } from "./steam-guard-session.svelte";
import { askConfirm } from "../state/dialogs.svelte";
import { showToast } from "../state/state.svelte";
import { invoke } from "../api/tauri";
import type { AccountType, LinkStartType } from "../types";

export type LinkDepsType = {
  session: GuardSessionType;
  entry: () => AccountType | null;
};

export const createLinkController = (deps: LinkDepsType) => {
  const { session } = deps;

  let link = $state<LinkStartType | null>(null);
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
      showToast(t.guard_link_success(), true, deps.entry());
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
      showToast(t.guard_transfer_success(), true, deps.entry());
    }
  };

  const revokeDuringLink = async (): Promise<void> => {
    const entry = deps.entry();
    if (!entry) return;
    const current = link;
    if (!current || !revocationInput) {
      session.fail(ErrorApiMessage.REVOCATION_CODE_REQUIRED, SteamGuardField.REVOCATION);
      return;
    }
    const answer = await askConfirm({
      title: t.guard_revoke_title(),
      steamId: entry.steam_id,
      text: rich(t.guard_revoke_description, { account: entry.display_name }),
      confirmLabel: t.guard_revoke_confirm_action(),
      danger: true,
    });
    if (!answer) return;
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
      showToast(t.guard_revoke_success(), true, entry);
    }
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
