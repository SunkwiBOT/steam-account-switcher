<script lang="ts">
  import { errorMessage } from "../../lib/errors/errors.svelte";

  import { t } from "../../lib/i18n/i18n.svelte";

  import { IdentityField } from "./AccountIdentity.constants";
  import { untrack } from "svelte";
  import { Eye, EyeOff } from "@lucide/svelte";
  import { app, showToast } from "../../lib/state/state.svelte";
  import { invoke } from "../../lib/api/tauri";
  import AsyncIcon from "../ui/AsyncIcon.svelte";
  import CopyButton from "../ui/CopyButton.svelte";
  import type { AccountType } from "../../lib/types";

  type AccountIdentityPropsType = {
    entry: AccountType;
    field?: IdentityField;
  };

  let { entry, field = IdentityField.NAME }: AccountIdentityPropsType = $props();
  const hiddenGlobally = $derived(
    field === IdentityField.REVOCATION
      ? true
      : field === IdentityField.NAME
        ? app.snapshot?.masked !== false
        : !app.snapshot?.steam_ids_visible,
  );
  const label = $derived(
    field === IdentityField.REVOCATION
      ? t.guard_revocation_value()
      : field === IdentityField.NAME
        ? t.account_login_name_value()
        : t.steam_ids_64(),
  );
  const selectedId = $derived(entry.steam_id);
  let visible = $state(false);
  let revealedValue = $state("");
  let busy = $state(false);
  let generation = 0;
  let previousKey = "";

  $effect(() => {
    const id = selectedId;
    const hidden = hiddenGlobally;
    const key = `${id}:${field}:${hidden}:${field === IdentityField.REVOCATION && app.snapshot?.vault.locked}`;
    untrack(() => {
      if (!id || key === previousKey) return;
      previousKey = key;
      generation += 1;
      visible = !hidden;
      revealedValue = "";
      busy = false;
    });
  });

  const readValue = (): Promise<string> => {
    if (field === IdentityField.STEAM_ID) return Promise.resolve(entry.steam_id);
    return field === IdentityField.REVOCATION
      ? invoke("steam_guard_revocation_code", { steamId: entry.steam_id })
      : invoke("account_login_name", { steamId: entry.steam_id });
  };

  const toggle = async (): Promise<void> => {
    if (busy) return;
    if (visible) {
      visible = false;
      revealedValue = "";
      return;
    }
    const current = generation;
    busy = true;
    try {
      if (field === IdentityField.REVOCATION || (field === IdentityField.NAME && hiddenGlobally)) {
        const name = await readValue();
        if (current !== generation) return;
        revealedValue = name;
      }
      visible = true;
    } catch (cause) {
      if (current === generation) showToast(errorMessage(cause), false, entry);
    } finally {
      if (current === generation) busy = false;
    }
  };
</script>

<span class="fact-row">
  <span class="fact-value">
    {visible
      ? field === IdentityField.STEAM_ID
        ? entry.steam_id
        : field === IdentityField.REVOCATION
          ? revealedValue
          : revealedValue || entry.account_name
      : "******"}
  </span>
  <button
    class="copy-button"
    type="button"
    title={busy
      ? t.common_loading_value({ value: label })
      : visible
        ? t.common_hide_value({ value: label })
        : t.common_show_value({ value: label })}
    aria-label={busy
      ? t.common_loading_value({ value: label })
      : visible
        ? t.common_hide_value({ value: label })
        : t.common_show_value({ value: label })}
    aria-busy={busy}
    aria-pressed={visible}
    disabled={busy || (field === IdentityField.REVOCATION && app.snapshot?.vault.locked)}
    onclick={toggle}
  >
    <AsyncIcon icon={visible ? EyeOff : Eye} size={15} {busy} />
  </button>
  <CopyButton
    steamId={entry.steam_id}
    copyKey={field === IdentityField.REVOCATION
      ? `revocation:${entry.steam_id}`
      : field === IdentityField.NAME
        ? `name:${entry.steam_id}`
        : `id:${entry.steam_id}:Steam ID 64`}
    label={t.clipboard_copy_label({ value: label })}
    valueLabel={field === IdentityField.REVOCATION
      ? t.guard_revocation_label()
      : field === IdentityField.NAME
        ? t.guard_login_name_label()
        : label}
    read={readValue}
    disabled={field === IdentityField.REVOCATION && app.snapshot?.vault.locked}
  />
</span>
