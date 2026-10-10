export enum SettingsSection {
  PREFERENCES = "preferences",
  FOLDERS = "folders",
  INSTALLATIONS = "installations",
  PLAYTIME = "playtime",
  VAULT = "vault",
  LOGS = "logs",
  UPDATES = "updates",
}

export enum UpdateCheck {
  IDLE = "idle",
  CHECKING = "checking",
  SUCCESS = "success",
  ERROR = "error",
}

export enum UpdateInterval {
  NEVER = "never",
  STARTUP = "startup",
  FIFTEEN_MINUTES = "15m",
  THIRTY_MINUTES = "30m",
  ONE_HOUR = "1h",
  SIX_HOURS = "6h",
  ONE_DAY = "1d",
  ONE_WEEK = "1w",
}

export enum SearchScope {
  /** Only the name shown on the card is matched. */
  DISPLAY_NAME = "display_name",
  /** Only the Steam login name and the Steam ID are matched. */
  IDENTIFIER = "identifier",
  BOTH = "both",
}

export enum VaultLockInterval {
  /** The vault is only locked when the app starts. */
  STARTUP = "startup",
  FIVE_MINUTES = "5m",
  TEN_MINUTES = "10m",
  FIFTEEN_MINUTES = "15m",
  TWENTY_MINUTES = "20m",
  THIRTY_MINUTES = "30m",
  ONE_HOUR = "1h",
  TWO_HOURS = "2h",
  FOUR_HOURS = "4h",
}

export enum UpdateStage {
  IDLE = "",
  DOWNLOADING = "downloading",
  DOWNLOADED = "downloaded",
  INSTALLING = "installing",
}

export enum ReportTab {
  GAME = "game",
  ACCOUNT = "account",
  DAILY = "daily",
  HISTORY = "history",
}
