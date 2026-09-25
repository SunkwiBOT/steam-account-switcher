<script lang="ts">
  import { CARD_COLORS } from "../lib/api";
  import {
    copySteamId,
    openAccountReport,
    removeAccount,
    switchAccount,
  } from "../lib/actions.svelte";
  import { openDropdown } from "../lib/menus.svelte";
  import { formatRating, premierBand, rankIconUrl, UNRANKED } from "../lib/ranks";
  import { accountById, app, folders, run } from "../lib/state.svelte";
  import type { Account, RankTemplate } from "../lib/types";
  import Avatar from "./Avatar.svelte";
  import { ArrowRight, ChevronDown, Copy, Timer, Trash2, X } from "@lucide/svelte";

  const entry = $derived<Account | null>(app.openAccountId ? accountById(app.openAccountId) : null);
  const templates = $derived<RankTemplate[]>(app.snapshot?.rank_templates ?? []);
  const template = $derived<RankTemplate | undefined>(
    templates.find((candidate) => candidate.id === entry?.rank_template) ?? templates[0],
  );
  const isRating = $derived(template?.kind === "rating");
  const rankIcon = $derived(template && entry ? rankIconUrl(template.id, entry.rank_family) : null);
  const folderName = $derived(
    folders().find((folder) => folder.id === entry?.folder_id)?.name ?? "No folder",
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
    ratingDigits ? `${formatRating(ratingDigits)} · ${premierBand(ratingDigits).name}` : "Unranked",
  );

  function close(): void {
    app.openAccountId = null;
  }

  function pickTemplate(event: MouseEvent): void {
    if (!entry) {
      return;
    }
    const account = entry;
    openDropdown({
      anchor: event.currentTarget as HTMLElement,
      value: account.rank_template,
      options: templates.map((item) => ({ value: item.id, label: item.label })),
      onSelect: (id) =>
        run("set_rank", { steamId: account.steam_id, template: id, rank: account.rank }),
    });
  }

  function pickRank(event: MouseEvent): void {
    if (!template || !entry) {
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
      anchor: event.currentTarget as HTMLElement,
      value: account.rank,
      options: ladder.ranks.map((rank) => ({
        value: rank.value,
        label: rank.value,
        icon: iconFor(rank.family),
      })),
      onSelect: (rank) => run("set_rank", { steamId: account.steam_id, template: ladder.id, rank }),
    });
  }

  function pickFolder(event: MouseEvent): void {
    if (!entry) {
      return;
    }
    const account = entry;
    openDropdown({
      anchor: event.currentTarget as HTMLElement,
      value: account.folder_id ?? "",
      options: [
        { value: "", label: "No folder" },
        ...folders().map((folder) => ({ value: folder.id, label: folder.name })),
      ],
      onSelect: (value) =>
        run("assign_folder", { steamId: account.steam_id, folderId: value || null }),
    });
  }

  function setRating(): void {
    if (!entry) {
      return;
    }
    run("set_rank", { steamId: entry.steam_id, template: "cs2-premier", rank: ratingDigits });
  }
</script>

{#if entry}
  <div class="modal">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="modal-backdrop" onclick={close}></div>
    <section class="dialog account-dialog" role="dialog" aria-modal="true">
      <header class="account-head">
        <div class="account-head-avatar"><Avatar {entry} size={56} /></div>
        <div class="account-head-text">
          <h2 class="dialog-title">{entry.display_name}</h2>
          <p class="dialog-subtitle">
            {entry.is_current ? "Currently selected in Steam" : "Steam account"}
          </p>
        </div>
        <button class="icon-button" type="button" aria-label="Close" onclick={close}>
          <X size={16} />
        </button>
      </header>

      <div class="account-body">
        <div class="account-facts">
          <div class="fact">
            <span class="fact-label">Account Name</span>
            <span class="fact-value">{entry.account_name}</span>
          </div>
          <div class="fact">
            <span class="fact-label">Steam ID 64</span>
            <span class="fact-row">
              <span class="fact-value">
                {app.snapshot?.steam_ids_visible ? entry.steam_id : "******"}
              </span>
              <button
                class="copy-button"
                type="button"
                title="Copy Steam ID 64"
                aria-label="Copy Steam ID 64"
                onclick={() => copySteamId(entry.steam_id)}
              >
                <Copy size={15} />
              </button>
            </span>
          </div>
          <div class="fact">
            <span class="fact-label">Last login</span>
            <span class="fact-value">{entry.last_login_text} ({entry.last_login_absolute})</span>
          </div>
          <div
            class="fact clickable"
            title="Open the full breakdown"
            role="button"
            tabindex="0"
            onclick={() => openAccountReport(entry.steam_id)}
            onkeydown={(event) => event.key === "Enter" && openAccountReport(entry.steam_id)}
          >
            <span class="fact-label">Local playtime</span>
            <span class="fact-value accent">
              {entry.playtime_text} · {entry.session_count} session(s)
            </span>
          </div>
        </div>

        <section class="account-block">
          <h3 class="block-title">Rank</h3>
          <div class="rank-controls">
            <button
              class="select-button"
              type="button"
              aria-haspopup="listbox"
              onclick={pickTemplate}
            >
              <span>{template?.label ?? "Rank"}</span>
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
                  placeholder="0 – 50 000"
                  aria-label="CS2 Premier rating"
                  title={ratingTitle}
                  style={ratingColour ? `color:${ratingColour}` : ""}
                  bind:value={rating}
                  onkeydown={(event) => {
                    if (event.key === "Enter") {
                      event.preventDefault();
                      setRating();
                    }
                  }}
                />
                <button class="button primary" type="button" onclick={setRating}>Set</button>
                <span class="rating-hint">Premier rating, 0 to 50 000</span>
              </div>
            {:else}
              <button class="rank-picker" type="button" aria-haspopup="listbox" onclick={pickRank}>
                {#if rankIcon}
                  <img class="rank-icon" src={rankIcon} alt="" />
                {/if}
                <span>{entry.rank}</span>
                <span class="picker-caret"><ChevronDown size={14} /></span>
              </button>
            {/if}
          </div>
        </section>

        <div class="account-row">
          <section class="account-block">
            <h3 class="block-title">Folder</h3>
            <button
              class="select-button"
              type="button"
              aria-haspopup="listbox"
              onclick={pickFolder}
            >
              <span>{folderName}</span>
              <span class="picker-caret"><ChevronDown size={14} /></span>
            </button>
          </section>

          <section class="account-block">
            <h3 class="block-title">Card colour</h3>
            <div class="color-grid">
              {#each CARD_COLORS as color (color)}
                <button
                  class="color-swatch"
                  class:none={!color}
                  class:active={(entry.color ?? "") === color}
                  type="button"
                  title={color || "No colour"}
                  style={color ? `background:${color}` : ""}
                  onclick={() =>
                    run("set_account_color", { steamId: entry.steam_id, color: color || null })}
                >
                  {#if !color}<X size={14} />{/if}
                </button>
              {/each}
            </div>
          </section>
        </div>
      </div>

      <footer class="dialog-foot">
        <button class="button danger" type="button" onclick={() => removeAccount(entry.steam_id)}>
          <Trash2 size={15} /> Remove
        </button>
        <button class="button" type="button" onclick={() => openAccountReport(entry.steam_id)}>
          <Timer size={15} /> Playtime
        </button>
        <button
          class="button primary"
          type="button"
          disabled={entry.is_current}
          onclick={() => switchAccount(entry.steam_id)}
        >
          <ArrowRight size={15} />
          <span>{entry.is_current ? "Already active" : "Switch"}</span>
        </button>
      </footer>
    </section>
  </div>
{/if}
