<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";

  import { untrack } from "svelte";
  import { Popover } from "bits-ui";
  import { ChevronDown, Eye, EyeOff, Fingerprint } from "@lucide/svelte";
  import { app } from "../../lib/state/state.svelte";
  import CopyButton from "../ui/CopyButton.svelte";
  import { steamIdentifiers } from "../../lib/accounts/steam-ids";
  import { MASKED_LABEL } from "../../lib/accounts/identity";

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
</script>

{#if rows.length}
  <Popover.Root>
    <Popover.Trigger
      class="group flex min-h-[54px] w-full items-center gap-[7px] rounded-lg border border-[var(--border)] bg-[var(--base)] px-2.5 py-[9px] text-[var(--muted)] focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-[var(--accent)]"
    >
      <Fingerprint size={15} />
      {t.steam_ids_title()}
      <ChevronDown size={15} class="ml-auto data-open:rotate-180" />
    </Popover.Trigger>
    <Popover.Portal>
      <Popover.Content
        sideOffset={6}
        align="start"
        class="z-50 w-[min(580px,82vw)] max-h-[min(370px,55vh)] overflow-auto rounded-lg border border-[var(--border-strong)] bg-[var(--base)] shadow-[0_8px_24px_var(--popover-shadow)]"
      >
        <div
          class="flex items-center justify-between gap-2 px-2.5 py-2 text-[12px] text-[var(--muted)]"
        >
          <span>{t.steam_ids_description()}</span>
          <button
            class="inline-flex size-[25px] items-center justify-center rounded-md border border-transparent bg-transparent p-0 text-[var(--muted)] not-disabled:hover:border-[var(--border-strong)] not-disabled:hover:bg-[var(--button)] not-disabled:hover:text-[var(--text-bright)] focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-[var(--accent)] disabled:cursor-default disabled:opacity-45"
            type="button"
            title={visible ? t.settings_privacy_hide_ids() : t.steam_ids_show()}
            aria-label={visible ? t.settings_privacy_hide_ids() : t.steam_ids_show()}
            aria-pressed={visible}
            onclick={() => (visible = !visible)}
          >
            {#if visible}<EyeOff size={15} />{:else}<Eye size={15} />{/if}
          </button>
        </div>
        <dl class="m-0">
          {#each rows as row (row.value)}
            <div
              class="grid grid-cols-[130px_minmax(0,1fr)] items-center gap-3 border-t border-[var(--border)] px-2.5 py-1.5 max-[540px]:grid-cols-[100px_minmax(0,1fr)]"
            >
              <dt class="text-[var(--muted)]">{row.label}</dt>
              <dd class="m-0 flex min-w-0 items-center gap-2">
                <span class="[overflow-wrap:anywhere] [font-variant-numeric:tabular-nums]"
                  >{visible ? row.value : MASKED_LABEL}</span
                >
                <CopyButton
                  {steamId}
                  className="copy-button ml-auto shrink-0"
                  copyKey={`id:${steamId}:${row.value}`}
                  label={t.clipboard_copy_label({ value: row.label })}
                  valueLabel={row.label}
                  read={() => row.value}
                />
              </dd>
            </div>
          {/each}
        </dl>
      </Popover.Content>
    </Popover.Portal>
  </Popover.Root>
{/if}
