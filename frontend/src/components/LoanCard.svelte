<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { LoanSummary } from '../lib/api/client';
  import { api } from '../lib/api/client';
  import type { AmortizationRow } from '../lib/api/client';
  import { formatDate, formatLastPayment, formatMoney, formatPercent, todayIso } from '../lib/format';
  import { locale, t } from '../lib/i18n';
  import { resolvePlanAction } from '../lib/sonderzahlung';
  import SonderzahlungForm from './SonderzahlungForm.svelte';
  import UpcomingPayments from './UpcomingPayments.svelte';
  import InterestSummary from './InterestSummary.svelte';
  import PaymentHistory from './PaymentHistory.svelte';
  import EditLoanModal from './EditLoanModal.svelte';
  import AmortizationTable from './AmortizationTable.svelte';

  export let loan: LoanSummary;
  export let expanded = false;

  const dispatch = createEventDispatcher<{ refresh: void }>();

  let detail: Record<string, unknown> | null = null;
  let error = '';
  let showEdit = false;
  let showAmortization = false;
  let amortRows: AmortizationRow[] | null = null;
  let amortTotal = 0;
  let amortLoading = false;

  $: tr = $t;
  $: freqLabel =
    loan.payment_frequency === 'yearly'
      ? tr('loan.frequency.yearly')
      : tr('loan.frequency.monthly');
  $: progress = Math.min(100, Math.max(0, loan.progress_percent));
  $: progressLabel = formatPercent(progress, $locale);

  async function loadDetail() {
    detail = await api.loanDetail(loan.id);
  }

  async function toggle() {
    expanded = !expanded;
    if (expanded) await loadDetail();
    if (!expanded) showAmortization = false;
  }

  async function toggleAmortization() {
    showAmortization = !showAmortization;
    if (showAmortization && amortRows === null) {
      amortLoading = true;
      try {
        const sched = await api.loanAmortization(loan.id);
        amortRows = sched.rows;
        amortTotal = sched.total_payments;
      } catch {
        amortRows = [];
      } finally {
        amortLoading = false;
      }
    }
  }

  async function refresh() {
    await loadDetail();
    // Amortization data depends on the current balance; invalidate so it reloads.
    amortRows = null;
    if (showAmortization) {
      await toggleAmortization();
    }
    dispatch('refresh');
  }

  async function onImmediate(e: CustomEvent<{ amount: string }>) {
    error = '';
    try {
      await api.immediateSonderzahlung(loan.id, {
        amount_minor: Math.round(parseFloat(e.detail.amount) * 100),
        paid_at: todayIso(),
        confirm_overpayment: true,
        recalculate_from_past: false,
      });
      await refresh();
    } catch (err) {
      error = err instanceof Error ? err.message : tr('errors.generic');
    }
  }

  async function onSchedule(e: CustomEvent<{ amount: string; date: string }>) {
    error = '';
    const amountMinor = Math.round(parseFloat(e.detail.amount) * 100);
    const planDate = e.detail.date;
    try {
      if (resolvePlanAction(planDate, todayIso()) === 'backdated') {
        await api.immediateSonderzahlung(loan.id, {
          amount_minor: amountMinor,
          paid_at: planDate,
          confirm_overpayment: true,
          recalculate_from_past: true,
        });
      } else {
        await api.scheduleSonderzahlung(loan.id, {
          amount_minor: amountMinor,
          due_date: planDate,
        });
      }
      await refresh();
    } catch (err) {
      error = err instanceof Error ? err.message : tr('errors.generic');
    }
  }

  async function cancelScheduled(id: string) {
    await api.cancelScheduled(loan.id, id);
    await refresh();
  }

  async function archive() {
    if (!confirm(tr('loan.confirmArchive'))) return;
    await api.archiveLoan(loan.id);
    dispatch('refresh');
  }

  async function remove() {
    if (!confirm(tr('loan.confirmDelete'))) return;
    await api.deleteLoan(loan.id);
    dispatch('refresh');
  }

  $: upcoming = (detail?.upcoming_scheduled as Array<{
    id: string;
    amount_minor: number;
    due_date: string;
    status: string;
  }>) ?? [];
  $: interestPaid = detail?.interest_paid_to_date as { amount_minor: number } | undefined;
  $: interestRemaining = detail?.interest_remaining_estimate as { amount_minor: number } | undefined;
  $: editApr = (detail?.apr_percent as number | null | undefined) ?? null;
  $: editTilgungEuro =
    detail?.payment_type === 'tilgung_euro'
      ? ((detail.tilgung_euro_minor as number | null) ?? null)
      : null;
  $: editTilgungPercent =
    detail?.payment_type === 'tilgung_percent'
      ? ((detail.tilgung_percent_basis_points as number | null) ?? null)
      : null;
  $: editPaymentType =
    (detail?.payment_type as 'tilgung_percent' | 'tilgung_euro') ?? 'tilgung_euro';
</script>

<article class="card loan-card" class:expanded={expanded}>
  <button
    type="button"
    class="loan-header"
    data-testid="loan-expand"
    on:click={toggle}
    aria-expanded={expanded}
  >
    <div>
      <span class="loan-title">{loan.label}</span>
      <span class="loan-sub tabular"
        >{formatMoney(loan.remaining_balance.amount_minor)} · {freqLabel}</span
      >
      <div class="progress-track" aria-hidden="true">
        <div class="progress-fill" style="width: {progress}%"></div>
      </div>
      <div class="progress-label">
        <span>{tr('loan.paidPercent', { percent: progressLabel })}</span>
        <span>{tr('loan.payoff')}: {formatDate(loan.projected_payoff_date)}</span>
      </div>
    </div>
    <div class="loan-meta">
      <strong class="tabular">{formatMoney(loan.periodic_payment.amount_minor)}</strong>
      <span>{freqLabel}</span>
      <span class="chevron" aria-hidden="true">▼</span>
    </div>
  </button>

  {#if expanded}
    <div class="loan-details">
      <p class="muted">
        {tr('loan.paymentLine', {
          payment: formatMoney(loan.periodic_payment.amount_minor),
          last: formatLastPayment(loan.last_payment_date, tr('loan.noPaymentsYet')),
        })}
      </p>
      {#if detail?.first_payment_date}
        <p class="muted">
          {tr('loan.firstPaymentDate', {
            date: formatDate(detail.first_payment_date as string),
          })}
        </p>
      {/if}

      {#if detail}
        <InterestSummary
          interestPaidMinor={interestPaid?.amount_minor ?? null}
          interestRemainingMinor={interestRemaining?.amount_minor ?? null}
          message={(detail.interest_message as string) ?? null}
          currency={loan.remaining_balance.currency}
        />
      {/if}

      <p class="muted auto-hint">{tr('loan.autoPaymentsHint')}</p>

      <SonderzahlungForm on:immediate={onImmediate} on:schedule={onSchedule} />
      <UpcomingPayments items={upcoming} onCancel={cancelScheduled} />
      <PaymentHistory loanId={loan.id} />

      <div class="amort-toggle-row">
        <button
          type="button"
          class="link-button amort-toggle-btn"
          on:click={toggleAmortization}
          aria-expanded={showAmortization}
        >
          <span class="amort-chevron" class:open={showAmortization}>▶</span>
          {showAmortization ? tr('amort.hidePlan') : tr('amort.showPlan')}
        </button>
      </div>

      {#if showAmortization}
        <div class="amort-section">
          {#if amortLoading}
            <p class="muted" style="font-size:0.8rem">{tr('common.loading')}</p>
          {:else if amortRows !== null}
            <AmortizationTable
              rows={amortRows}
              currency={loan.remaining_balance.currency}
              totalPayments={amortTotal}
            />
          {/if}
        </div>
      {/if}

      <div class="loan-actions row-flex">
        <button type="button" class="secondary" on:click={() => (showEdit = true)}>{tr('loan.edit')}</button>
        <button type="button" class="secondary" on:click={archive}>{tr('loan.markPaidOff')}</button>
        <button type="button" class="danger" on:click={remove}>{tr('loan.delete')}</button>
      </div>
      {#if error}<p class="error">{error}</p>{/if}
    </div>
  {/if}
</article>

<EditLoanModal
  open={showEdit}
  loanId={loan.id}
  initialLabel={loan.label}
  initialAprPercent={editApr}
  initialTilgungEuro={editTilgungEuro}
  initialTilgungPercent={editTilgungPercent}
  initialPaymentType={editPaymentType}
  on:close={() => (showEdit = false)}
  on:saved={() => {
    showEdit = false;
    dispatch('refresh');
  }}
/>

<style>
  .auto-hint {
    margin: 0.75rem 0;
    font-size: 0.8125rem;
    padding: 0.65rem 0.75rem;
    background: var(--color-accent-soft);
    border-radius: var(--radius-sm);
    border: 1px solid color-mix(in srgb, var(--color-accent) 20%, var(--color-border));
    color: var(--color-primary);
  }

  .loan-actions {
    margin-top: 1rem;
  }

  .amort-toggle-row {
    margin-top: 1rem;
    margin-bottom: 0;
  }

  .amort-toggle-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.8125rem;
    color: var(--color-text-muted);
    padding: 0;
  }
  .amort-toggle-btn:hover {
    color: var(--color-primary);
  }

  .amort-chevron {
    font-size: 0.6rem;
    display: inline-block;
    transition: transform 0.2s;
  }
  .amort-chevron.open {
    transform: rotate(90deg);
  }

  .amort-section {
    margin-top: 0.5rem;
    padding: 0.75rem;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
  }
</style>
