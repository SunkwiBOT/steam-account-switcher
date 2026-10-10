<script lang="ts">
  import { Language } from "../../lib/i18n/i18n.constants";
  import { SearchScope } from "../../lib/state/state.constants";
  import { locale, t } from "../../lib/i18n/i18n.svelte";
  import { dropdown, openDropdown } from "../../lib/state/menus.svelte";

  import { app, EMPTY_SETTINGS, run, showToast } from "../../lib/state/state.svelte";
  import { invoke } from "../../lib/api/tauri";
  import type { SettingsType } from "../../lib/types";
  import AsyncIcon from "../ui/AsyncIcon.svelte";
  import SettingToggle from "./SettingToggle.svelte";

  import { ChevronDown, Eye, Languages, Power, Search } from "@lucide/svelte";

  const settings = $derived<SettingsType>(app.snapshot?.settings ?? EMPTY_SETTINGS);

  let autostartEnabled = $state(false);
  let autostartAvailable = $state(true);
  let languageBusy = $state(false);
  let languageButton = $state<HTMLButtonElement | null>(null);
  const languageOptions = $derived([
    { value: Language.AUTO, label: t.language_system() },
    { value: Language.EN, label: t.language_english() },
    { value: Language.FR, label: t.language_french() },
  ]);
  const languageLabel = $derived(
    languageOptions.find((option) => option.value === locale.preference)?.label ??
      t.language_system(),
  );

  const changeLanguage = async (language: Language): Promise<void> => {
    if (languageBusy || language === locale.preference) return;
    languageBusy = true;
    try {
      await run("set_language", { language });
    } finally {
      languageBusy = false;
    }
  };

  const pickLanguage = (): void => {
    if (!languageButton || languageBusy) return;
    openDropdown({
      anchor: languageButton,
      value: locale.preference,
      options: languageOptions,
      onSelect: (value) => {
        const language = Object.values(Language).find((choice) => choice === value);
        if (language) void changeLanguage(language);
      },
    });
  };

  let searchBusy = $state(false);
  let searchButton = $state<HTMLButtonElement | null>(null);
  const searchScopes = $derived([
    { value: SearchScope.BOTH, label: t.settings_search_both() },
    { value: SearchScope.DISPLAY_NAME, label: t.settings_search_display_name() },
    { value: SearchScope.IDENTIFIER, label: t.settings_search_identifier() },
  ]);
  const searchScopeLabel = $derived(
    searchScopes.find((option) => option.value === settings.search_scope)?.label ??
      t.settings_search_both(),
  );

  const changeSearchScope = async (scope: SearchScope): Promise<void> => {
    if (searchBusy || scope === settings.search_scope) return;
    searchBusy = true;
    try {
      await run("set_search_scope", { scope });
    } finally {
      searchBusy = false;
    }
  };

  const pickSearchScope = (): void => {
    if (!searchButton || searchBusy) return;
    openDropdown({
      anchor: searchButton,
      value: settings.search_scope,
      options: searchScopes,
      onSelect: (value) => {
        const scope = Object.values(SearchScope).find((choice) => choice === value);
        if (scope) void changeSearchScope(scope);
      },
    });
  };

  $effect(() => {
    invoke("autostart_status")
      .then((enabled: boolean) => {
        autostartAvailable = enabled !== null;
        autostartEnabled = enabled === true;
      })
      .catch(() => {
        autostartAvailable = false;
      });
  });

  const toggleAutostart = async (enabled: boolean): Promise<void> => {
    const result = await run("set_autostart", { enabled });
    if (result === null) {
      autostartEnabled = !enabled;
    } else {
      autostartEnabled = enabled;
      showToast(enabled ? t.settings_autostart_enabled() : t.settings_autostart_disabled(), true);
    }
  };
</script>

<div class="settings-cards">
  <section class="settings-card">
    <h3 class="settings-card-title"><Languages size={15} /> {t.language_label()}</h3>
    <p class="settings-note">{t.language_description()}</p>
    <button
      class="select-button"
      type="button"
      aria-label={t.language_label()}
      aria-haspopup="listbox"
      aria-expanded={dropdown.open && dropdown.anchor === languageButton}
      bind:this={languageButton}
      onclick={pickLanguage}
      disabled={languageBusy}
      aria-busy={languageBusy}
    >
      <span>{languageBusy ? t.common_saving() : languageLabel}</span>
      <span class="picker-caret">
        <AsyncIcon icon={ChevronDown} busy={languageBusy} size={14} />
      </span>
    </button>
  </section>
  <section class="settings-card">
    <h3 class="settings-card-title"><Search size={15} /> {t.settings_search_title()}</h3>
    <p class="settings-note">{t.settings_search_description()}</p>
    <button
      class="select-button"
      type="button"
      aria-label={t.settings_search_label()}
      aria-haspopup="listbox"
      aria-expanded={dropdown.open && dropdown.anchor === searchButton}
      bind:this={searchButton}
      onclick={pickSearchScope}
      disabled={searchBusy}
      aria-busy={searchBusy}
    >
      <span>{searchBusy ? t.common_saving() : searchScopeLabel}</span>
      <span class="picker-caret">
        <AsyncIcon icon={ChevronDown} busy={searchBusy} size={14} />
      </span>
    </button>
  </section>
  <section class="settings-card">
    <h3 class="settings-card-title"><Power size={15} /> {t.settings_startup_title()}</h3>
    <SettingToggle
      label={t.settings_autostart_label()}
      checked={autostartEnabled}
      disabled={!autostartAvailable}
      onchange={toggleAutostart}
    />
    <SettingToggle
      label={t.settings_close_quit_label()}
      checked={settings.close_to_tray === false}
      onchange={(enabled) => run("set_close_to_tray", { closeToTray: !enabled })}
    />
    <p class="settings-note">
      {settings.close_to_tray === false
        ? settings.tracking_enabled === false
          ? t.settings_close_quit_description()
          : t.settings_close_quit_tracking_description()
        : settings.tracking_enabled === false
          ? t.settings_close_tray_description()
          : t.settings_close_tray_tracking_description()}
    </p>
  </section>

  <section class="settings-card">
    <h3 class="settings-card-title"><Eye size={15} /> {t.settings_privacy_title()}</h3>
    <SettingToggle
      label={t.settings_privacy_hide_login_names()}
      checked={app.snapshot?.masked === true}
      onchange={(enabled) => run("set_masked", { masked: enabled })}
    />
    <SettingToggle
      label={t.settings_privacy_hide_ids()}
      checked={app.snapshot?.steam_ids_visible === false}
      onchange={(enabled) => run("set_steam_ids_visible", { visible: !enabled })}
    />
  </section>
</div>
