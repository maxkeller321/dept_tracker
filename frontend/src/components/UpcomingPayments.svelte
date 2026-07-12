<script lang="ts">
  import { formatDate, formatMoney } from '../lib/format';
  import { t } from '../lib/i18n';

  export let items: Array<{
    id: string;
    amount_minor: number;
    due_date: string;
    status: string;
  }> = [];

  export let onCancel: ((id: string) => void) | undefined = undefined;

  $: tr = $t;
</script>

{#if items.length > 0}
  <section class="panel upcoming">
    <h3>{tr('upcoming.title')}</h3>
    <ul class="data-list">
      {#each items as item (item.id)}
        <li>
          <span class="tabular"
            >{formatMoney(item.amount_minor)} {tr('upcoming.on')} {formatDate(item.due_date)}</span
          >
          {#if onCancel && item.status === 'pending'}
            <button type="button" class="secondary" on:click={() => onCancel(item.id)}>{tr('upcoming.cancel')}</button>
          {/if}
        </li>
      {/each}
    </ul>
  </section>
{/if}

<style>
  .data-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .data-list li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem 0;
    border-bottom: 1px solid var(--color-border);
    font-size: 0.875rem;
  }
  .data-list li:last-child {
    border-bottom: none;
  }
</style>
