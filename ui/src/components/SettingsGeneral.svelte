<script lang="ts">
  import { app, run, showToast } from "../lib/state.svelte";
  import { invoke } from "../lib/tauri";
  import type { Settings } from "../lib/types";

  const EMPTY_SETTINGS: Settings = {
    show_installation_selector: true,
    show_machine_playtime: true,
    entries: [],
    settings_path: "",
  };
  const settings = $derived<Settings>(app.snapshot?.settings ?? EMPTY_SETTINGS);

  let autostartEnabled = $state(false);
  let autostartAvailable = $state(true);
  let autostartBusy = $state(false);

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

  async function toggleAutostart(event: Event): Promise<void> {
    const input = event.currentTarget as HTMLInputElement;
    const enabled = input.checked;
    autostartBusy = true;
    const result = await run("set_autostart", { enabled });
    autostartBusy = false;
    if (result === null) {
      autostartEnabled = !enabled;
    } else {
      autostartEnabled = enabled;
      showToast(enabled ? "Autostart enabled." : "Autostart disabled.", true);
    }
  }
</script>

<div class="settings-cards">
  <section class="settings-card">
    <h3 class="settings-card-title">Appearance</h3>
    <label class="setting-line">
      <span>Installation selector in the header</span>
      <input
        class="switch"
        type="checkbox"
        checked={settings.show_installation_selector !== false}
        onchange={(event) =>
          run("update_settings", {
            showInstallationSelector: event.currentTarget.checked,
            showMachinePlaytime: settings.show_machine_playtime !== false,
          })}
      />
    </label>
    <label class="setting-line">
      <span>“This PC” total in the footer</span>
      <input
        class="switch"
        type="checkbox"
        checked={settings.show_machine_playtime !== false}
        onchange={(event) =>
          run("update_settings", {
            showInstallationSelector: settings.show_installation_selector !== false,
            showMachinePlaytime: event.currentTarget.checked,
          })}
      />
    </label>
  </section>

  <section class="settings-card">
    <h3 class="settings-card-title">Startup</h3>
    <label class="setting-line">
      <span>Start the switcher when you log in</span>
      <input
        class="switch"
        type="checkbox"
        checked={autostartEnabled}
        disabled={!autostartAvailable || autostartBusy}
        onchange={toggleAutostart}
      />
    </label>
    <p class="settings-note">
      The switcher stays in the system tray and keeps tracking local playtime.
    </p>
  </section>

  <section class="settings-card">
    <h3 class="settings-card-title">Privacy</h3>
    <label class="setting-line">
      <span>Show Account Names</span>
      <input
        class="switch"
        type="checkbox"
        checked={app.snapshot?.masked === false}
        onchange={(event) => run("set_masked", { masked: !event.currentTarget.checked })}
      />
    </label>
    <label class="setting-line">
      <span>Show Steam ID 64</span>
      <input
        class="switch"
        type="checkbox"
        checked={app.snapshot?.steam_ids_visible === true}
        onchange={(event) => run("set_steam_ids_visible", { visible: event.currentTarget.checked })}
      />
    </label>
    <p class="settings-note">
      Only accounts the local Steam client already remembers are listed. Passwords, Steam Guard
      codes and tokens are never read or stored.
    </p>
  </section>
</div>
