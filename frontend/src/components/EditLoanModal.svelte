<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { api } from '../lib/api/client';
  import { t } from '../lib/i18n';

  export let open = false;
  export let loanId = '';
  export let initialLabel = '';
  export let initialAprPercent: number | null = null;
  export let initialTilgungEuro: number | null = null;
  export let initialTilgungPercent: number | null = null;
  export let initialPaymentType: 'tilgung_percent' | 'tilgung_euro' = 'tilgung_euro';

  const dispatch = createEventDispatcher<{ saved: void; close: void }>();

  let label = '';
  let tilgungEuro = '';
  let tilgungPercent = '';
  let aprPercent = '';
  let paymentType: 'tilgung_percent' | 'tilgung_euro' = 'tilgung_euro';
  let error = '';

  $: tr = $t;

  $: if (open) {
    label = initialLabel;
    paymentType = initialPaymentType;
    aprPercent = initialAprPercent != null ? String(initialAprPercent) : '';
    tilgungEuro =
      initialTilgungEuro != null ? String(initialTilgungEuro / 100) : '';
    tilgungPercent =
      initialTilgungPercent != null ? String(initialTilgungPercent / 100) : '';
  }

  function fieldValue(id: string, fallback: string) {
    const bound = fallback.trim();
    const el = document.getElementById(id) as HTMLInputElement | null;
    const dom = (el?.value ?? '').trim();
    return bound || dom;
  }

  async function save() {
    error = '';
    const aprVal = fieldValue('edit-apr', aprPercent);
    const euroVal = fieldValue('edit-tilgung-euro', tilgungEuro);
    const pctVal = fieldValue('edit-tilgung-pct', tilgungPercent);
    if (!aprVal) {
      error = tr('errors.aprRequired');
      return;
    }
    const body: Record<string, unknown> = {
      label,
      payment_type: paymentType,
      apr_basis_points: Math.round(parseFloat(aprVal) * 100),
    };
    if (paymentType === 'tilgung_euro') {
      if (!euroVal) {
        error = tr('errors.tilgungEuroRequired');
        return;
      }
      body.tilgung_euro_minor = Math.round(parseFloat(euroVal) * 100);
    } else {
      if (!pctVal) {
        error = tr('errors.tilgungPercentRequired');
        return;
      }
      body.tilgung_percent_basis_points = Math.round(parseFloat(pctVal) * 100);
    }
    try {
      await api.updateLoan(loanId, body);
      dispatch('saved');
      dispatch('close');
    } catch (e) {
      error = e instanceof Error ? e.message : tr('errors.updateFailed');
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
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="modal"
      role="dialog"
      aria-modal="true"
      aria-labelledby="edit-loan-title"
      tabindex="0"
      on:click|stopPropagation={() => {}}
      on:keydown|stopPropagation={() => {}}
    >
      <div class="modal-header">
        <h2 id="edit-loan-title">{tr('editLoan.title')}</h2>
      </div>
      <div class="modal-body">
      <div class="field">
        <label for="edit-label">{tr('editLoan.name')}</label>
        <input id="edit-label" bind:value={label} />
      </div>
      <div class="field">
        <label for="edit-apr">{tr('editLoan.interestRate')}</label>
        <input id="edit-apr" type="number" step="0.01" min="0" bind:value={aprPercent} required />
      </div>
      <div class="field">
        <label for="edit-ptype">{tr('editLoan.paymentMethod')}</label>
        <select id="edit-ptype" bind:value={paymentType}>
          <option value="tilgung_percent">{tr('addLoan.tilgungPercentOption')}</option>
          <option value="tilgung_euro">{tr('addLoan.tilgungEuroOption')}</option>
        </select>
      </div>
      {#if paymentType === 'tilgung_percent'}
        <div class="field">
          <label for="edit-tilgung-pct">{tr('addLoan.tilgungPercent')}</label>
          <input id="edit-tilgung-pct" type="number" step="0.01" min="0" bind:value={tilgungPercent} required />
        </div>
      {:else}
        <div class="field">
          <label for="edit-tilgung-euro">{tr('addLoan.tilgungEuro')}</label>
          <input id="edit-tilgung-euro" type="number" step="0.01" min="0" bind:value={tilgungEuro} required />
        </div>
      {/if}
      {#if error}<p class="error">{error}</p>{/if}
      <div class="modal-actions">
        <button type="button" class="secondary" on:click={() => dispatch('close')}>{tr('common.cancel')}</button>
        <button type="button" class="primary-accent" on:click={save}>{tr('common.saveChanges')}</button>
      </div>
      </div>
    </div>
  </div>
{/if}

