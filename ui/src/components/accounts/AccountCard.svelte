<script lang="ts">
  import { folderLabel, t } from "../../lib/i18n/i18n.svelte";

  import { formatSeconds } from "../../lib/api/api";
  import { folderIcon } from "../../lib/folders/folder-style";
  import { playtimeTooltip } from "../../lib/playtime/reports";
  import { copySteamGuardCode, switchAccount } from "../../lib/actions/actions.svelte";
  import { openAccountMenu } from "../../lib/state/menus.svelte";
  import { rankBadge } from "../../lib/ranks/ranks";
  import { app, folderById, trackingEnabled } from "../../lib/state/state.svelte";
  import { hiddenIdentity } from "../../lib/accounts/identity";
  import Avatar from "../ui/Avatar.svelte";
  import CopyButton from "../ui/CopyButton.svelte";
  import SteamGuardIcon from "../steam-guard/SteamGuardIcon.svelte";
  import { Eye } from "@lucide/svelte";
  import type { AccountType } from "../../lib/types";

  type AccountCardPropsType = { entry: AccountType };

  let { entry }: AccountCardPropsType = $props();

  const badge = $derived(rankBadge(entry));
  const showPlaytime = $derived(trackingEnabled());
  const hidden = $derived(hiddenIdentity(entry));
  const switching = $derived(app.switchingId === entry.steam_id);
  /// The all-accounts list mixes folder contents while searching, so a card has
  /// to say which folder it comes from; inside a folder every card shares it.
  const folder = $derived(
    app.folderId === null && entry.folder_id ? folderById(entry.folder_id) : null,
  );
  const Glyph = $derived(folderIcon(folder?.icon));
  const folderStyle = $derived(
    folder?.color
      ? `color:${folder.color};border-color:${folder.color};background:${folder.color}33`
      : "",
  );
</script>

<!-- A group rather than a button: it holds buttons of its own. Enter opens the
     details; switching stays on double-click and in the context menu. -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="card"
  class:current={entry.is_current}
  class:has-folder={Boolean(folder)}
  class:switching
  role="group"
  tabindex="0"
  aria-label={hidden ? t.common_hidden_value() : entry.display_name}
  title={entry.is_local ? t.account_card_tooltip() : t.guard_saved_card_tooltip()}
  onclick={() => {
    if (!entry.is_local) {
      app.steamGuardAccountId = entry.steam_id;
      app.steamGuardOpen = true;
    }
  }}
  ondblclick={() => {
    if (entry.is_local) void switchAccount(entry.steam_id);
  }}
  oncontextmenu={(event) => openAccountMenu(event, entry.steam_id)}
  onkeydown={(event) => {
    if (event.target !== event.currentTarget || event.key !== "Enter") return;
    event.preventDefault();
    app.openAccountId = entry.steam_id;
  }}
>
  {#if entry.color}
    <span class="card-accent" style="--card-accent:{entry.color}"></span>
  {/if}
  {#if folder}
    <span class="card-folder" style={folderStyle} title={folderLabel(folder)} aria-hidden="true">
      {#if folder.system}<SteamGuardIcon size={14} />{:else}<Glyph size={13} />{/if}
    </span>
  {/if}
  <span class="card-badge">{t.account_current()}</span>
  {#if switching}
    <span class="card-loader"><span class="spinner small"></span></span>
  {/if}
  {#if entry.has_authenticator}
    <CopyButton
      className="card-view card-copy"
      copyKey={`guard:${entry.steam_id}`}
      label={t.guard_code_copy()}
      title={app.snapshot?.vault.locked ? t.guard_code_unlock_copy() : t.guard_code_copy()}
      guard
      action={() => copySteamGuardCode(entry.steam_id)}
    />
  {/if}

  <button
    class="card-view"
    type="button"
    title={t.account_details_open()}
    aria-label={t.account_details_open()}
    onclick={(event) => {
      event.stopPropagation();
      app.openAccountId = entry.steam_id;
    }}
    ondblclick={(event) => event.stopPropagation()}
  >
    <Eye size={15} />
  </button>
  <Avatar {entry} size={56} extra="card-avatar" />
  <span class="card-name" class:hidden-name={hidden} aria-hidden={hidden || undefined}
    >{entry.display_name}</span
  >
  <div class="card-meta" class:rank-only={!showPlaytime && Boolean(badge.text)}>
    {#if showPlaytime}<span class="card-playtime" title={playtimeTooltip(entry)}
        >{formatSeconds(entry.playtime_seconds)}</span
      >{/if}
    {#if badge.text}
      <span class="card-rank" title={badge.text}>
        {#if badge.icon}
          <img class="card-rank-icon" src={badge.icon} alt="" />
        {/if}
        <span style={badge.colour ? `color:${badge.colour}` : ""}>{badge.text}</span>
      </span>
    {/if}
  </div>
</div>
