<script lang="ts">
  import { folderLabel, t } from "../../lib/i18n/i18n.svelte";

  import { cn } from "$lib/utils";
  import { formatSeconds } from "../../lib/api/api";
  import { folderIcon } from "../../lib/folders/folder-style";
  import { playtimeTooltip } from "../../lib/playtime/reports";
  import { copySteamGuardCode } from "../../lib/actions/account-copy.svelte";
  import { switchAccount } from "../../lib/actions/account-lifecycle.svelte";
  import AccountContextMenu from "./AccountContextMenu.svelte";
  import { rankBadge } from "../../lib/ranks/ranks";
  import { app, trackingEnabled } from "../../lib/state/state.svelte";
  import { folderById } from "../../lib/state/folders.svelte";
  import { hiddenIdentity } from "../../lib/accounts/identity";
  import Avatar from "../ui/Avatar.svelte";
  import CopyButton from "../ui/CopyButton.svelte";
  import SteamGuardIcon from "../steam-guard/SteamGuardIcon.svelte";
  import { Eye } from "@lucide/svelte";
  import type { AccountDto } from "../../lib/bindings/AccountDto";

  type AccountCardPropsType = { entry: AccountDto };

  let { entry }: AccountCardPropsType = $props();

  const badge = $derived(rankBadge(entry));
  const showPlaytime = $derived(trackingEnabled());
  const hidden = $derived(hiddenIdentity(entry));
  const switching = $derived(app.switchingId === entry.steam_id);
  const locked = $derived(Boolean(app.switchingId) && !switching);
  const folder = $derived(
    app.folderId === null && entry.folder_id ? folderById(entry.folder_id) : null,
  );
  const Glyph = $derived(folderIcon(folder?.icon));
  const folderStyle = $derived(
    folder?.color
      ? `color:${folder.color};border-color:${folder.color};background:${folder.color}33`
      : "",
  );
  const rankOnly = $derived(!showPlaytime && Boolean(badge.text));
  const viewButtonClasses =
    "absolute top-2 right-2 inline-flex size-[25px] items-center justify-center rounded-full border border-[var(--border-strong)] bg-[var(--panel)] p-0 text-[var(--text)] opacity-0 transition-opacity duration-[120ms] group-hover:opacity-100 group-focus-within:opacity-100 focus-visible:opacity-100 hover:border-[var(--accent)] hover:bg-[var(--button)] hover:text-[var(--text-bright)] cursor-pointer";
</script>

<AccountContextMenu {entry}>
  {#snippet children({ props })}
    <!-- ! A group, not a button, because it holds buttons of its own. -->
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      {...props}
      class={cn(
        "group relative flex h-full min-h-[146px] cursor-pointer flex-col items-center gap-1.5 overflow-hidden rounded-xl border bg-[var(--base-alt)] px-2.5 pt-2 pb-1 text-center text-inherit transition-[transform,border-color,background-color] duration-[120ms] ease-out hover:z-[2] hover:bg-[var(--surface-hover)] [&_img]:pointer-events-none [&_img]:[-webkit-user-drag:none] [&_img]:select-none [&_svg]:pointer-events-none [&_svg]:select-none",
        entry.is_current
          ? "border-[var(--current)] shadow-[inset_0_0_0_1px_var(--current),0_0_14px_rgba(31,122,63,0.35)]"
          : "border-[var(--border)] hover:border-[var(--border-strong)]",
        switching
          ? "opacity-[0.65] pointer-events-none"
          : locked
            ? "opacity-[0.45] pointer-events-none"
            : "",
      )}
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
      onkeydown={(event) => {
        if (event.target !== event.currentTarget || event.key !== "Enter") return;
        event.preventDefault();
        app.openAccountId = entry.steam_id;
      }}
    >
      {#if entry.color}
        <span class="absolute inset-y-0 left-0 w-1" style="background:{entry.color}"></span>
      {/if}
      {#if folder}
        <span
          class="absolute top-1.5 left-2 inline-flex size-[22px] items-center justify-center rounded-full border border-[var(--border-strong)] bg-[var(--panel)] text-[var(--accent)] [&_svg]:size-3.5"
          style={folderStyle}
          title={folderLabel(folder)}
          aria-hidden="true"
        >
          {#if folder.system}<SteamGuardIcon size={14} />{:else}<Glyph size={13} />{/if}
        </span>
      {/if}
      <span
        class={cn(
          "absolute top-2 rounded-[8px] bg-[var(--current)] px-[7px] py-px text-[10px] font-bold text-[var(--text-bright)] uppercase",
          folder ? "left-[38px]" : "left-2.5",
          entry.is_current ? "inline-block" : "hidden",
        )}>{t.account_current()}</span
      >
      {#if switching}
        <span
          class="absolute inset-0 flex items-center justify-center rounded-[inherit] bg-[rgba(8,12,18,0.45)]"
          ><span class="spinner size-[22px] border-2"></span></span
        >
      {/if}
      {#if entry.has_authenticator}
        <CopyButton
          className={cn(viewButtonClasses, "top-[40px]")}
          copyKey={`guard:${entry.steam_id}`}
          label={t.guard_code_copy()}
          title={app.snapshot?.vault.locked ? t.guard_code_unlock_copy() : t.guard_code_copy()}
          guard
          action={() => copySteamGuardCode(entry.steam_id)}
        />
      {/if}

      <button
        class={viewButtonClasses}
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
      <Avatar {entry} size={56} extra="text-[22px]" />
      <span
        class={cn(
          "w-full truncate text-center font-semibold text-[var(--text-strong)]",
          hidden && "blur-[5px]",
        )}
        aria-hidden={hidden || undefined}>{entry.display_name}</span
      >
      <div
        class="mt-auto flex h-[30px] w-full items-center gap-1.5 text-[11px] text-[var(--muted)]"
      >
        {#if showPlaytime}<span
            class="text-[var(--accent)] whitespace-nowrap"
            title={playtimeTooltip(entry)}>{formatSeconds(entry.playtime_seconds)}</span
          >{/if}
        {#if badge.text}
          <span
            class={cn(
              "flex min-w-0 items-center justify-end gap-0.5 text-[11px] whitespace-nowrap text-[var(--text)]",
              rankOnly ? "mx-auto flex-initial justify-center" : "ml-auto flex-auto",
            )}
            title={badge.text}
          >
            {#if badge.icon}
              <img class="size-[30px] object-contain" src={badge.icon} alt="" />
            {/if}
            <span
              class="overflow-hidden text-ellipsis"
              style={badge.colour ? `color:${badge.colour}` : ""}>{badge.short}</span
            >
          </span>
        {/if}
      </div>
    </div>
  {/snippet}
</AccountContextMenu>
