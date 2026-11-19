export enum SteamGuardScreen {
  HOME = "home",
  IMPORT = "import",
  LOGIN = "login",
  LINK = "link",
}

export enum LoginPurpose {
  LINK = "link",
  SIGN_IN = "signin",
}

export enum GuardAction {
  IDLE = "",
  ACTION = "action",
  PASSWORD = "password",
  QR = "qr",
  LOGIN = "login",
  LOGIN_CODE = "login-code",
  LINK = "link",
  CANCEL_LOGIN = "cancel-login",
  CANCEL_LINK = "cancel-link",
  IMPORT = "import",
  REMOVE = "remove",
  FINALIZE = "finalize",
  TRANSFER_START = "transfer-start",
  TRANSFER_FINISH = "transfer-finish",
  REVOKE = "revoke",
}

export type ConfirmationActionType = `confirmation:${string}:${boolean}`;
export type GuardBusyActionType = GuardAction | ConfirmationActionType;

export enum SteamGuardField {
  PASSWORD = "password",
  LOGIN_CODE = "login-code",
  LOGIN = "login",
  IMPORT_PASSWORD = "import-password",
  IMPORT = "import",
  LINK_CODE = "link-code",
  TRANSFER = "transfer",
  REVOCATION = "revocation",
  LINK = "link",
  SECURITY = "security",
  CONFIRMATIONS = "confirmations",
  GENERAL = "general",
}
