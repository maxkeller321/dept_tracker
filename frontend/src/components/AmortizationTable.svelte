<script lang="ts">
  import type { AmortizationRow } from '../lib/api/client';
  import { formatMoney } from '../lib/format';
  import { t } from '../lib/i18n';

  export let rows: AmortizationRow[];
  export let currency: string;
  export let totalPayments: number;

  const PAGE_SIZE = 12;

  $: tr = $t;
  $: pageCount = Math.ceil(rows.length / PAGE_SIZE);

  let page = 0;

  // Reset to first page whenever the row data changes (e.g. after a payment).
  $: if (rows) page = 0;

  $: pagedRows = rows.slice(page * PAGE_SIZE, (page + 1) * PAGE_SIZE);

  function prevPage() {
    page = Math.max(0, page - 1);
  }
  function nextPage() {
    page = Math.min(pageCount - 1, page + 1);
  }

  /** Short date label: "Jun '26" */
  function shortDate(iso: string): string {
    const d = new Date(iso + 'T00:00:00');
    return d.toLocaleDateString(undefined, { month: 'short', year: '2-digit' });
  }

  /** Interest share as a percentage string for the mini pie (e.g. for aria). */
  function interestPct(row: AmortizationRow): number {
    if (row.payment_minor <= 0) return 0;
    return Math.round((row.interest_minor / row.payment_minor) * 100);
  }
</script>

<div class="amort-table-wrap">
  <div class="amort-meta">
    <span>{tr('amort.paymentsRemaining', { n: totalPayments })}</span>
    {#if pageCount > 1}
      <div class="amort-pagination">
        <button
          type="button"
          class="amort-page-btn"
          disabled={page === 0}
          on:click={prevPage}
          aria-label={tr('amort.prevYear')}
        >‹</button>
        <span class="amort-page-label">
          {tr('amort.yearOf', { current: page + 1, total: pageCount })}
        </span>
        <button
          type="button"
          class="amort-page-btn"
          disabled={page >= pageCount - 1}
          on:click={nextPage}
          aria-label={tr('amort.nextYear')}
        >›</button>
      </div>
    {/if}
  </div>

  <table class="amort-table" aria-label={tr('amort.tableLabel')}>
    <thead>
      <tr>
        <th class="col-date">{tr('amort.colDate')}</th>
        <th class="col-num col-interest">{tr('amort.colInterest')}</th>
        <th class="col-num col-principal">{tr('amort.colPrincipal')}</th>
        <th class="col-num col-payment">{tr('amort.colPayment')}</th>
        <th class="col-num col-balance">{tr('amort.colBalance')}</th>
      </tr>
    </thead>
    <tbody>
      {#each pagedRows as row (row.date)}
        {@const pct = interestPct(row)}
        <tr class="amort-row" title="{pct}% Zinsen">
          <td class="col-date">{shortDate(row.date)}</td>
          <td class="col-num col-interest amort-interest">
            {formatMoney(row.interest_minor, currency)}
          </td>
          <td class="col-num col-principal amort-principal">
            {formatMoney(row.principal_minor, currency)}
          </td>
          <td class="col-num col-payment amort-payment">
            {formatMoney(row.payment_minor, currency)}
          </td>
          <td class="col-num col-balance amort-balance">
            {formatMoney(row.balance_minor, currency)}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>

  {#if rows.length === 0}
    <p class="amort-empty">{tr('amort.empty')}</p>
  {/if}
</div>

<style>
  .amort-table-wrap {
    margin-top: 0.75rem;
    overflow-x: auto;
    -webkit-overflow-scrolling: touch;
  }

  .amort-meta {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 0.75rem;
    color: var(--color-text-muted);
    margin-bottom: 0.4rem;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .amort-pagination {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .amort-page-btn {
    background: none;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 0.1rem 0.4rem;
    font-size: 0.875rem;
    cursor: pointer;
    color: var(--color-text-muted);
    line-height: 1.4;
    transition: background 0.15s, color 0.15s;
  }
  .amort-page-btn:hover:not(:disabled) {
    background: var(--color-accent-soft);
    color: var(--color-primary);
  }
  .amort-page-btn:disabled {
    opacity: 0.35;
    cursor: default;
  }

  .amort-page-label {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    min-width: 4.5rem;
    text-align: center;
  }

  .amort-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.78rem;
    font-variant-numeric: tabular-nums;
  }

  .amort-table thead tr {
    border-bottom: 1px solid var(--color-border);
  }

  .amort-table th {
    font-weight: 600;
    font-size: 0.7rem;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.03em;
    padding: 0.25rem 0.4rem;
    white-space: nowrap;
  }

  .amort-row {
    border-bottom: 1px solid color-mix(in srgb, var(--color-border) 50%, transparent);
    transition: background 0.1s;
  }
  .amort-row:last-child {
    border-bottom: none;
  }
  .amort-row:hover {
    background: var(--color-accent-soft);
  }

  .amort-table td {
    padding: 0.25rem 0.4rem;
    vertical-align: middle;
    white-space: nowrap;
  }

  .col-date {
    color: var(--color-text-muted);
    font-size: 0.75rem;
    min-width: 3.5rem;
  }

  .col-num {
    text-align: right;
  }

  .amort-interest {
    color: var(--color-danger, #e55);
    opacity: 0.85;
  }

  .amort-principal {
    color: var(--color-accent);
  }

  .amort-balance {
    font-weight: 500;
    color: var(--color-text);
  }

  .amort-payment {
    font-weight: 500;
  }

  .amort-empty {
    text-align: center;
    color: var(--color-text-muted);
    font-size: 0.85rem;
    padding: 1rem 0;
  }
</style>
