<script lang="ts">
  import { ShieldCheck } from "@lucide/svelte";
  type SteamGuardIconPropsType = { size?: number };

  let { size = 16 }: SteamGuardIconPropsType = $props();
</script>

<ShieldCheck {size} />
