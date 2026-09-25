<script lang="ts">
  import { switchAccount } from "../lib/actions.svelte";
  import { openAccountMenu } from "../lib/menus.svelte";
  import { rankBadge } from "../lib/ranks";
  import { app } from "../lib/state.svelte";
  import Avatar from "./Avatar.svelte";
  import { Eye } from "@lucide/svelte";

  let { entry } = $props();

  const badge = $derived(rankBadge(entry));
  const switching = $derived(app.switchingId === entry.steam_id);
</script>

<div
  class="card"
  class:current={entry.is_current}
  class:switching
  role="button"
  tabindex="0"
  aria-label={entry.display_name}
  title="Hover for details · double click to switch · right click for actions"
  ondblclick={() => switchAccount(entry.steam_id)}
  oncontextmenu={(event) => openAccountMenu(event, entry.steam_id)}
  onkeydown={(event) => {
    if (event.key === "Enter") {
      app.openAccountId = entry.steam_id;
    } else if (event.key === " ") {
      event.preventDefault();
      switchAccount(entry.steam_id);
    }
  }}
>
  {#if entry.color}
    <span class="card-accent" style="--card-accent:{entry.color}"></span>
  {/if}
  <span class="card-badge">current</span>
  {#if switching}
    <span class="card-loader"><span class="spinner small"></span></span>
  {/if}
  <button
    class="card-view"
    type="button"
    tabindex="-1"
    title="View account details"
    aria-label="View account details"
    onclick={(event) => {
      event.stopPropagation();
      app.openAccountId = entry.steam_id;
    }}
  >
    <Eye size={15} />
  </button>
  <Avatar {entry} size={56} extra="card-avatar" />
  <span class="card-name">{entry.display_name}</span>
  <div class="card-meta">
    <span class="card-playtime" title={entry.playtime_tooltip}>{entry.playtime_text}</span>
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
