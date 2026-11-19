<script lang="ts">
  import { LANGUAGE_LABELS, RESOLVED_LANGUAGES } from "../../lib/i18n/i18n.constants";
  import { t } from "../../lib/i18n/i18n.svelte";
  import SelectSetting from "../ui/SelectSetting.svelte";

  import { settingsDraft } from "../../lib/state/settings-draft.svelte";
  import type { Language } from "../../lib/bindings/Language";
  import type { SearchScope } from "../../lib/bindings/SearchScope";
  import SettingToggle from "./SettingToggle.svelte";

  import { Eye, Languages, Power, Search } from "@lucide/svelte";

  const draft = $derived(settingsDraft.value);
  const languageOptions = $derived<{ value: Language; label: string }[]>([
    { value: "auto", label: t.language_system() },
    ...RESOLVED_LANGUAGES.map((value) => ({ value, label: LANGUAGE_LABELS[value] })),
  ]);

  const languageLabel = $derived(
    languageOptions.find((option) => option.value === draft.language)?.label ?? t.language_system(),
  );

  const searchScopes = $derived<{ value: SearchScope; label: string }[]>([
    { value: "BOTH", label: t.settings_search_both() },
    { value: "DISPLAY_NAME", label: t.settings_search_display_name() },
    { value: "IDENTIFIER", label: t.settings_search_identifier() },
  ]);
  const searchScopeLabel = $derived(
    searchScopes.find((option) => option.value === draft.searchScope)?.label ??
      t.settings_search_both(),
  );
</script>

<div class="settings-cards">
  <section class="settings-card">
    <h3 class="settings-card-title"><Languages size={15} /> {t.language_label()}</h3>
    <p class="settings-note">{t.language_description()}</p>
    <SelectSetting
      value={draft.language}
      options={languageOptions}
      label={languageLabel}
      ariaLabel={t.language_label()}
      disabled={settingsDraft.saving}
      onselect={(value) => settingsDraft.set("language", value as Language)}
    />
  </section>
  <section class="settings-card">
    <h3 class="settings-card-title"><Search size={15} /> {t.settings_search_title()}</h3>
    <p class="settings-note">{t.settings_search_description()}</p>
    <SelectSetting
      value={draft.searchScope}
      options={searchScopes}
      label={searchScopeLabel}
      ariaLabel={t.settings_search_label()}
      disabled={settingsDraft.saving}
      onselect={(value) => settingsDraft.set("searchScope", value as SearchScope)}
    />
  </section>
  <section class="settings-card">
    <h3 class="settings-card-title"><Power size={15} /> {t.settings_startup_title()}</h3>
    <SettingToggle
      label={t.settings_autostart_label()}
      checked={draft.autostart}
      disabled={!settingsDraft.autostartAvailable || settingsDraft.saving}
      onchange={async (enabled) => {
        settingsDraft.set("autostart", enabled);
      }}
    />
    <SettingToggle
      label={t.settings_close_quit_label()}
      checked={!draft.closeToTray}
      disabled={settingsDraft.saving}
      onchange={async (enabled) => {
        settingsDraft.set("closeToTray", !enabled);
      }}
    />
    <p class="settings-note">
      {!draft.closeToTray
        ? draft.trackingEnabled === false
          ? t.settings_close_quit_description()
          : t.settings_close_quit_tracking_description()
        : draft.trackingEnabled === false
          ? t.settings_close_tray_description()
          : t.settings_close_tray_tracking_description()}
    </p>
  </section>

  <section class="settings-card">
    <h3 class="settings-card-title"><Eye size={15} /> {t.settings_privacy_title()}</h3>
    <SettingToggle
      label={t.settings_privacy_hide_login_names()}
      checked={draft.masked}
      disabled={settingsDraft.saving}
      onchange={async (enabled) => {
        settingsDraft.set("masked", enabled);
      }}
    />
    <SettingToggle
      label={t.settings_privacy_hide_ids()}
      checked={!draft.steamIdsVisible}
      disabled={settingsDraft.saving}
      onchange={async (enabled) => {
        settingsDraft.set("steamIdsVisible", !enabled);
      }}
    />
  </section>
</div>
