import type { ErrorApiMessage } from "../errors/errors.constants";
import { errorMessage, resolveErrorCode } from "../errors/errors.svelte";
import { t } from "../i18n/i18n.svelte";
import type { GuardBusyActionType } from "./steam-guard.constants";
import { isCurrentResponse } from "./steam-guard-rules";
import { showToast } from "../state/state.svelte";
import type { ToastAccountType } from "../types";
import { invoke } from "../api/tauri";
import type { ConfirmationItemType } from "../types";

export type ConfirmationsDepsType = {
  steamId: () => string | null;
  account: () => ToastAccountType | null;
  generation: () => number;
  isBusy: () => boolean;
  runVoid: (action: () => Promise<void>, name: GuardBusyActionType) => Promise<boolean | null>;
};

export const createConfirmationsController = (deps: ConfirmationsDepsType) => {
  let items = $state<ConfirmationItemType[]>([]);
  let busy = $state(false);
  let loaded = $state(false);
  let attempted = $state(false);
  let error = $state<ErrorApiMessage | "">("");
  //! Bumped whenever the list is invalidated so a load or action started
  //! before it cannot write its result afterwards.
  let revision = 0;
  let refreshQueued = false;

  const isCurrent = (requested: { generation: number; revision: number }): boolean =>
    isCurrentResponse(requested, { generation: deps.generation(), revision });

  const invalidate = (): void => {
    revision += 1;
  };

  const clear = (): void => {
    invalidate();
    items = [];
    loaded = false;
    attempted = false;
    error = "";
    refreshQueued = false;
  };

  const reset = (): void => {
    clear();
    busy = false;
  };

  const load = async (): Promise<void> => {
    const steamId = deps.steamId();
    if (!steamId || deps.isBusy()) return;
    if (busy) {
      refreshQueued = true;
      return;
    }
    const requested = { generation: deps.generation(), revision };
    busy = true;
    attempted = true;
    error = "";
    try {
      const list = await invoke("steam_guard_confirmations", { steamId });
      if (isCurrent(requested)) {
        items = list;
        loaded = true;
      }
    } catch (cause) {
      if (isCurrent(requested)) error = resolveErrorCode(cause);
    } finally {
      if (requested.generation === deps.generation()) {
        busy = false;
        if (refreshQueued) {
          refreshQueued = false;
          void load();
        }
      }
    }
  };

  const act = async (item: ConfirmationItemType, accept: boolean): Promise<void> => {
    const steamId = deps.steamId();
    if (!steamId || deps.isBusy()) return;
    invalidate();
    const done = await deps.runVoid(
      () =>
        invoke("steam_guard_confirmation_action", {
          steamId,
          id: item.id,
          nonce: item.nonce,
          accept,
        }),
      `confirmation:${item.id}:${accept}`,
    );
    if (done !== null) {
      items = items.filter((candidate) => candidate.id !== item.id);
      showToast(
        accept ? t.guard_confirmation_accept_success() : t.guard_confirmation_refuse_success(),
        true,
        deps.account(),
      );
    }
    await load();
  };

  return {
    get items() {
      return items;
    },
    get busy() {
      return busy;
    },
    get loaded() {
      return loaded;
    },
    get attempted() {
      return attempted;
    },
    get error() {
      return error ? errorMessage(error) : "";
    },
    invalidate,
    clear,
    reset,
    load,
    act,
  };
};
