<script lang="ts">
  import type { LoanSummary } from '../lib/api/client';
  import { formatMoney, formatPercent } from '../lib/format';
  import { locale, t } from '../lib/i18n';
  import { buildLoanChartSlices, describeDonutSegment, type LoanChartSlice } from '../lib/loan-chart';

  export let loans: LoanSummary[] = [];

  const CX = 100;
  const CY = 100;
  const OUTER_R = 86;
  const INNER_R = 64;

  let hoveredId: string | null = null;

  $: tr = $t;
  $: inputs = loans.map((loan) => ({
    id: loan.id,
    label: loan.label,
    amountMinor: loan.remaining_balance.amount_minor,
  }));
  $: slices = buildLoanChartSlices(inputs);
  $: totalMinor = slices.reduce((sum, s) => sum + s.amountMinor, 0);
  $: currency = loans[0]?.remaining_balance.currency ?? 'EUR';
  $: activeSlice = hoveredId ? slices.find((s) => s.id === hoveredId) ?? null : null;
  $: chartLabel = slices.map((s) => `${s.label} ${formatPercent(s.percent, $locale)}%`).join(', ');

  function setHover(id: string | null) {
    hoveredId = id;
  }

  function sliceAriaLabel(slice: LoanChartSlice) {
    return `${slice.label}: ${formatMoney(slice.amountMinor, currency)} (${formatPercent(slice.percent, $locale)}%)`;
  }
</script>

<section class="card balance-chart" aria-label={tr('loans.chartTitle')}>
  <header class="balance-chart-header">
    <h2 class="balance-chart-title">{tr('loans.chartTitle')}</h2>
    <p class="balance-chart-sub muted">{tr('loans.chartSubtitle')}</p>
  </header>

  {#if slices.length === 0}
    <p class="balance-chart-empty muted">{tr('loans.chartEmpty')}</p>
  {:else}
    <div class="balance-chart-body">
      <div class="balance-chart-visual">
        <svg
          class="balance-chart-svg"
          class:is-hovering={hoveredId != null}
          viewBox="0 0 200 200"
          role="group"
          aria-label={chartLabel}
        >
          {#each slices as slice (slice.id)}
            <path
              class="chart-slice"
              class:is-active={hoveredId === slice.id}
              d={describeDonutSegment(CX, CY, OUTER_R, INNER_R, slice.startAngle, slice.endAngle)}
              fill={slice.color}
              stroke="var(--color-surface)"
              stroke-width="1.5"
              tabindex="0"
              role="button"
              aria-label={sliceAriaLabel(slice)}
              on:mouseenter={() => setHover(slice.id)}
              on:mouseleave={() => setHover(null)}
              on:focus={() => setHover(slice.id)}
              on:blur={() => setHover(null)}
            />
          {/each}
        </svg>
        <div class="balance-chart-center" aria-live="polite">
          {#if activeSlice}
            <span class="balance-chart-center-label is-loan">{activeSlice.label}</span>
            <span class="balance-chart-total tabular">{formatMoney(activeSlice.amountMinor, currency)}</span>
            <span class="balance-chart-center-pct tabular">{formatPercent(activeSlice.percent, $locale)}%</span>
          {:else}
            <span class="balance-chart-center-label">{tr('household.totalRemaining')}</span>
            <span class="balance-chart-total tabular">{formatMoney(totalMinor, currency)}</span>
          {/if}
        </div>
      </div>

      <ul class="balance-chart-legend">
        {#each slices as slice (slice.id)}
          <li
            class="balance-chart-legend-item"
            class:is-active={hoveredId === slice.id}
          >
            <button
              type="button"
              class="balance-chart-legend-btn"
              aria-label={sliceAriaLabel(slice)}
              on:mouseenter={() => setHover(slice.id)}
              on:mouseleave={() => setHover(null)}
              on:focus={() => setHover(slice.id)}
              on:blur={() => setHover(null)}
            >
              <span class="balance-chart-dot" style="background-color: {slice.color}"></span>
              <span class="balance-chart-legend-label">{slice.label}</span>
              <span class="balance-chart-legend-meta tabular">
                {formatMoney(slice.amountMinor, currency)}
                <span class="balance-chart-legend-pct">({formatPercent(slice.percent, $locale)}%)</span>
              </span>
            </button>
          </li>
        {/each}
      </ul>
    </div>
  {/if}
</section>
