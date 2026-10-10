<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";
  import { hiddenIdentity } from "../../lib/accounts/identity";
  import type { ToastAccountType } from "../../lib/types";
  import Avatar from "../ui/Avatar.svelte";
  type AccountCaptionPropsType = {
    entry: ToastAccountType;
    size?: number;
    inline?: boolean;
    bold?: boolean;
  };

  let { entry, size = 28, inline = false, bold = false }: AccountCaptionPropsType = $props();
  const hidden = $derived(hiddenIdentity(entry));
</script>

<span class="account-caption" class:inline style:--avatar-size={`${size}px`}
  ><Avatar {entry} {size} /><span
    title={hidden ? undefined : entry.display_name}
    class:hidden-name={hidden}
    aria-hidden={hidden || undefined}
    >{#if bold}<strong>{entry.display_name}</strong>{:else}{entry.display_name}{/if}</span
  >{#if hidden}<span class="sr-only">{t.common_hidden_value()}</span>{/if}</span
>
