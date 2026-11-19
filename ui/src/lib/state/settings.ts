import type { SettingsDto } from "../bindings/SettingsDto";

export const EMPTY_SETTINGS: SettingsDto = {
  language: "auto",
  resolved_language: "en",
  show_installation_selector: true,
  tracking_enabled: true,
  tracking_history: [],
  close_to_tray: true,
  update_interval: "SIX_HOURS",
  search_scope: "BOTH",
  vault_lock_interval: "STARTUP",
  last_update_check_at: null,
  entries: [],
  database_path: "",
  vault_offer_dismissed: false,
};
