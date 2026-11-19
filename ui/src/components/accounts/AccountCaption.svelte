<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";
  import { cn } from "$lib/utils";
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
  const avatarExtra = $derived(
    cn(
      "shrink-0 text-[12px] outline outline-1 outline-[var(--avatar-outline)]",
      inline && "absolute top-1/2 left-0 -translate-y-1/2",
    ),
  );
  const rootClasses = $derived(
    inline
      ? "inline-flex relative items-center max-w-full align-baseline gap-1 min-h-[var(--avatar-size)] pl-[calc(var(--avatar-size)+4px)] [font-size:inherit] [color:inherit]"
      : "flex items-center gap-1 mt-[9px] text-[13px] text-[var(--muted)]",
  );
</script>

<span class={rootClasses} style:--avatar-size={`${size}px`}
  ><Avatar {entry} {size} extra={avatarExtra} /><span
    title={hidden ? undefined : entry.display_name}
    class={cn("overflow-hidden text-ellipsis whitespace-nowrap", hidden && "blur-[5px]")}
    aria-hidden={hidden || undefined}
    >{#if bold}<strong>{entry.display_name}</strong>{:else}{entry.display_name}{/if}</span
  >{#if hidden}<span class="sr-only">{t.common_hidden_value()}</span>{/if}</span
>
