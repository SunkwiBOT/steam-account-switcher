<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";

  import { untrack } from "svelte";
  import { ChevronDown, Eye, EyeOff, Fingerprint } from "@lucide/svelte";
  import { app } from "../../lib/state/state.svelte";
  import CopyButton from "../ui/CopyButton.svelte";
  import { steamIdentifiers } from "../../lib/accounts/steam-ids";

  type SteamIdentifiersPropsType = { steamId: string };

  let { steamId }: SteamIdentifiersPropsType = $props();
  const rows = $derived(steamIdentifiers(steamId));
  const globallyVisible = $derived(app.snapshot?.steam_ids_visible === true);
  let visible = $state(false);
  let previousId = "";
  let previousGlobal: boolean | undefined;
  $effect(() => {
    const id = steamId;
    const show = globallyVisible;
    untrack(() => {
      if (id === previousId && show === previousGlobal) return;
      previousId = id;
      previousGlobal = show;
      visible = show;
    });
  });

  let element = $state<HTMLDetailsElement | null>(null);
  let popup = $state<HTMLDivElement | null>(null);
  const position = (): void => {
    if (!element?.open || !popup) return;
    const box = element.getBoundingClientRect();
    const width = Math.min(580, window.innerWidth - 32);
    const below = window.innerHeight - box.bottom - 18;
    const above = box.top - 18;
    const height = Math.min(370, Math.max(below, above));
    const top =
      below >= Math.min(370, popup.scrollHeight) || below >= above
        ? box.bottom + 6
        : Math.max(12, box.top - Math.min(height, popup.scrollHeight) - 6);
    popup.style.width = `${width}px`;
    popup.style.left = `${Math.max(16, Math.min(box.right - width, window.innerWidth - width - 16))}px`;
    popup.style.top = `${top}px`;
    popup.style.maxHeight = `${height}px`;
  };
  $effect(() => {
    const details = element;
    if (!details) return;
    const outside = (event: PointerEvent) => {
      if (event.target instanceof Node && !details.contains(event.target)) details.open = false;
    };
    const escape = (event: KeyboardEvent) => {
      if (event.key === "Escape" && details.open) {
        details.open = false;
        details.querySelector("summary")?.focus();
        event.stopPropagation();
      }
    };
    document.addEventListener("pointerdown", outside);
    window.addEventListener("resize", position);
    window.addEventListener("scroll", position, true);
    details.addEventListener("keydown", escape);
    return () => {
      document.removeEventListener("pointerdown", outside);
      window.removeEventListener("resize", position);
      window.removeEventListener("scroll", position, true);
      details.removeEventListener("keydown", escape);
    };
  });
</script>

{#if rows.length}
  <details class="account-identifiers identifiers-picker" bind:this={element} ontoggle={position}>
    <summary><Fingerprint size={15} /> {t.steam_ids_title()} <ChevronDown size={15} /></summary>
    <div class="identifiers-popover" bind:this={popup}>
      <div class="identifiers-toolbar">
        <span>{t.steam_ids_description()}</span>
        <button
          class="copy-button"
          type="button"
          title={visible ? t.settings_privacy_hide_ids() : t.steam_ids_show()}
          aria-label={visible ? t.settings_privacy_hide_ids() : t.steam_ids_show()}
          aria-pressed={visible}
          onclick={() => (visible = !visible)}
        >
          {#if visible}<EyeOff size={15} />{:else}<Eye size={15} />{/if}
        </button>
      </div>
      <dl class="identifiers-list">
        {#each rows as row (row.value)}
          <div>
            <dt>{row.label}</dt>
            <dd>
              <span>{visible ? row.value : "******"}</span>
              <CopyButton
                {steamId}
                copyKey={`id:${steamId}:${row.value}`}
                label={t.clipboard_copy_label({ value: row.label })}
                valueLabel={row.label}
                read={() => row.value}
              />
            </dd>
          </div>
        {/each}
      </dl>
    </div>
  </details>
{/if}
