<script lang="ts">
  import DialogHeader from "../dialogs/DialogHeader.svelte";
  import { t, formatDate, relativeDate } from "../../lib/i18n/i18n.svelte";

  import { formatSeconds } from "../../lib/api/api";
  import ColorSwatches from "../folders/ColorSwatches.svelte";
  import {
    openAccountReport,
    removeAccount,
    switchAccount,
  } from "../../lib/actions/actions.svelte";
  import { openDropdown } from "../../lib/state/menus.svelte";
  import {
    formatRating,
    premierBand,
    rankIconUrl,
    UNRANKED,
    rankLabel,
  } from "../../lib/ranks/ranks";
  import { RankKind } from "../../lib/ranks/ranks.constants";
  import { accountById, app, folders, run, trackingEnabled } from "../../lib/state/state.svelte";
  import type { AccountType, RankTemplateType } from "../../lib/types";
  import ActionButton from "../ui/ActionButton.svelte";
  import AccountIdentity from "./AccountIdentity.svelte";
  import AccountCaption from "./AccountCaption.svelte";
  import SteamGuardIcon from "../steam-guard/SteamGuardIcon.svelte";
  import SteamIdentifiers from "./SteamIdentifiers.svelte";
  import {
    ArrowRightLeft,
    ChartLine,
    Check,
    ChevronDown,
    Folder,
    Palette,
    Timer,
    Trash2,
    Trophy,
    UserRound,
  } from "@lucide/svelte";

  const entry = $derived<AccountType | null>(
    app.openAccountId ? accountById(app.openAccountId) : null,
  );
  const templates = $derived<RankTemplateType[]>(app.snapshot?.rank_templates ?? []);
  const template = $derived<RankTemplateType | undefined>(
    templates.find((candidate) => candidate.id === entry?.rank_template) ?? templates[0],
  );
  const isRating = $derived(template?.kind === RankKind.RATING);
  const rankIcon = $derived(template && entry ? rankIconUrl(template.id, entry.rank_family) : null);
  const folderName = $derived(
    folders().find((folder) => folder.id === entry?.folder_id)?.name ?? t.folder_none(),
  );

  let rating = $state<string | number>("");

  $effect(() => {
    const value = entry?.rank;
    if (value !== undefined) {
      rating = value === UNRANKED ? "" : String(value).replace(/[^0-9]/g, "");
    }
  });

  const ratingDigits = $derived(String(rating ?? "").replace(/[^0-9]/g, ""));
  const ratingColour = $derived(ratingDigits ? premierBand(ratingDigits).colour : "");
  const ratingTitle = $derived(
    ratingDigits
      ? `${formatRating(ratingDigits)} · ${premierBand(ratingDigits).name}`
      : t.rank_unranked(),
  );

  const close = (): void => {
    app.openAccountId = null;
  };

  const pickTemplate = (event: MouseEvent): void => {
    const anchor = event.currentTarget;
    if (!entry || !(anchor instanceof HTMLElement)) {
      return;
    }
    const account = entry;
    openDropdown({
      anchor,
      value: account.rank_template,
      options: templates.map((item) => ({ value: item.id, label: item.label })),
      onSelect: (id) =>
        run("set_rank", { steamId: account.steam_id, template: id, rank: account.rank }),
    });
  };

  const pickRank = (event: MouseEvent): void => {
    const anchor = event.currentTarget;
    if (!template || !entry || !(anchor instanceof HTMLElement)) {
      return;
    }
    const account = entry;
    const ladder = template;
    const iconFor = (family: string): string => {
      const url = rankIconUrl(ladder.id, family);
      return url
        ? `<img src="${url}" alt="" style="width:18px;height:18px;object-fit:contain" />`
        : "";
    };
    openDropdown({
      anchor,
      value: account.rank,
      options: ladder.ranks.map((rank) => ({
        value: rank.value,
        label: rankLabel(rank.value),
        icon: iconFor(rank.family),
      })),
      onSelect: (rank) => run("set_rank", { steamId: account.steam_id, template: ladder.id, rank }),
    });
  };

  const pickFolder = (event: MouseEvent): void => {
    const anchor = event.currentTarget;
    if (!entry || !(anchor instanceof HTMLElement)) {
      return;
    }
    const account = entry;
    openDropdown({
      anchor,
      value: account.folder_id ?? "",
      options: [
        { value: "", label: t.folder_none() },
        ...folders()
          .filter((folder) => !folder.system)
          .map((folder) => ({ value: folder.id, label: folder.name })),
      ],
      onSelect: (value) =>
        run("assign_folder", { steamId: account.steam_id, folderId: value || null }),
    });
  };

  const setRating = (): void => {
    if (!entry) {
      return;
    }
    run("set_rank", { steamId: entry.steam_id, template: "cs2-premier", rank: ratingDigits });
  };

  const openSteamGuard = (): void => {
    app.steamGuardAccountId = entry?.steam_id ?? null;
    app.steamGuardOpen = true;
  };
  const uid = $props.id();
</script>

{#if entry && !app.steamGuardOpen && !app.vaultOpen && !app.settingsOpen && !(app.report && trackingEnabled())}
  <div class="modal" inert={app.steamGuardOpen || app.vaultOpen}>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="modal-backdrop" onclick={close}></div>
    <div
      class="dialog account-dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby={`${uid}-title`}
      tabindex="-1"
    >
      <DialogHeader
        id={`${uid}-title`}
        title={t.account_details_title()}
        icon={UserRound}
        onclose={close}
      />
      <p class="dialog-text">
        <AccountCaption {entry} bold />
      </p>
      {#if !entry.is_local}<p class="settings-note">
          {t.guard_saved_account_status()}
        </p>{/if}

      <div class="account-body">
        <div class="account-facts">
          <div class="fact">
            <span class="fact-label">{t.account_login_name()}</span>
            <AccountIdentity {entry} />
          </div>
          <SteamIdentifiers steamId={entry.steam_id} />
          <div class="fact">
            <span class="fact-label">{t.account_last_login()}</span>
            <span class="fact-value">
              {relativeDate(entry.last_login_seconds)}
              {#if entry.last_login_seconds}({formatDate(entry.last_login_seconds)}){/if}
            </span>
          </div>
          {#if trackingEnabled()}
            <div class="fact has-action">
              <span class="fact-label">{t.playtime_local()}</span>
              <span class="fact-value accent">
                {formatSeconds(entry.playtime_seconds)} · {t.report_sessions_count({
                  count: entry.session_count,
                })}
              </span>
              <button
                class="fact-action"
                type="button"
                title={t.report_open_details()}
                aria-label={t.report_open_details()}
                onclick={() => openAccountReport(entry.steam_id)}
              >
                <ChartLine size={15} />
              </button>
            </div>
          {/if}
        </div>

        <div class="account-row account-overview">
          {#if entry.is_local}
            <section class="account-block">
              <h3 class="block-title"><Trophy size={15} /> {t.rank_label()}</h3>
              <div class="rank-controls">
                <button
                  class="select-button"
                  type="button"
                  aria-haspopup="listbox"
                  onclick={pickTemplate}
                  disabled={!entry.is_local}
                >
                  <span>{template?.label ?? t.rank_label()}</span>
                  <span class="picker-caret"><ChevronDown size={14} /></span>
                </button>

                {#if isRating}
                  <div class="rating-row">
                    <input
                      class="input rating-input"
                      type="number"
                      inputmode="numeric"
                      min="0"
                      max={template?.max_rating ?? 50000}
                      step="100"
                      aria-label={t.rank_premier_label()}
                      title={ratingTitle}
                      style={ratingColour ? `color:${ratingColour}` : ""}
                      bind:value={rating}
                      disabled={!entry.is_local}
                      onkeydown={(event) => {
                        if (event.key === "Enter") {
                          event.preventDefault();
                          setRating();
                        }
                      }}
                    />
                    <button
                      class="button primary"
                      type="button"
                      disabled={!entry.is_local}
                      onclick={setRating}><Check size={15} /> {t.rank_premier_set_action()}</button
                    >
                    <span class="rating-hint">{t.rank_premier_placeholder()}</span>
                  </div>
                {:else}
                  <button
                    class="rank-picker"
                    type="button"
                    aria-haspopup="listbox"
                    onclick={pickRank}
                    disabled={!entry.is_local}
                  >
                    {#if rankIcon}
                      <img class="rank-icon" src={rankIcon} alt="" />
                    {/if}
                    <span>{rankLabel(entry.rank)}</span>
                    <span class="picker-caret"><ChevronDown size={14} /></span>
                  </button>
                {/if}
              </div>
            </section>
          {/if}

          <section class="account-block steamguard-summary">
            <h3 class="block-title"><SteamGuardIcon size={17} /> {t.guard_title()}</h3>
            <p class="steamguard-note">
              {#if entry.has_authenticator}
                {app.snapshot?.vault.locked ? t.guard_status_locked() : t.guard_status_unlocked()}
              {:else if app.snapshot?.vault.locked}
                {app.snapshot?.vault.indexed
                  ? t.guard_status_unconfigured()
                  : t.guard_status_unlock_hint()}
              {:else}
                {t.guard_status_missing()}
              {/if}
            </p>
            <button class="button" type="button" onclick={openSteamGuard}>
              <SteamGuardIcon size={17} />
              {entry.has_authenticator ||
              (app.snapshot?.vault.locked && !app.snapshot?.vault.indexed)
                ? t.guard_open_action()
                : t.guard_setup()}
            </button>
          </section>
        </div>

        <div class="account-row">
          {#if entry.is_local}
            <section class="account-block">
              <h3 class="block-title"><Folder size={15} /> {t.folder_label()}</h3>
              <button
                class="select-button"
                type="button"
                aria-haspopup="listbox"
                onclick={pickFolder}
                disabled={!entry.is_local}
              >
                <span>{folderName}</span>
                <span class="picker-caret"><ChevronDown size={14} /></span>
              </button>
            </section>
          {/if}

          <section class="account-block">
            <h3 class="block-title"><Palette size={15} /> {t.account_colour()}</h3>
            <ColorSwatches
              value={entry.color}
              onselect={(color) =>
                run("set_account_color", { steamId: entry.steam_id, color: color || null })}
            />
          </section>
        </div>
      </div>

      <footer class="dialog-foot">
        <button class="button danger" type="button" onclick={() => removeAccount(entry.steam_id)}>
          <Trash2 size={15} />
          {entry.is_local ? t.common_remove() : t.guard_saved_delete()}
        </button>
        {#if trackingEnabled()}
          <ActionButton
            icon={Timer}
            label={t.playtime_title()}
            busyLabel={t.playtime_loading()}
            busy={app.reportLoadingId === entry.steam_id}
            disabled={Boolean(app.reportLoadingId) || !app.snapshot?.installations.length}
            onclick={() => openAccountReport(entry.steam_id)}
          />
        {/if}
        {#if entry.is_local}
          <button
            class="button primary"
            type="button"
            disabled={entry.is_current}
            onclick={() => switchAccount(entry.steam_id)}
          >
            <ArrowRightLeft size={15} />
            <span>{entry.is_current ? t.account_active_status() : t.account_switch_action()}</span>
          </button>
        {/if}
      </footer>
    </div>
  </div>
{/if}
