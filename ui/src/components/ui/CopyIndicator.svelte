<script lang="ts">
  import AsyncIcon from "./AsyncIcon.svelte";
  import { Check, Copy } from "@lucide/svelte";
  import SteamGuardIcon from "../steam-guard/SteamGuardIcon.svelte";
  type CopyIndicatorPropsType = {
    copied?: boolean;
    pending?: boolean;
    guard?: boolean;
    size?: number;
  };

  let {
    copied = false,
    pending = false,
    guard = false,
    size = 15,
  }: CopyIndicatorPropsType = $props();
</script>

{#if pending}
  <span class="copy-indicator" aria-hidden="true"><AsyncIcon {size} /></span>
{:else}
  <span class="copy-indicator" class:copied aria-hidden="true">
    <span class="copy-default"
      >{#if guard}<SteamGuardIcon {size} />{:else}<Copy {size} />{/if}</span
    >
    <span class="copy-success"><Check {size} /></span>
  </span>
{/if}
