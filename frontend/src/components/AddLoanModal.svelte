<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { api } from '../lib/api/client';
  import { todayIso } from '../lib/format';
  import { t } from '../lib/i18n';
  import RecurringSonderzahlungFields from './RecurringSonderzahlungFields.svelte';

  export let open = false;

  const dispatch = createEventDispatcher<{ saved: void; close: void }>();

  let label = '';
  let balance = '';
  let aprPercent = '';
  let paymentType: 'tilgung_percent' | 'tilgung_euro' = 'tilgung_euro';
  let tilgungPercent = '';
  let tilgungEuro = '';
  let frequency: 'monthly' | 'yearly' = 'monthly';
  let startDate = todayIso();
  let firstPaymentDate = todayIso();
  let error = '';
  let saving = false;
  let recurringFields: RecurringSonderzahlungFields;
  let prevOpen = false;

  $: tr = $t;

  $: {
    if (open && !prevOpen) resetForm();
    prevOpen = open;
  }

  /** Prefer live DOM value; fall back to bind state. */
  function fieldValue(id: string, fallback: string) {
    const el = document.getElementById(id) as HTMLInputElement | HTMLSelectElement | null;
    const dom = (el?.value ?? '').trim();
    const bound = fallback.trim();
    return dom || bound;
  }

  function parseEuroAmount(raw: string): number | null {
    const normalized = raw.trim().replace(/\s/g, '').replace(',', '.');
    if (!normalized) return null;
    const value = Number.parseFloat(normalized);
    return Number.isFinite(value) ? value : null;
  }

  function resetForm() {
    label = '';
    balance = '';
    aprPercent = '';
    paymentType = 'tilgung_euro';
    tilgungPercent = '';
    tilgungEuro = '';
    frequency = 'monthly';
    startDate = todayIso();
    firstPaymentDate = todayIso();
    error = '';
    saving = false;
    if (recurringFields) recurringFields.enabled = false;
  }

  async function save() {
    error = '';
    const nameVal = fieldValue('label', label);
    const aprVal = fieldValue('apr', aprPercent);
    const balanceVal = fieldValue('balance', balance);
    const percentVal = fieldValue('tilgung-pct', tilgungPercent);
    const euroVal = fieldValue('tilgung-euro', tilgungEuro);
    if (!nameVal) {
      error = tr('errors.labelRequired');
      document.getElementById('label')?.focus();
      return;
    }
    if (!aprVal) {
      error = tr('errors.aprRequired');
      document.getElementById('apr')?.focus();
      return;
    }
    const balanceAmount = parseEuroAmount(balanceVal);
    if (balanceAmount == null || balanceAmount <= 0) {
      error = tr('errors.balanceRequired');
      document.getElementById('balance')?.focus();
      return;
    }
    if (paymentType === 'tilgung_euro' && !euroVal) {
      error = tr('errors.tilgungEuroRequired');
      document.getElementById('tilgung-euro')?.focus();
      return;
    }
    if (paymentType === 'tilgung_percent' && !percentVal) {
      error = tr('errors.tilgungPercentRequired');
      document.getElementById('tilgung-pct')?.focus();
      return;
    }
    const remaining = Math.round(balanceAmount * 100);
    const aprParsed = parseEuroAmount(aprVal);
    if (aprParsed == null || aprParsed < 0) {
      error = tr('errors.aprRequired');
      document.getElementById('apr')?.focus();
      return;
    }
    const body: Record<string, unknown> = {
      label: nameVal,
      setup_mode: 'advanced',
      remaining_balance_minor: remaining,
      payment_frequency: frequency,
      payment_type: paymentType,
      apr_basis_points: Math.round(aprParsed * 100),
    };
    if (paymentType === 'tilgung_euro') {
      const euroAmount = parseEuroAmount(euroVal);
      if (euroAmount == null || euroAmount <= 0) {
        error = tr('errors.tilgungEuroRequired');
        document.getElementById('tilgung-euro')?.focus();
        return;
      }
      body.tilgung_euro_minor = Math.round(euroAmount * 100);
    } else {
      const pctAmount = parseEuroAmount(percentVal);
      if (pctAmount == null || pctAmount < 0) {
        error = tr('errors.tilgungPercentRequired');
        document.getElementById('tilgung-pct')?.focus();
        return;
      }
      body.tilgung_percent_basis_points = Math.round(pctAmount * 100);
      body.original_principal_minor = remaining;
    }
    body.loan_start_date = fieldValue('start', startDate);
    body.first_payment_date = fieldValue('first-payment', firstPaymentDate);
    if (recurringFields?.enabled) {
      const recurring = recurringFields.toApiPayload();
      if (recurring.length) body.recurring_sonderzahlungen = recurring;
    }
    saving = true;
    try {
      await api.createLoan(body);
      dispatch('saved');
      dispatch('close');
    } catch (e) {
      error = e instanceof Error ? e.message : tr('errors.saveFailed');
    } finally {
      saving = false;
    }
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="modal-backdrop"
    role="presentation"
    tabindex="-1"
    on:click={() => dispatch('close')}
    on:keydown={(e) => e.key === 'Escape' && dispatch('close')}
  >
    <div
      class="modal"
      role="dialog"
      aria-modal="true"
      aria-labelledby="add-loan-title"
      tabindex="0"
      on:click|stopPropagation={() => {}}
      on:keydown|stopPropagation={() => {}}
    >
      <div class="modal-header">
        <h2 id="add-loan-title">{tr('addLoan.title')}</h2>
        {#if error}
          <p class="error modal-error" role="alert">{error}</p>
        {/if}
      </div>
      <div class="modal-body">
      <div class="field">
        <label for="label">{tr('addLoan.name')}</label>
        <input id="label" bind:value={label} required />
      </div>
      <div class="field">
        <label for="balance">{tr('addLoan.balance')}</label>
        <input id="balance" type="number" step="0.01" bind:value={balance} required />
      </div>
      <div class="field">
        <label for="apr">{tr('addLoan.interestRate')}</label>
        <input id="apr" type="number" step="0.01" min="0" bind:value={aprPercent} required />
        <p class="hint muted">{tr('addLoan.interestHint')}</p>
      </div>
      <div class="field">
        <label for="ptype">{tr('addLoan.paymentMethod')}</label>
        <select id="ptype" bind:value={paymentType}>
          <option value="tilgung_percent">{tr('addLoan.tilgungPercentOption')}</option>
          <option value="tilgung_euro">{tr('addLoan.tilgungEuroOption')}</option>
        </select>
      </div>
      {#if paymentType === 'tilgung_percent'}
        <div class="field">
          <label for="tilgung-pct">{tr('addLoan.tilgungPercent')}</label>
          <input id="tilgung-pct" type="number" step="0.01" min="0" bind:value={tilgungPercent} required />
        </div>
      {:else}
        <div class="field">
          <label for="tilgung-euro">{tr('addLoan.tilgungEuro')}</label>
          <input id="tilgung-euro" type="number" step="0.01" min="0" bind:value={tilgungEuro} required />
        </div>
      {/if}
      <div class="field">
        <label for="freq">{tr('addLoan.frequency')}</label>
        <select id="freq" bind:value={frequency}>
          <option value="monthly">{tr('addLoan.monthly')}</option>
          <option value="yearly">{tr('addLoan.yearly')}</option>
        </select>
      </div>
      <div class="field">
        <label for="start">{tr('addLoan.startDate')}</label>
        <input id="start" type="date" bind:value={startDate} />
      </div>
      <div class="field">
        <label for="first-payment">{tr('addLoan.firstPaymentDate')}</label>
        <input id="first-payment" type="date" bind:value={firstPaymentDate} />
      </div>
      <RecurringSonderzahlungFields bind:this={recurringFields} />
      <div class="modal-actions">
        <button type="button" class="secondary" disabled={saving} on:click={() => dispatch('close')}>{tr('common.cancel')}</button>
        <button type="button" class="primary-accent" disabled={saving} on:click={save}>
          {saving ? tr('common.saving') : tr('common.saveLoan')}
        </button>
      </div>
      </div>
    </div>
  </div>
{/if}

