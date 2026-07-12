<script lang="ts">
  import type { PayoffTimelineData } from '../lib/payoff-timeline-chart';
  import {
    buildChartLayout,
    buildExtendedLayers,
    buildStackedLayers,
    closestDataIndex,
    downsampleTimeline,
    formatAxisMoney,
    formatMonthLabel,
    splitAreaPaths,
    splitExtendedPaths,
    yScale,
  } from '../lib/payoff-timeline-chart';
  import { formatMoney } from '../lib/format';
  import { locale, t } from '../lib/i18n';

  export let timeline: PayoffTimelineData | null = null;
  export let currency = 'EUR';

  $: tr = $t;
  $: sampled = timeline ? downsampleTimeline(timeline) : null;

  let showInterest = false;

  $: layers = sampled ? buildStackedLayers(sampled) : [];
  $: extLayers = sampled ? buildExtendedLayers(sampled) : [];
  $: layout = sampled ? buildChartLayout(sampled, showInterest) : null;

  let hoveredIdx: number | null = null;

  $: todayIdx = sampled ? Math.min(sampled.as_of_index, (sampled.dates.length || 1) - 1) : null;
  $: todayX = layout && todayIdx != null ? layout.xs[todayIdx] : null;
  $: hoverX = layout && hoveredIdx != null ? layout.xs[hoveredIdx] : null;

  $: hoveredDate = hoveredIdx != null && sampled ? sampled.dates[hoveredIdx] : null;
  $: hoveredRows =
    hoveredIdx != null && sampled
      ? sampled.series.map((s, i) => ({
          id: s.id,
          label: s.label,
          color: layers[i]?.color ?? '#888',
          principal: s.balances_minor[hoveredIdx!] ?? 0,
          interest: showInterest ? (s.interest_remaining_minor?.[hoveredIdx!] ?? 0) : 0,
        })).filter((r) => r.principal > 0)
      : null;
  $: hoveredTotalPrincipal = hoveredRows
    ? hoveredRows.reduce((s, r) => s + r.principal, 0)
    : null;
  $: hoveredTotalInterest = hoveredRows
    ? hoveredRows.reduce((s, r) => s + r.interest, 0)
    : null;

  function onMouseMove(e: MouseEvent) {
    if (!layout) return;
    const svg = e.currentTarget as SVGElement;
    const rect = svg.getBoundingClientRect();
    const scaleX = layout.width / rect.width;
    const svgX = (e.clientX - rect.left) * scaleX;
    if (svgX < layout.padLeft - 4 || svgX > layout.width - layout.padRight + 4) {
      hoveredIdx = null;
      return;
    }
    hoveredIdx = closestDataIndex(svgX, layout);
  }

  function onMouseLeave() {
    hoveredIdx = null;
  }

  function tooltipX(hx: number): number {
    if (!layout) return hx + 12;
    return hx + 12 + 140 > layout.width - layout.padRight ? hx - 152 : hx + 12;
  }

  function tooltipRows(rows: typeof hoveredRows): { label: string; color: string; val: number; isInterest?: boolean }[] {
    if (!rows) return [];
    const out: { label: string; color: string; val: number; isInterest?: boolean }[] = [];
    for (const r of rows) {
      out.push({ label: r.label, color: r.color, val: r.principal });
      if (showInterest && r.interest > 0) {
        out.push({ label: tr('loans.timelineInterestOf') + ' ' + r.label, color: r.color, val: r.interest, isInterest: true });
      }
    }
    return out;
  }

  // Tooltip height depends on number of rows
  function tooltipHeight(rows: typeof hoveredRows): number {
    if (!rows) return 40;
    const rowCount = rows.length + (showInterest ? rows.filter((r) => r.interest > 0).length : 0);
    const hasTotal = rows.length > 1;
    return 22 + rowCount * 14 + (hasTotal ? 18 : 0);
  }
</script>

<section class="card payoff-timeline" aria-label={tr('loans.timelineTitle')}>
  <header class="balance-chart-header">
    <div class="ptl-header-row">
      <div>
        <h2 class="balance-chart-title">{tr('loans.timelineTitle')}</h2>
        <p class="balance-chart-sub muted">{tr('loans.timelineSubtitle')}</p>
      </div>
      <button
        type="button"
        class="ptl-interest-toggle"
        class:is-active={showInterest}
        on:click={() => { showInterest = !showInterest; hoveredIdx = null; }}
        title={showInterest ? tr('loans.timelineHideInterest') : tr('loans.timelineShowInterest')}
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2z"/>
          <path d="M12 6v6l4 2" stroke-linecap="round"/>
        </svg>
        {showInterest ? tr('loans.timelineHideInterest') : tr('loans.timelineShowInterest')}
      </button>
    </div>
  </header>

  {#if !sampled || (showInterest ? extLayers : layers).length === 0 || !layout}
    <p class="balance-chart-empty muted">{tr('loans.timelineEmpty')}</p>
  {:else}
    <div class="payoff-timeline-body">
      <div class="payoff-timeline-chart-wrap">
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <svg
          class="payoff-timeline-svg"
          viewBox="0 0 {layout.width} {layout.height}"
          role="img"
          aria-label={layers.map((l) => l.label).join(', ')}
          on:mousemove={onMouseMove}
          on:mouseleave={onMouseLeave}
        >
          <!-- Stripe patterns for each loan's interest cap -->
          <defs>
            {#each layers as layer (layer.id)}
              <pattern
                id="stripe-{layer.id}"
                patternUnits="userSpaceOnUse"
                width="7"
                height="7"
                patternTransform="rotate(45 0 0)"
              >
                <rect width="7" height="7" fill={layer.color} fill-opacity="0.12" />
                <line x1="0" y1="0" x2="0" y2="7" stroke={layer.color} stroke-width="2.5" stroke-opacity="0.50" />
              </pattern>
            {/each}
          </defs>

          <!-- Y grid lines + labels -->
          {#each layout.yTicks as tick}
            {@const y = yScale(tick, layout)}
            <line
              x1={layout.padLeft} x2={layout.width - layout.padRight}
              y1={y} y2={y}
              class="payoff-grid-line"
            />
            <text
              x={layout.padLeft - 6} y={y + 4}
              class="payoff-axis-label payoff-y-label"
              text-anchor="end"
            >{formatAxisMoney(tick, currency, $locale)}</text>
          {/each}

          {#if showInterest}
            <!-- Extended mode: principal solid + interest striped cap -->
            {#each extLayers as layer (layer.id)}
              {@const { principalPast, principalFuture, interestPast, interestFuture } = splitExtendedPaths(layer, layout, sampled.as_of_index)}
              <!-- Interest caps (striped) -->
              {#if interestFuture}
                <path d={interestFuture} fill="url(#stripe-{layer.id})" fill-opacity="0.45" stroke="none" />
              {/if}
              {#if interestPast}
                <path d={interestPast} fill="url(#stripe-{layer.id})" fill-opacity="0.85" stroke="none" />
              {/if}
              <!-- Principal fills -->
              {#if principalFuture}
                <path d={principalFuture} fill={layer.color} fill-opacity="0.18" stroke="none" />
                <path d={principalFuture} fill="none" stroke={layer.color} stroke-width="1" stroke-opacity="0.28" stroke-dasharray="4 3" />
              {/if}
              {#if principalPast}
                <path d={principalPast} fill={layer.color} fill-opacity="0.80" stroke="none" />
                <path d={principalPast} fill="none" stroke={layer.color} stroke-width="1.5" stroke-opacity="0.60" />
              {/if}
            {/each}
          {:else}
            <!-- Principal-only mode -->
            {#each layers as layer (layer.id)}
              {@const { past, future } = splitAreaPaths(layer, layout, sampled.as_of_index)}
              {#if future}
                <path d={future} fill={layer.color} fill-opacity="0.18" stroke="none" />
                <path d={future} fill="none" stroke={layer.color} stroke-width="1" stroke-opacity="0.28" stroke-dasharray="4 3" />
              {/if}
              {#if past}
                <path d={past} fill={layer.color} fill-opacity="0.80" stroke="none" />
                <path d={past} fill="none" stroke={layer.color} stroke-width="1.5" stroke-opacity="0.60" />
              {/if}
            {/each}
          {/if}

          <!-- Today marker -->
          {#if todayX != null}
            <line
              x1={todayX} x2={todayX}
              y1={layout.padTop} y2={layout.height - layout.padBottom}
              class="payoff-today-line"
            />
            <text x={todayX + 4} y={layout.padTop + 9} class="payoff-axis-label" fill="var(--color-accent)" font-size="9">
              {tr('loans.timelineToday')}
            </text>
          {/if}

          <!-- Hover crosshair -->
          {#if hoverX != null}
            <line
              x1={hoverX} x2={hoverX}
              y1={layout.padTop} y2={layout.height - layout.padBottom}
              class="payoff-hover-line"
            />
          {/if}

          <!-- Hover dots on each layer top -->
          {#if hoveredIdx != null && hoverX != null}
            {#if showInterest}
              {#each extLayers as layer (layer.id)}
                {@const topY = yScale(layer.interestUpper[hoveredIdx], layout)}
                <circle cx={hoverX} cy={topY} r="3.5" fill="url(#stripe-{layer.id})" stroke={layer.color} stroke-width="1.5" />
              {/each}
            {:else}
              {#each layers as layer (layer.id)}
                {@const topY = yScale(layer.upper[hoveredIdx], layout)}
                <circle cx={hoverX} cy={topY} r="3.5" fill={layer.color} stroke="var(--color-surface)" stroke-width="1.5" />
              {/each}
            {/if}
          {/if}

          <!-- Hover tooltip -->
          {#if hoveredIdx != null && hoverX != null && hoveredRows && hoveredDate}
            {@const tx = tooltipX(hoverX)}
            {@const trows = tooltipRows(hoveredRows)}
            {@const hasTotal = (hoveredRows?.length ?? 0) > 1}
            {@const boxH = tooltipHeight(hoveredRows)}
            <g class="payoff-tooltip" transform="translate({tx}, {layout.padTop + 4})">
              <rect x="0" y="0" width="140" height={boxH} class="payoff-tooltip-bg" rx="6" />
              <text x="8" y="14" class="payoff-tooltip-date">{formatMonthLabel(hoveredDate, $locale)}</text>
              {#each trows as row, i}
                {#if row.isInterest}
                  <!-- Interest row: hatched small rect instead of dot -->
                  <rect x="7" y={22 + i * 14 - 5} width="8" height="8" fill="url(#stripe-{hoveredRows?.find((r) => r.color === row.color)?.id ?? ''})" stroke={row.color} stroke-width="0.8" rx="1" />
                {:else}
                  <circle cx="11" cy={22 + i * 14 - 1} r="4" fill={row.color} />
                {/if}
                <text x="22" y={22 + i * 14 + 3} class="payoff-tooltip-row {row.isInterest ? 'is-interest' : ''}">
                  {row.label.length > 11 ? row.label.slice(0, 11) + '…' : row.label}
                </text>
                <text x="132" y={22 + i * 14 + 3} class="payoff-tooltip-val" text-anchor="end">
                  {formatAxisMoney(row.val, currency, $locale)}
                </text>
              {/each}
              {#if hasTotal}
                {@const totalY = 22 + trows.length * 14}
                <line x1="8" x2="132" y1={totalY - 2} y2={totalY - 2} class="payoff-tooltip-divider" />
                <text x="8" y={totalY + 9} class="payoff-tooltip-total">{tr('household.totalRemaining')}</text>
                <text x="132" y={totalY + 9} class="payoff-tooltip-val" text-anchor="end">
                  {formatAxisMoney((hoveredTotalPrincipal ?? 0) + (showInterest ? (hoveredTotalInterest ?? 0) : 0), currency, $locale)}
                </text>
              {/if}
            </g>
          {/if}

          <!-- X axis labels -->
          {#each layout.xLabelIndices as idx}
            {@const x = layout.xs[idx]}
            <text
              x={x} y={layout.height - 8}
              class="payoff-axis-label payoff-x-label"
              text-anchor={idx === 0 ? 'start' : idx === layout.xs.length - 1 ? 'end' : 'middle'}
            >{formatMonthLabel(sampled.dates[idx], $locale)}</text>
          {/each}
        </svg>
      </div>

      <!-- Legend -->
      <ul class="balance-chart-legend payoff-timeline-legend">
        {#each layers as layer (layer.id)}
          <li class="balance-chart-legend-item ptl-legend-item">
            <span class="balance-chart-dot" style="background-color: {layer.color}"></span>
            <span class="balance-chart-legend-label">{layer.label}</span>
          </li>
        {/each}
        {#if showInterest}
          <li class="ptl-legend-interest-row">
            <span class="ptl-stripe-swatch" style="--swatch-color: {layers[0]?.color ?? '#888'}"></span>
            <span class="ptl-hint-label">{tr('loans.timelineInterestLabel')}</span>
          </li>
        {/if}
        <li class="ptl-legend-hint">
          <span class="ptl-hint-past"></span>
          <span class="ptl-hint-label">{tr('loans.timelinePast')}</span>
        </li>
        <li class="ptl-legend-hint">
          <span class="ptl-hint-future"></span>
          <span class="ptl-hint-label">{tr('loans.timelineFuture')}</span>
        </li>
      </ul>
    </div>
  {/if}
</section>
