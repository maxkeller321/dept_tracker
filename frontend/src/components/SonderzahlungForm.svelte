<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { todayIso } from '../lib/format';
  import { t } from '../lib/i18n';

  export let disabled = false;

  const dispatch = createEventDispatcher<{
    immediate: { amount: string };
    schedule: { amount: string; date: string };
  }>();

  let extraAmount = '';
  let scheduleAmount = '';
  let scheduleDate = todayIso();

  $: tr = $t;
  $: today = todayIso();
</script>

<section class="panel sonderzahlung">
  <h3>{tr('sonder.title')} <span class="muted">{tr('sonder.subtitle')}</span></h3>

  <div class="sonder-row">
    <div class="sonder-row-head">
      <span class="sonder-row-label">{tr('sonder.applyNow')}</span>
      <span class="sonder-row-desc muted">{tr('sonder.immediateHint')}</span>
    </div>
    <div class="sonder-grid">
      <div class="field">
        <label for="sonder-extra-amount">{tr('sonder.extraAmount')}</label>
        <input
          id="sonder-extra-amount"
          type="number"
          step="0.01"
          placeholder="0,00"
          aria-label={tr('sonder.extraAmount')}
          bind:value={extraAmount}
          disabled={disabled}
        />
      </div>
      <div class="field field-date">
        <label for="sonder-extra-date">{tr('loan.paymentDate')}</label>
        <input
          id="sonder-extra-date"
          type="date"
          value={today}
          min={today}
          max={today}
          disabled
          aria-label={tr('loan.paymentDate')}
        />
      </div>
      <div class="field field-action">
        <span class="field-action-spacer" aria-hidden="true"></span>
        <button
          type="button"
          class="primary-accent"
          disabled={disabled || !extraAmount}
          on:click={() => dispatch('immediate', { amount: extraAmount })}
        >{tr('sonder.applyNow')}</button>
      </div>
    </div>
  </div>

  <div class="sonder-divider" role="separator"></div>

  <div class="sonder-row">
    <div class="sonder-row-head">
      <span class="sonder-row-label">{tr('sonder.schedule')}</span>
      <span class="sonder-row-desc muted">{tr('sonder.planHint')}</span>
    </div>
    <div class="sonder-grid">
      <div class="field">
        <label for="sonder-schedule-amount">{tr('sonder.scheduleAmount')}</label>
        <input
          id="sonder-schedule-amount"
          type="number"
          step="0.01"
          placeholder="0,00"
          aria-label={tr('sonder.scheduleAmount')}
          bind:value={scheduleAmount}
          disabled={disabled}
        />
      </div>
      <div class="field field-date">
        <label for="sonder-schedule-date">{tr('loan.paymentDate')}</label>
        <input
          id="sonder-schedule-date"
          type="date"
          bind:value={scheduleDate}
          disabled={disabled}
          aria-label={tr('loan.paymentDate')}
        />
      </div>
      <div class="field field-action">
        <span class="field-action-spacer" aria-hidden="true"></span>
        <button
          type="button"
          class="secondary"
          disabled={disabled || !scheduleAmount}
          on:click={() => dispatch('schedule', { amount: scheduleAmount, date: scheduleDate })}
        >{tr('sonder.schedule')}</button>
      </div>
    </div>
  </div>
</section>

<style>
  .sonder-row + .sonder-divider {
    margin: 1.1rem 0;
  }

  .sonder-divider {
    height: 1px;
    background: var(--color-border);
  }

  .sonder-row-head {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    margin-bottom: 0.65rem;
  }

  .sonder-row-label {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text);
  }

  .sonder-row-desc {
    font-size: 0.8125rem;
    line-height: 1.4;
  }

  .sonder-grid {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(9.5rem, 11rem) auto;
    gap: 0.65rem 0.75rem;
    align-items: end;
  }

  .sonder-grid .field {
    margin-bottom: 0;
  }

  .sonder-grid label {
    display: block;
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--color-text-secondary);
    margin-bottom: 0.35rem;
  }

  .field-action {
    display: flex;
    flex-direction: column;
  }

  .field-action-spacer {
    display: block;
    height: 1.25rem;
  }

  .field-action button {
    white-space: nowrap;
  }

  @media (max-width: 640px) {
    .sonder-grid {
      grid-template-columns: 1fr;
    }

    .field-action-spacer {
      display: none;
    }

    .field-action button {
      width: 100%;
    }
  }
</style>
