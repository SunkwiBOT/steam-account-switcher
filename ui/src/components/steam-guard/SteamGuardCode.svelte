<script lang="ts">
  import { errorMessage, resolveErrorCode } from "../../lib/errors/errors.svelte";

  import { t } from "../../lib/i18n/i18n.svelte";

  import CopyButton from "../ui/CopyButton.svelte";
  import { copySteamGuardCode } from "../../lib/actions/actions.svelte";
  import { app } from "../../lib/state/state.svelte";
  import { invoke } from "../../lib/api/tauri";
  import type { CodeInfoType } from "../../lib/types";

  type SteamGuardCodePropsType = { steamId: string };

  let { steamId }: SteamGuardCodePropsType = $props();
  const selectedId = $derived(steamId);
  const vaultLocked = $derived(app.snapshot?.vault.locked === true);
  let code = $state<CodeInfoType | null>(null);
  let error = $state("");
  let ready = $state(false);
  let remaining = $state(0);
  const paused = $derived(app.idle || !ready);

  $effect(() => {
    const id = selectedId;
    code = null;
    error = "";
    void id;
  });

  $effect(() => {
    const id = selectedId;
    ready = false;
    if (app.idle || vaultLocked) return;
    let stopped = false;
    let inFlight = false;
    let expiresAt = 0;
    let retryAt = 0;
    const refresh = async () => {
      if (stopped || inFlight) return;
      inFlight = true;
      ready = false;
      try {
        const next = await invoke("steam_guard_code", { steamId: id });
        if (!stopped) {
          expiresAt = next.valid_until
            ? next.valid_until * 1000
            : Date.now() + next.seconds_remaining * 1000;
          remaining = Math.max(0, Math.ceil((expiresAt - Date.now()) / 1000));
          code = next;
          error = "";
          ready = remaining > 0;
        }
      } catch (cause) {
        if (!stopped) {
          error = resolveErrorCode(cause);
          retryAt = Date.now() + 3000;
        }
      } finally {
        inFlight = false;
      }
    };
    const tick = () => {
      remaining = Math.max(0, Math.ceil((expiresAt - Date.now()) / 1000));
      if (remaining === 0 && Date.now() >= retryAt) void refresh();
    };
    void refresh();
    // Only the countdown ticks in the UI; Rust generates a code at expiry or resume.
    const timer = setInterval(tick, 250);
    return () => {
      stopped = true;
      clearInterval(timer);
    };
  });
</script>

{#if code}
  <div class="steamguard-code" class:paused>
    <span class="steamguard-code-value" aria-hidden={paused}>{code.code}</span>
    <CopyButton
      copyKey={`guard:${steamId}`}
      label={t.guard_code_copy_label()}
      title={t.guard_code_copy_tooltip()}
      disabled={paused}
      action={() => copySteamGuardCode(steamId)}
    />
  </div>
  <div class="steamguard-timer" class:paused aria-hidden="true">
    <span style={`width:${((remaining / code.period) * 100).toFixed(1)}%`}></span>
  </div>
  <p class="steamguard-note">
    {app.idle
      ? t.guard_code_paused()
      : (error && errorMessage(error)) ||
        (ready ? t.guard_code_countdown({ seconds: remaining }) : t.guard_code_refresh_pending())}
  </p>
{:else}
  <p class="steamguard-note">
    {(error && errorMessage(error)) || (app.idle ? t.guard_code_hidden() : t.guard_code_loading())}
  </p>
{/if}
