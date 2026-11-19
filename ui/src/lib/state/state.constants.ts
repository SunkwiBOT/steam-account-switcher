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

export enum ToastKind {
  SUCCESS = "success",
  ERROR = "error",
}

export const TOAST_DURATION_MS: Record<ToastKind, number> = {
  [ToastKind.SUCCESS]: 4_200,
  [ToastKind.ERROR]: 4_200,
};
