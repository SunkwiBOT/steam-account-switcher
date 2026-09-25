<script lang="ts">
  let { entry, size = 68, extra = "" } = $props();
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
  <span class="avatar initial {extra}" style="width:{size}px;height:{size}px">
    {entry.initial || "?"}
  </span>
{/if}
