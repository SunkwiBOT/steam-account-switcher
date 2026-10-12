import { ErrorApiMessage } from "../errors/errors.constants";
import { GuardAction, SteamGuardField } from "./steam-guard.constants";
import type { GuardBusyActionType } from "./steam-guard.constants";

export const fieldForGuardAction = (
  action: GuardBusyActionType,
  code: ErrorApiMessage,
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
    case GuardAction.BROWSE:
      return SteamGuardField.IMPORT_FILE;
    case GuardAction.IMPORT:
      if (
        code === ErrorApiMessage.BACKUP_PASSWORD_REQUIRED ||
        code === ErrorApiMessage.BACKUP_DECRYPTION_FAILED
      )
        return SteamGuardField.IMPORT_PASSWORD;
      return SteamGuardField.IMPORT_FILE;
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

//! A response only lands when neither the session (generation) nor the
//! confirmation list (revision) moved on while the request was in flight.
export const isCurrentResponse = (
  requested: { generation: number; revision: number },
  current: { generation: number; revision: number },
): boolean =>
  requested.generation === current.generation && requested.revision === current.revision;
