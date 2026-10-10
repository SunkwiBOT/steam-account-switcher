import { ErrorApiMessage } from "../errors/errors.constants";
import { errorMessage, resolveErrorCode } from "../errors/errors.svelte";
import { GuardAction, SteamGuardScreen, SteamGuardField } from "./steam-guard.constants";
import type { GuardBusyActionType } from "./steam-guard.constants";
import { fieldForGuardAction } from "./steam-guard-rules";
import { applySnapshot, isSnapshot } from "../state/state.svelte";

export type GuardSessionDepsType = {
  needsCode: () => boolean;
};

export const createGuardSession = (deps: GuardSessionDepsType) => {
  let screen = $state<SteamGuardScreen>(SteamGuardScreen.HOME);
  let busy = $state(false);
  let busyAction = $state<GuardBusyActionType>(GuardAction.IDLE);
  let error = $state<ErrorApiMessage | "">("");
  let errorField = $state(SteamGuardField.GENERAL);

  // Bumped on every reset and cancel so responses from a closed or restarted session are dropped.
  let generation = 0;

  const invalidate = (): void => {
    generation += 1;
  };

  const reset = (): void => {
    invalidate();
    screen = SteamGuardScreen.HOME;
    busy = false;
    busyAction = GuardAction.IDLE;
    error = "";
  };

  const guard = async <T>(
    action: () => Promise<T>,
    name: GuardBusyActionType = GuardAction.ACTION,
  ): Promise<T | null> => {
    if (busy) return null;
    const current = generation;
    busy = true;
    busyAction = name;
    error = "";
    try {
      const result = await action();
      return current === generation ? result : null;
    } catch (cause) {
      if (current === generation) {
        error = resolveErrorCode(cause);
        errorField = fieldForGuardAction(name, error, deps.needsCode());
      }
      return null;
    } finally {
      if (current === generation) {
        busy = false;
        busyAction = GuardAction.IDLE;
      }
    }
  };

  const guardVoid = async (
    action: () => Promise<void>,
    name: GuardBusyActionType = GuardAction.ACTION,
  ): Promise<boolean | null> => {
    return guard(async () => {
      await action();
      return true;
    }, name);
  };

  const fail = (code: ErrorApiMessage, field: SteamGuardField): void => {
    error = code;
    errorField = field;
  };

  const clearError = (): void => {
    error = "";
  };

  const clearFieldError = (field: SteamGuardField): void => {
    if (errorField === field) error = "";
  };

  const errorFor = (field: SteamGuardField): string =>
    errorField === field && error ? errorMessage(error) : "";

  const applyResult = (result: unknown): void => {
    if (isSnapshot(result)) {
      applySnapshot(result);
    }
  };

  return {
    generation: (): number => generation,
    invalidate,
    reset,
    guard,
    guardVoid,
    fail,
    clearError,
    clearFieldError,
    errorFor,
    applyResult,
    setScreen(value: SteamGuardScreen): void {
      screen = value;
    },
    setBusy(value: boolean): void {
      busy = value;
    },
    get screen() {
      return screen;
    },
    set screen(value: SteamGuardScreen) {
      screen = value;
      error = "";
    },
    get busy() {
      return busy;
    },
    set busy(value: boolean) {
      busy = value;
    },
    get busyAction() {
      return busyAction;
    },
    get error() {
      return error ? errorMessage(error) : "";
    },
  };
};

export type GuardSessionType = ReturnType<typeof createGuardSession>;
