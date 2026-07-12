<script lang="ts">
  import type { DashboardResponse } from '../lib/api/client';
  import { formatMoney, parseLocalDate } from '../lib/format';
  import { locale, t } from '../lib/i18n';

  export let dashboard: DashboardResponse;

  $: tr = $t;

  // ── Debt-free countdown ───────────────────────────────────────
  $: debtFreeDate = (() => {
    const dates = dashboard.loans
      .map((l) => l.projected_payoff_date)
      .filter((d): d is string => !!d)
      .sort();
    return dates.length ? dates[dates.length - 1] : null;
  })();

  $: countdown = (() => {
    if (!debtFreeDate) return null;
    const today = new Date();
    const target = parseLocalDate(debtFreeDate);
    if (target <= today) return { years: 0, months: 0, alreadyFree: true };
    let years = target.getFullYear() - today.getFullYear();
    let months = target.getMonth() - today.getMonth();
    if (months < 0) { years--; months += 12; }
    return { years, months, alreadyFree: false };
  })();

  $: debtFreeDateFormatted = debtFreeDate
    ? new Intl.DateTimeFormat($locale === 'de' ? 'de-DE' : $locale, {
        month: 'long',
        year: 'numeric',
      }).format(parseLocalDate(debtFreeDate))
    : null;

  // ── Total cost of credit ──────────────────────────────────────
  $: currency = dashboard.household.total_balance.currency;

  /** Remaining principal across all active loans. */
  $: totalRemainingPrincipal = dashboard.loans.reduce(
    (s, l) => s + l.remaining_balance.amount_minor,
    0,
  );

  /** Future interest still to be paid, from the timeline at as_of_index. */
  $: totalFutureInterest = (() => {
    const ai = dashboard.payoff_timeline.as_of_index;
    return dashboard.payoff_timeline.series.reduce(
      (s, ser) => s + (ser.interest_remaining_minor?.[ai] ?? 0),
      0,
    );
  })();

  $: totalStillToPay = totalRemainingPrincipal + totalFutureInterest;

  /** Sum of original_principal across loans that have it recorded. */
  $: totalOriginal = dashboard.loans.reduce(
    (s, l) => s + (l.original_principal?.amount_minor ?? l.remaining_balance.amount_minor),
    0,
  );

  /** Total already paid down in principal. */
  $: totalPaidDown = dashboard.loans.reduce((s, l) => {
    const orig = l.original_principal?.amount_minor ?? l.remaining_balance.amount_minor;
    return s + Math.max(0, orig - l.remaining_balance.amount_minor);
  }, 0);

  $: overallProgress = totalOriginal > 0
    ? Math.min(100, (totalPaidDown / totalOriginal) * 100)
    : 0;

  $: interestPremiumPct = totalRemainingPrincipal > 0
    ? Math.round((totalFutureInterest / totalRemainingPrincipal) * 100)
    : 0;

  function fmt(minor: number) {
    return formatMoney(minor, currency);
  }
</script>

<div class="insights-banner card">
  <!-- ── Debt-free countdown ───────────────────────── -->
  <div class="insights-countdown">
    {#if countdown?.alreadyFree}
      <p class="insights-countdown-label">{tr('insights.allPaidOff')}</p>
    {:else if countdown}
      <p class="insights-countdown-label">{tr('insights.debtFreeIn')}</p>
      <div class="insights-countdown-value">
        {#if countdown.years > 0}
          <span class="insights-num">{countdown.years}</span>
          <span class="insights-unit">{tr(countdown.years === 1 ? 'insights.year' : 'insights.years')}</span>
        {/if}
        {#if countdown.months > 0 || countdown.years === 0}
          <span class="insights-num">{countdown.months}</span>
          <span class="insights-unit">{tr(countdown.months === 1 ? 'insights.month' : 'insights.months')}</span>
        {/if}
      </div>
      {#if debtFreeDateFormatted}
        <p class="insights-countdown-date">{tr('insights.expectedOn')} {debtFreeDateFormatted}</p>
      {/if}
    {:else}
      <p class="insights-countdown-label muted">{tr('insights.noProjection')}</p>
    {/if}

    <!-- Overall progress bar -->
    {#if overallProgress > 0}
      <div class="insights-progress-wrap" title="{overallProgress.toFixed(1)}% {tr('insights.paidOff')}">
        <div class="insights-progress-bar">
          <div class="insights-progress-fill" style="width: {overallProgress.toFixed(1)}%"></div>
        </div>
        <span class="insights-progress-label">{overallProgress.toFixed(1)}% {tr('insights.paidOff')}</span>
      </div>
    {/if}
  </div>

  <div class="insights-divider" aria-hidden="true"></div>

  <!-- ── Total cost of credit ──────────────────────── -->
  <div class="insights-cost">
    <p class="insights-cost-title">{tr('insights.totalCostTitle')}</p>

    <dl class="insights-cost-rows">
      <div class="insights-cost-row">
        <dt class="muted">{tr('insights.remainingPrincipal')}</dt>
        <dd class="tabular">{fmt(totalRemainingPrincipal)}</dd>
      </div>
      <div class="insights-cost-row insights-cost-row--interest">
        <dt class="muted">{tr('insights.futureInterest')}</dt>
        <dd class="tabular">
          + {fmt(totalFutureInterest)}
          {#if interestPremiumPct > 0}
            <span class="insights-premium">+{interestPremiumPct}%</span>
          {/if}
        </dd>
      </div>
      <div class="insights-cost-divider" aria-hidden="true"></div>
      <div class="insights-cost-row insights-cost-row--total">
        <dt>{tr('insights.totalStillToPay')}</dt>
        <dd class="tabular">{fmt(totalStillToPay)}</dd>
      </div>
    </dl>

    <p class="insights-cost-note muted">
      {tr('insights.costNote')}
    </p>
  </div>
</div>
