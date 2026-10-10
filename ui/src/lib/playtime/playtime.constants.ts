export enum PlaytimeStatus {
  IDLE = "",
  RUNNING = "running",
}

export enum TrackingCoverage {
  FULL = "full",
  PARTIAL = "partial",
}

export enum SessionEndReason {
  ACTIVE = "",
  UNKNOWN = "unknown",
  STEAM_RUNNING_LIST = "steam_running_list",
  ROOT_PROCESS_EXIT_LOG = "root_process_exit_log",
  PROCESS_EXIT = "process_exit",
  PROCESS_MISSING_AFTER_RESTART = "process_missing_after_restart",
  REPLACED_BY_NEW_LAUNCH = "replaced_by_new_launch",
  TRACKING_DISABLED = "tracking_disabled",
}
