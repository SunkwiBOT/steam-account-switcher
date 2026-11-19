import { t, rich } from "../i18n/i18n.svelte";
import { GuardAction, SteamGuardScreen, SteamGuardField } from "./steam-guard.constants";
import { SettingsSection } from "../state/state.constants";
import { untrack } from "svelte";
import { accountById, app, showToast } from "../state/state.svelte";
import { ToastKind } from "../state/state.constants";
import { invoke } from "../api/tauri";
import { askConfirm } from "../state/dialogs.svelte";
import { createConfirmationsController } from "./steam-guard-confirmations.svelte";
import { createGuardSession } from "./steam-guard-session.svelte";
import { createLoginController } from "./steam-guard-login.svelte";
import { createLinkController } from "./steam-guard-link.svelte";

export const createSteamGuardController = () => {
  const entry = $derived(accountById(app.steamGuardAccountId ?? ""));
  const selectedAccountId = $derived(entry?.steam_id ?? null);
  const vault = $derived(
    app.snapshot?.vault ?? {
      encrypted: false,
      path: "",
      locked: false,
      indexed: true,
      count: 0,
      error: null,
    },
  );

  const vaultReady = $derived(!vault.locked && !vault.error);

  let importPasskey = $state("");

  const session = createGuardSession({ needsCode: () => loginFlow.guardNeedsCode });

  const confirmations = createConfirmationsController({
    steamId: () => entry?.steam_id ?? null,
    account: () => entry,
    generation: () => session.generation(),
    isBusy: () => session.busy,
    runVoid: session.guardVoid,
  });

  const linkFlow = createLinkController({ session, entry: () => entry });

  const completeSignIn = async (): Promise<void> => {
    session.setScreen(SteamGuardScreen.HOME);
    confirmations.clear();
    session.applyResult(await invoke("snapshot"));
    showToast(ToastKind.SUCCESS, t.guard_confirmations_sign_in_success(), { account: entry });
  };

  const loginFlow = createLoginController({
    session,
    entry: () => entry,
    startLink: linkFlow.start,
    completeSignIn,
    resetAll: () => reset(),
  });

  const reset = (): void => {
    session.reset();
    importPasskey = "";
    confirmations.reset();
    loginFlow.reset();
    linkFlow.reset();
  };

  const back = (): void => {
    if (session.busy && !loginFlow.waiting) return;
    if (session.screen === SteamGuardScreen.LINK) {
      void linkFlow.cancelLink();
    } else if (session.screen === SteamGuardScreen.LOGIN) {
      void loginFlow.cancelLogin();
    } else {
      session.screen = SteamGuardScreen.HOME;
    }
  };

  const close = (): void => {
    if (session.busy && !loginFlow.waiting) return;
    if (loginFlow.loginBegin && !linkFlow.link) {
      void invoke("steam_guard_login_cancel", {
        sessionId: loginFlow.loginBegin.session_id,
      }).catch(() => {});
    }
    if (linkFlow.link) {
      void invoke("steam_guard_link_cancel", { sessionId: linkFlow.link.session_id }).catch(
        () => {},
      );
    }
    app.steamGuardOpen = false;
    app.steamGuardAccountId = null;
    reset();
  };

  $effect(() => {
    const open = app.steamGuardOpen;
    const accountId = selectedAccountId;
    untrack(() => {
      reset();
      if (open && !accountId) app.steamGuardOpen = false;
    });
  });

  $effect(() => {
    if (
      app.steamGuardOpen &&
      session.screen === SteamGuardScreen.HOME &&
      entry?.has_authenticator_session &&
      !vault.locked &&
      vaultReady &&
      !app.idle
    ) {
      untrack(() => {
        if (!confirmations.attempted) void confirmations.load();
      });
    }
  });

  const openVaultSettings = (): void => {
    close();
    app.settingsSection = SettingsSection.VAULT;
    app.settingsOpen = true;
  };

  const runImport = async (): Promise<void> => {
    if (!entry) return;
    const result = await session.guard(
      () =>
        invoke("steam_guard_import", {
          steamId: entry.steam_id,
          passkey: importPasskey || null,
        }),
      GuardAction.IMPORT,
    );
    if (result) {
      session.applyResult(result);
      session.setScreen(SteamGuardScreen.HOME);
      importPasskey = "";
      showToast(ToastKind.SUCCESS, t.guard_import_success(), { account: entry });
    }
  };

  const removeAuthenticator = (): void => {
    if (!entry) {
      return;
    }
    askConfirm({
      title: t.guard_remove_title(),
      steamId: entry.steam_id,
      text: rich(t.guard_remove_description, { account: entry.display_name }),
      confirmLabel: t.guard_remove_action(),
      busyLabel: t.common_removing(),
      danger: true,
      action: async () => {
        const result = await session.guard(
          () =>
            invoke("steam_guard_remove", {
              steamId: entry.steam_id,
              revocationCode: null,
            }),
          GuardAction.REMOVE,
        );
        if (result) {
          session.applyResult(result);
          linkFlow.clearRevocation();
          confirmations.clear();
          showToast(ToastKind.SUCCESS, t.guard_remove_success(), { account: entry });
        }
        return result !== null;
      },
    });
  };

  return {
    get entry() {
      return entry;
    },
    get vault() {
      return vault;
    },
    get vaultReady() {
      return vaultReady;
    },
    get screen() {
      return session.screen;
    },
    set screen(value: SteamGuardScreen) {
      session.screen = value;
    },
    get busy() {
      return session.busy;
    },
    set busy(value: boolean) {
      session.busy = value;
    },
    get busyAction() {
      return session.busyAction;
    },
    get error() {
      return session.error;
    },
    get confirmations() {
      return confirmations.items;
    },
    get confirmationsBusy() {
      return confirmations.busy;
    },
    get confirmationsLoaded() {
      return confirmations.loaded;
    },
    get confirmationsAttempted() {
      return confirmations.attempted;
    },
    get confirmationsError() {
      return confirmations.error;
    },
    get importPasskey() {
      return importPasskey;
    },
    set importPasskey(value: string) {
      importPasskey = value;
      session.clearFieldError(SteamGuardField.IMPORT_PASSWORD);
    },
    get password() {
      return loginFlow.password;
    },
    set password(value: string) {
      loginFlow.password = value;
    },
    get loginBegin() {
      return loginFlow.loginBegin;
    },
    get selectedGuard() {
      return loginFlow.selectedGuard;
    },
    get loginCode() {
      return loginFlow.loginCode;
    },
    set loginCode(value: string) {
      loginFlow.loginCode = value;
    },
    get waiting() {
      return loginFlow.waiting;
    },
    get loginPurpose() {
      return loginFlow.loginPurpose;
    },
    get link() {
      return linkFlow.link;
    },
    get linkCode() {
      return linkFlow.linkCode;
    },
    set linkCode(value: string) {
      linkFlow.linkCode = value;
    },
    get transferStarted() {
      return linkFlow.transferStarted;
    },
    get revocationInput() {
      return linkFlow.revocationInput;
    },
    set revocationInput(value: string) {
      linkFlow.revocationInput = value;
    },
    get guardNeedsCode() {
      return loginFlow.guardNeedsCode;
    },
    close,
    back,
    errorFor: session.errorFor,
    openVaultSettings,
    runImport,
    startPassword: loginFlow.startPassword,
    startQr: loginFlow.startQr,
    needsExternalConfirmation: loginFlow.needsExternalConfirmation,
    cancelLogin: loginFlow.cancelLogin,
    beginSignIn: loginFlow.beginSignIn,
    beginLink: loginFlow.beginLink,
    submitPasswordCode: loginFlow.submitPasswordCode,
    finalizeLink: linkFlow.finalizeLink,
    startTransfer: linkFlow.startTransfer,
    finishTransfer: linkFlow.finishTransfer,
    revokeDuringLink: linkFlow.revokeDuringLink,
    loadConfirmations: confirmations.load,
    actOnConfirmation: confirmations.act,
    removeAuthenticator,
    cancelLink: linkFlow.cancelLink,
  };
};

export type SteamGuardControllerType = ReturnType<typeof createSteamGuardController>;
