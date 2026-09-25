<script lang="ts">
  import { dropdown, selectDropdown } from "../lib/menus.svelte";

  let element = $state<HTMLDivElement | null>(null);
  let position = $state({ left: 0, top: 0 });

  $effect(() => {
    if (!dropdown.open || !element) {
      return;
    }
    const width = element.offsetWidth;
    const height = element.offsetHeight;
    const left = Math.max(12, Math.min(dropdown.x, window.innerWidth - width - 12));
    const below = dropdown.y;
    const top =
      below + height > window.innerHeight - 12
        ? Math.max(12, dropdown.anchorTop - height - 4)
        : below;
    position = { left, top };
  });
</script>

{#if dropdown.open}
  <div
    class="context-menu dropdown-menu"
    role="listbox"
    bind:this={element}
    style="left:{position.left}px;top:{position.top}px"
  >
    {#each dropdown.options as option (option.value)}
      <button
        type="button"
        class="menu-item"
        class:checked={option.value === dropdown.value}
        role="option"
        aria-selected={option.value === dropdown.value}
        onclick={() => selectDropdown(option.value)}
      >
        <span class="menu-check">{option.value === dropdown.value ? "✓" : ""}</span>
        {#if option.icon}
          <span class="menu-icon">{@html option.icon}</span>
        {/if}
        {option.label}
      </button>
    {/each}
  </div>
{/if}
