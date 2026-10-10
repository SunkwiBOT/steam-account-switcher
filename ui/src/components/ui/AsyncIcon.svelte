<script lang="ts">
  import { LoaderCircle } from "@lucide/svelte";
  type AsyncIconPropsType = {
    icon?: typeof LoaderCircle;
    busy?: boolean;
    size?: number;
  };

  let { icon: Icon = LoaderCircle, busy = true, size = 15 }: AsyncIconPropsType = $props();
</script>

<!-- Rotate an HTML wrapper; SVG transforms behave differently across webviews. -->
<span class="action-icon" class:icon-spin={busy} aria-hidden="true"
  >{#if busy}<LoaderCircle {size} />{:else}<Icon {size} />{/if}</span
>
