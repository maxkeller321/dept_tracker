<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api/client';
  import { formatDate, formatMoney } from '../lib/format';
  import { t } from '../lib/i18n';

  export let loanId: string;

  let events: Array<{
    id: string;
    event_type: string;
    amount_minor: number;
    paid_at: string;
  }> = [];
  let loading = true;

  $: tr = $t;

  onMount(async () => {
    try {
      events = (await api.listPayments(loanId)) as typeof events;
    } finally {
      loading = false;
    }
  });
</script>

<section class="panel history" aria-label={tr('history.regionLabel')}>
  <h3>{tr('history.title')}</h3>
  {#if loading}
    <p class="muted">{tr('history.loading')}</p>
  {:else if events.length === 0}
    <p class="muted">{tr('history.empty')}</p>
  {:else}
    <ul class="data-list">
      {#each events as ev (ev.id)}
        <li>
          <span class="badge" class:badge-extra={ev.event_type === 'sonderzahlung'}
            >{ev.event_type === 'sonderzahlung' ? tr('history.extra') : tr('history.regular')}</span
          >
          <span class="tabular">{formatMoney(ev.amount_minor)}</span>
          <span class="muted tabular">{formatDate(ev.paid_at)}</span>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .data-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .data-list li {
    display: grid;
    grid-template-columns: 1fr auto auto;
    gap: 0.75rem;
    align-items: center;
    padding: 0.5rem 0;
    border-bottom: 1px solid var(--color-border);
    font-size: 0.875rem;
  }
  .data-list li:last-child {
    border-bottom: none;
  }
  .badge {
    font-size: 0.7rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    padding: 0.2rem 0.45rem;
    border-radius: 4px;
    background: var(--color-primary-soft);
    color: var(--color-primary);
    width: fit-content;
  }
  .badge-extra {
    background: var(--color-accent-soft);
    color: var(--color-accent);
  }
</style>
