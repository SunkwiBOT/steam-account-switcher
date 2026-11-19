import { locale, t } from "../i18n/i18n.svelte";
import { app, run, showToast } from "./state.svelte";
import { EMPTY_SETTINGS } from "./settings";
import { ToastKind } from "./state.constants";
import { invoke } from "../api/tauri";
import type { Language } from "../bindings/Language";
import type { SearchScope } from "../bindings/SearchScope";
import type { UpdateInterval } from "../bindings/UpdateInterval";
import type { VaultLockInterval } from "../bindings/VaultLockInterval";
import { checkForUpdateIfDue } from "../actions/updates.svelte";

export type SettingsOverridesType = {
  language: Language;
  searchScope: SearchScope;
  autostart: boolean;
  closeToTray: boolean;
  masked: boolean;
  steamIdsVisible: boolean;
  showInstallationSelector: boolean;
  trackingEnabled: boolean;
  updateInterval: UpdateInterval;
  vaultLockInterval: VaultLockInterval;
};

let autostartEnabled = $state(false);
let autostartAvailable = $state(true);
let autostartLoaded = false;

const loadAutostart = (): void => {
  if (autostartLoaded) return;
  autostartLoaded = true;
  invoke("autostart_status")
    .then((enabled: boolean | null) => {
      autostartAvailable = enabled !== null;
      autostartEnabled = enabled === true;
    })
    .catch(() => {
      autostartAvailable = false;
    });
};
loadAutostart();

const settings = $derived(app.snapshot?.settings ?? EMPTY_SETTINGS);

const baseline = $derived<SettingsOverridesType>({
  language: locale.preference,
  searchScope: settings.search_scope,
  autostart: autostartEnabled,
  closeToTray: settings.close_to_tray !== false,
  masked: app.snapshot?.masked === true,
  steamIdsVisible: app.snapshot?.steam_ids_visible !== false,
  showInstallationSelector: settings.show_installation_selector !== false,
  trackingEnabled: settings.tracking_enabled !== false,
  updateInterval: settings.update_interval,
  vaultLockInterval: settings.vault_lock_interval,
});

let overrides = $state<Partial<SettingsOverridesType>>({});
let saving = $state(false);

const draft = $derived<SettingsOverridesType>({ ...baseline, ...overrides });
const dirty = $derived(Object.keys(overrides).length > 0);

const setDraft = <K extends keyof SettingsOverridesType>(
  key: K,
  value: SettingsOverridesType[K],
): void => {
  if (value === baseline[key]) {
    delete overrides[key];
  } else {
    overrides[key] = value;
  }
};

const cancel = (): void => {
  overrides = {};
};

const save = async (): Promise<void> => {
  if (saving) return;
  saving = true;
  try {
    const autostart = overrides.autostart;
    if (autostart !== undefined) {
      const result = await run("set_autostart", { enabled: autostart });
      if (result !== null) {
        autostartEnabled = autostart;
        showToast(
          ToastKind.SUCCESS,
          autostart ? t.settings_autostart_enabled() : t.settings_autostart_disabled(),
        );
        delete overrides.autostart;
      }
    }
    const language = overrides.language;
    if (language !== undefined) {
      const result = await run("set_language", { language });
      if (result !== null) delete overrides.language;
    }
    const searchScope = overrides.searchScope;
    if (searchScope !== undefined) {
      const result = await run("set_search_scope", { scope: searchScope });
      if (result !== null) delete overrides.searchScope;
    }
    const closeToTray = overrides.closeToTray;
    if (closeToTray !== undefined) {
      const result = await run("set_close_to_tray", { closeToTray });
      if (result !== null) delete overrides.closeToTray;
    }
    const masked = overrides.masked;
    if (masked !== undefined) {
      const result = await run("set_masked", { masked });
      if (result !== null) delete overrides.masked;
    }
    const steamIdsVisible = overrides.steamIdsVisible;
    if (steamIdsVisible !== undefined) {
      const result = await run("set_steam_ids_visible", { visible: steamIdsVisible });
      if (result !== null) delete overrides.steamIdsVisible;
    }
    const showInstallationSelector = overrides.showInstallationSelector;
    if (showInstallationSelector !== undefined) {
      const result = await run("update_settings", {
        showInstallationSelector,
      });
      if (result !== null) delete overrides.showInstallationSelector;
    }
    const trackingEnabled = overrides.trackingEnabled;
    if (trackingEnabled !== undefined) {
      const result = await run("set_tracking_enabled", { enabled: trackingEnabled });
      if (result !== null) delete overrides.trackingEnabled;
    }
    const updateInterval = overrides.updateInterval;
    if (updateInterval !== undefined) {
      const result = await run("set_update_interval", { interval: updateInterval });
      if (result !== null) {
        delete overrides.updateInterval;
        void checkForUpdateIfDue();
      }
    }
    const vaultLockInterval = overrides.vaultLockInterval;
    if (vaultLockInterval !== undefined) {
      const result = await run("set_vault_lock_interval", {
        interval: vaultLockInterval,
      });
      if (result !== null) delete overrides.vaultLockInterval;
    }
  } finally {
    saving = false;
  }
};

export const settingsDraft = {
  get baseline() {
    return baseline;
  },
  get value() {
    return draft;
  },
  get dirty() {
    return dirty;
  },
  get saving() {
    return saving;
  },
  get autostartAvailable() {
    return autostartAvailable;
  },
  set: setDraft,
  cancel,
  save,
};
