import type { ConfirmationKind as ConfirmationKindDto } from "../bindings/ConfirmationKind";
import type { UiErrorCode } from "../errors/errors.svelte";
import { GuardAction, SteamGuardField } from "./steam-guard.constants";
import type { GuardBusyActionType } from "./steam-guard.constants";

export const fieldForGuardAction = (
  action: GuardBusyActionType,
  code: UiErrorCode,
  needsCode: boolean,
): SteamGuardField => {
  switch (action) {
    case GuardAction.PASSWORD:
      return SteamGuardField.PASSWORD;
    case GuardAction.LOGIN_CODE:
      return SteamGuardField.LOGIN_CODE;
    case GuardAction.LOGIN:
      return needsCode ? SteamGuardField.LOGIN_CODE : SteamGuardField.LOGIN;
    case GuardAction.QR:
    case GuardAction.CANCEL_LOGIN:
    case GuardAction.LINK:
      return SteamGuardField.LOGIN;
    case GuardAction.IMPORT:
      if (code === "BACKUP_PASSWORD_REQUIRED" || code === "BACKUP_DECRYPTION_FAILED")
        return SteamGuardField.IMPORT_PASSWORD;
      return SteamGuardField.IMPORT;
    case GuardAction.FINALIZE:
    case GuardAction.TRANSFER_FINISH:
      return SteamGuardField.LINK_CODE;
    case GuardAction.TRANSFER_START:
      return SteamGuardField.TRANSFER;
    case GuardAction.REVOKE:
      return SteamGuardField.REVOCATION;
    case GuardAction.CANCEL_LINK:
      return SteamGuardField.LINK;
    case GuardAction.REMOVE:
      return SteamGuardField.SECURITY;
    default:
      return action.startsWith("confirmation:")
        ? SteamGuardField.CONFIRMATIONS
        : SteamGuardField.GENERAL;
  }
};

export const isCurrentResponse = (
  requested: { generation: number; revision: number },
  current: { generation: number; revision: number },
): boolean =>
  requested.generation === current.generation && requested.revision === current.revision;

export const requiresExtraConfirmation = (kind: ConfirmationKindDto): boolean => kind !== "TRADE";
