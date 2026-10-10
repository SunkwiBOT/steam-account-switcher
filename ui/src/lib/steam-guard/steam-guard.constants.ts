export enum GuardKind {
  DEVICE_CODE = "device_code",
  EMAIL_CODE = "email_code",
  DEVICE_CONFIRMATION = "device_confirmation",
  EMAIL_CONFIRMATION = "email_confirmation",
  NONE = "none",
  UNKNOWN = "unknown",
}

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
  BROWSE = "browse",
  IMPORT = "import",
  REMOVE = "remove",
  FINALIZE = "finalize",
  TRANSFER_START = "transfer-start",
  TRANSFER_FINISH = "transfer-finish",
  REVOKE = "revoke",
}

export enum LinkStatus {
  RESUME = "resume",
  LINKED = "linked",
  AUTHENTICATOR_PRESENT = "authenticator_present",
  NEEDS_PHONE = "needs_phone",
  NEEDS_EMAIL = "needs_email",
}

export enum LinkConfirmKind {
  SMS = "sms",
  EMAIL = "email",
  UNKNOWN = "unknown",
}

export type ConfirmationActionType = `confirmation:${string}:${boolean}`;
export type GuardBusyActionType = GuardAction | ConfirmationActionType;

export enum SteamGuardField {
  PASSWORD = "password",
  LOGIN_CODE = "login-code",
  LOGIN = "login",
  IMPORT_FILE = "import-file",
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
