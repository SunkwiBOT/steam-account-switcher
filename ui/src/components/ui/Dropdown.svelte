<script lang="ts">
  import { Check } from "@lucide/svelte";
  import { dropdown, selectDropdown } from "../../lib/state/menus.svelte";
  import { nextMenuIndex } from "../../lib/state/keyboard";

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

  $effect(() => {
    if (!dropdown.open || !element) {
      return;
    }
    const selected = element.querySelector<HTMLElement>("[aria-selected='true']");
    (selected ?? element.querySelector<HTMLElement>("[role='option']"))?.focus();
  });

  const moveFocus = (event: KeyboardEvent): void => {
    if (!element) return;
    const options = [...element.querySelectorAll<HTMLElement>("[role='option']")];
    const next = nextMenuIndex(
      options.findIndex((option) => option === document.activeElement),
      options.length,
      event.key,
    );
    if (next === null) return;
    event.preventDefault();
    options[next]?.focus();
  };
</script>

{#if dropdown.open}
  <div
    class="context-menu dropdown-menu"
    role="listbox"
    bind:this={element}
    style="left:{position.left}px;top:{position.top}px"
    tabindex="-1"
    onkeydown={moveFocus}
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
        {#if option.icon}
          <span class="menu-icon">{@html option.icon}</span>
        {/if}
        {option.label}
        <span class="menu-check">
          {#if option.value === dropdown.value}<Check size={15} />{/if}
        </span>
      </button>
    {/each}
  </div>
{/if}
