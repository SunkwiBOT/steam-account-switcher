<script lang="ts">
  import DialogHeader from "../dialogs/DialogHeader.svelte";
  import { t, formatDate, relativeDate } from "../../lib/i18n/i18n.svelte";

  import { formatSeconds } from "../../lib/api/api";
  import ColorSwatches from "../folders/ColorSwatches.svelte";
  import { openAccountReport } from "../../lib/actions/reports.svelte";
  import { removeAccount, switchAccount } from "../../lib/actions/account-lifecycle.svelte";
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuRadioGroup,
    DropdownMenuRadioItem,
    DropdownMenuTrigger,
  } from "$lib/components/ui/dropdown-menu/index.js";
  import SelectSetting from "../ui/SelectSetting.svelte";
  import {
    formatRating,
    premierBand,
    rankIconUrl,
    UNRANKED,
    rankLabel,
  } from "../../lib/ranks/ranks";
  import { accountById, app, run, trackingEnabled } from "../../lib/state/state.svelte";
  import { folders } from "../../lib/state/folders.svelte";
  import type { AccountDto } from "../../lib/bindings/AccountDto";
  import type { RankTemplateDto } from "../../lib/bindings/RankTemplateDto";
  import ActionButton from "../ui/ActionButton.svelte";
  import AccountIdentity from "./AccountIdentity.svelte";
  import AccountCaption from "./AccountCaption.svelte";
  import AccountNotes from "./AccountNotes.svelte";
  import SteamGuardIcon from "../steam-guard/SteamGuardIcon.svelte";
  import SteamIdentifiers from "./SteamIdentifiers.svelte";
  import {
    ArrowRightLeft,
    ChartLine,
    Check,
    ChevronDown,
    Folder,
    NotebookPen,
    Palette,
    Timer,
    Trash2,
    Trophy,
    UserRound,
  } from "@lucide/svelte";
  import AppDialog from "../dialogs/AppDialog.svelte";
  import { DialogContent } from "$lib/components/ui/dialog/index.js";
  let rankOpen = $state(false);

  const entry = $derived<AccountDto | null>(
    app.openAccountId ? accountById(app.openAccountId) : null,
  );
  const templates = $derived<RankTemplateDto[]>(app.snapshot?.rank_templates ?? []);
  const template = $derived<RankTemplateDto | undefined>(
    templates.find((candidate) => candidate.id === entry?.rank_template) ?? templates[0],
  );
  const isRating = $derived(template?.kind === "RATING");
  const rankIcon = $derived(template && entry ? rankIconUrl(template.id, entry.rank_family) : null);
  const folderName = $derived(
    folders().find((folder) => folder.id === entry?.folder_id)?.name ?? t.folder_none(),
  );

  let rating = $state<string | number>("");
  let syncedRank = $state<string | null>(null);

  $effect(() => {
    const value = entry?.rank;
    if (value === undefined) return;
    const key = `${app.openAccountId}:${value}`;
    if (key === syncedRank) return;
    syncedRank = key;
    rating = value === UNRANKED ? "" : String(value).replace(/[^0-9]/g, "");
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

  const selectTemplate = (id: string): void => {
    if (!entry) return;
    void run("set_rank", { steamId: entry.steam_id, template: id, rank: entry.rank });
  };

  const selectRank = (rank: string): void => {
    if (!entry || !template) return;
    void run("set_rank", { steamId: entry.steam_id, template: template.id, rank });
  };

  const selectFolder = (folderId: string): void => {
    if (!entry) return;
    void run("assign_folder", { steamId: entry.steam_id, folderId: folderId || null });
  };

  const setRating = (): void => {
    if (!entry) {
      return;
    }
    void run("set_rank", { steamId: entry.steam_id, template: "cs2-premier", rank: ratingDigits });
  };

  const openSteamGuard = (): void => {
    app.steamGuardAccountId = entry?.steam_id ?? null;
    app.steamGuardOpen = true;
  };
  const uid = $props.id();
</script>

{#if entry && !app.steamGuardOpen && !app.vaultOpen && !app.settingsOpen && !(app.report && trackingEnabled())}
  <AppDialog open={true} onclose={close}>
    <DialogContent
      showCloseButton={false}
      class="sm:max-w-[1160px] max-h-[88vh] flex flex-col overflow-hidden"
      aria-labelledby={`${uid}-title`}
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
                <SelectSetting
                  value={template?.id ?? ""}
                  options={templates.map((item) => ({ value: item.id, label: item.label }))}
                  label={template?.label ?? t.rank_label()}
                  ariaLabel={t.rank_label()}
                  disabled={!entry.is_local}
                  onselect={selectTemplate}
                />

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
                  <DropdownMenu bind:open={rankOpen}>
                    <DropdownMenuTrigger
                      class="rank-picker"
                      aria-haspopup="listbox"
                      disabled={!entry.is_local}
                    >
                      {#if rankIcon}
                        <img class="rank-icon" src={rankIcon} alt="" />
                      {/if}
                      <span>{rankLabel(entry.rank)}</span>
                      <span class="picker-caret"><ChevronDown size={14} /></span>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent>
                      <DropdownMenuRadioGroup value={entry.rank} onValueChange={selectRank}>
                        {#each template?.ranks ?? [] as rank (rank.value)}
                          {@const icon = template ? rankIconUrl(template.id, rank.family) : null}
                          <DropdownMenuRadioItem value={rank.value}>
                            {#if icon}
                              <img
                                src={icon}
                                alt=""
                                style="width:18px;height:18px;object-fit:contain"
                              />
                            {/if}
                            {rankLabel(rank.value)}
                          </DropdownMenuRadioItem>
                        {/each}
                      </DropdownMenuRadioGroup>
                    </DropdownMenuContent>
                  </DropdownMenu>
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
              <SelectSetting
                value={entry.folder_id ?? ""}
                options={[
                  { value: "", label: t.folder_none() },
                  ...folders()
                    .filter((item) => !item.system)
                    .map((item) => ({ value: item.id, label: item.name })),
                ]}
                label={folderName}
                ariaLabel={t.folder_label()}
                disabled={!entry.is_local}
                onselect={selectFolder}
              />
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

        <section class="account-block">
          <h3 class="block-title"><NotebookPen size={15} /> {t.account_notes()}</h3>
          <AccountNotes steamId={entry.steam_id} note={entry.note} />
        </section>
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
          {@const alreadyActive = entry.is_current && app.snapshot?.steam_running === true}
          <button
            class="button primary"
            type="button"
            disabled={alreadyActive}
            onclick={() => switchAccount(entry.steam_id)}
          >
            <ArrowRightLeft size={15} />
            <span>
              {alreadyActive
                ? t.account_active_status()
                : entry.is_current
                  ? t.account_start_action()
                  : t.account_switch_action()}
            </span>
          </button>
        {/if}
      </footer>
    </DialogContent>
  </AppDialog>
{/if}
