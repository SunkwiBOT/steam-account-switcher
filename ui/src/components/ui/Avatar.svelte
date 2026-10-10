<script lang="ts">
  import { CircleHelp } from "@lucide/svelte";
  import { hiddenIdentity } from "../../lib/accounts/identity";
  import type { AvatarSourceType } from "../../lib/types";

  type AvatarPropsType = { entry: AvatarSourceType; size?: number; extra?: string };

  let { entry, size = 68, extra = "" }: AvatarPropsType = $props();
  const hidden = $derived(hiddenIdentity({ persona_name: entry.persona_name ?? "" }));
</script>

{#if entry.avatar}
  <img
    class="avatar {extra}"
    style="width:{size}px;height:{size}px"
    src={entry.avatar}
    alt=""
    draggable="false"
  />
{:else}
  <span class="avatar initial {extra}" aria-hidden="true" style="width:{size}px;height:{size}px">
    {#if hidden}<CircleHelp size={Math.round(size * 0.5)} />{:else}{entry.initial || "?"}{/if}
  </span>
{/if}
