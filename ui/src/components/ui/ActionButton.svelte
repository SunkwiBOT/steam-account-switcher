<script lang="ts">
  import type { LoaderCircle } from "@lucide/svelte";
  import AsyncIcon from "./AsyncIcon.svelte";

  type ActionButtonPropsType = {
    icon: typeof LoaderCircle;
    label: string;
    busyLabel: string;
    busy?: boolean;
    disabled?: boolean;
    className?: string;
    size?: number;
    type?: "button" | "submit";
    onclick?: (event: MouseEvent) => unknown;
  };

  let {
    icon,
    label,
    busyLabel,
    busy = false,
    disabled = false,
    className = "button",
    size = 15,
    type = "button",
    onclick,
  }: ActionButtonPropsType = $props();
</script>

<button class={className} {type} disabled={disabled || busy} aria-busy={busy} {onclick}>
  <AsyncIcon {icon} {size} {busy} />
  {busy ? busyLabel : label}
</button>
