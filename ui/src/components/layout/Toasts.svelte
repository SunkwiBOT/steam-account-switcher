<script lang="ts">
  import Sonner from "../ui/Sonner.svelte";
  import { t } from "../../lib/i18n/i18n.svelte";
  import { layout, TOAST_FOOTER_GAP } from "../../lib/state/layout.svelte";

  const offset = $derived({ bottom: layout.footerHeight + TOAST_FOOTER_GAP });
</script>

<Sonner
  position="bottom-right"
  visibleToasts={4}
  gap={TOAST_FOOTER_GAP}
  {offset}
  mobileOffset={offset}
  containerAriaLabel={t.toast_region_label()}
/>
