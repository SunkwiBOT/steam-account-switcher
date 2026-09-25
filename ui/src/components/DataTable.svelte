<script lang="ts">
  import Avatar from "./Avatar.svelte";

  let { headers, rows, empty } = $props();
</script>

{#if rows.length === 0}
  <p class="empty-tab">{empty}</p>
{:else}
  <table class="data-table">
    <thead>
      <tr>
        {#each headers as header}<th>{header}</th>{/each}
      </tr>
    </thead>
    <tbody>
      {#each rows as cells}
        <tr>
          {#each cells as cell}
            <td title={cell.title ?? cell.text}>
              {#if cell.avatar}
                <span class="account-cell">
                  <Avatar entry={cell.avatar} size={24} />
                  {cell.text}
                </span>
              {:else}
                {cell.text}
              {/if}
            </td>
          {/each}
        </tr>
      {/each}
    </tbody>
  </table>
{/if}
