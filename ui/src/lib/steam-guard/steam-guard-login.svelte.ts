import { ErrorApiMessage } from "../errors/errors.constants";
import { resolveErrorCode } from "../errors/errors.svelte";
import {
  GuardAction,
  SteamGuardScreen,
  SteamGuardField,
  LoginPurpose,
  GuardKind,
} from "./steam-guard.constants";
import type { GuardSessionType } from "./steam-guard-session.svelte";
import type { AccountType, LoginBeginType } from "../types";
import { invoke } from "../api/tauri";

export type LoginDepsType = {
  session: GuardSessionType;
  entry: () => AccountType | null;
  startLink: () => Promise<void>;
  completeSignIn: () => Promise<void>;
  resetAll: () => void;
};

export const createLoginController = (deps: LoginDepsType) => {
  const { session } = deps;

  let password = $state("");
  let loginBegin = $state<LoginBeginType | null>(null);
  let selectedGuard = $state<GuardKind | null>(null);
  let loginCode = $state("");
  let waiting = $state(false);
  let loginPurpose = $state<LoginPurpose>(LoginPurpose.LINK);

  const guardNeedsCode = $derived(
    selectedGuard === GuardKind.DEVICE_CODE || selectedGuard === GuardKind.EMAIL_CODE,
  );

  const reset = (): void => {
    password = "";
    loginBegin = null;
    selectedGuard = null;
    loginCode = "";
    waiting = false;
    loginPurpose = LoginPurpose.LINK;
  };

  const cancelLogin = async (): Promise<void> => {
    if (session.busyAction === GuardAction.CANCEL_LOGIN) return;
    const sessionId = loginBegin?.session_id;
    if (!sessionId) {
      deps.resetAll();
      return;
    }
    session.invalidate();
    session.setBusy(false);
    waiting = false;
    const cancelled = await session.guardVoid(
      () => invoke("steam_guard_login_cancel", { sessionId }),
      GuardAction.CANCEL_LOGIN,
    );
    if (cancelled) deps.resetAll();
  };

  const chooseMethod = (methods: LoginBeginType["methods"]): void => {
    const codeMethod = methods.find(
      (method) => method.kind === GuardKind.DEVICE_CODE || method.kind === GuardKind.EMAIL_CODE,
    );
    selectedGuard = codeMethod?.kind ?? methods[0]?.kind ?? null;
  };

  const startPassword = async (): Promise<void> => {
    const entry = deps.entry();
    if (!entry || !password) {
      session.fail(ErrorApiMessage.STEAM_PASSWORD_REQUIRED, SteamGuardField.PASSWORD);
      return;
    }
    const begin = await session.guard(
      () =>
        invoke("steam_guard_login_password", {
          steamId: entry.steam_id,
          password,
        }),
      GuardAction.PASSWORD,
    );
    if (begin) {
      loginBegin = begin;
      password = "";
      chooseMethod(begin.methods);
    }
  };

  const startQr = async (): Promise<void> => {
    const begin = await session.guard(() => invoke("steam_guard_login_qr"), GuardAction.QR);
    if (begin) {
      loginBegin = begin;
      selectedGuard = null;
      waiting = true;
      void finishLogin(null, null);
    }
  };

  const needsExternalConfirmation = (): boolean => {
    return (
      selectedGuard === GuardKind.DEVICE_CONFIRMATION ||
      selectedGuard === GuardKind.EMAIL_CONFIRMATION
    );
  };

  const finishLogin = async (kind: GuardKind | null, value: string | null): Promise<void> => {
    const entry = deps.entry();
    if (!entry || !loginBegin) {
      return;
    }
    const sessionId = loginBegin.session_id;
    const steamId = entry.steam_id;
    const current = session.generation();
    waiting = true;
    try {
      const signedIn = await session.guard(
        () =>
          invoke("steam_guard_login_finish", {
            sessionId,
            guardKind: kind,
            code: value,
            steamId,
          }),
        GuardAction.LOGIN,
      );
      if (!signedIn) {
        return;
      }
      loginBegin = null;
      if (loginPurpose === LoginPurpose.LINK) {
        await deps.startLink();
      } else {
        await deps.completeSignIn();
      }
    } catch (cause) {
      if (current === session.generation()) {
        session.fail(resolveErrorCode(cause), SteamGuardField.LOGIN);
      }
    } finally {
      if (current === session.generation()) waiting = false;
    }
  };

  const beginSignIn = (): void => {
    loginPurpose = LoginPurpose.SIGN_IN;
    loginBegin = null;
    loginCode = "";
    session.clearError();
    session.setScreen(SteamGuardScreen.LOGIN);
  };

  const beginLink = (): void => {
    loginPurpose = LoginPurpose.LINK;
    loginBegin = null;
    loginCode = "";
    session.clearError();
    session.setScreen(SteamGuardScreen.LOGIN);
  };

  const submitPasswordCode = async (): Promise<void> => {
    if (guardNeedsCode) {
      let value = loginCode.trim();
      const entry = deps.entry();
      const steamId = entry?.steam_id;
      if (
        !value &&
        selectedGuard === GuardKind.DEVICE_CODE &&
        entry?.has_authenticator &&
        steamId
      ) {
        const current = await session.guard(
          () => invoke("steam_guard_code", { steamId }),
          GuardAction.LOGIN_CODE,
        );
        if (!current) return;
        value = current.code;
      }
      if (!value) {
        session.fail(ErrorApiMessage.GUARD_CODE_REQUIRED, SteamGuardField.LOGIN_CODE);
        return;
      }
      await finishLogin(selectedGuard, value);
    } else {
      await finishLogin(null, null);
    }
  };

  return {
    reset,
    cancelLogin,
    startPassword,
    startQr,
    needsExternalConfirmation,
    beginSignIn,
    beginLink,
    submitPasswordCode,
    get guardNeedsCode() {
      return guardNeedsCode;
    },
    get password() {
      return password;
    },
    set password(value: string) {
      password = value;
      session.clearFieldError(SteamGuardField.PASSWORD);
    },
    get loginBegin() {
      return loginBegin;
    },
    get selectedGuard() {
      return selectedGuard;
    },
    get loginCode() {
      return loginCode;
    },
    set loginCode(value: string) {
      loginCode = value;
      session.clearFieldError(SteamGuardField.LOGIN_CODE);
    },
    get waiting() {
      return waiting;
    },
    get loginPurpose() {
      return loginPurpose;
    },
  };
};

export type LoginControllerType = ReturnType<typeof createLoginController>;
