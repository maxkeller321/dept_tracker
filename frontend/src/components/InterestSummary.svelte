<script lang="ts">
  import { formatMoney } from '../lib/format';
  import { interestMessage, t } from '../lib/i18n';

  export let interestPaidMinor: number | null = null;
  export let interestRemainingMinor: number | null = null;
  export let message: string | null = null;
  export let currency = 'EUR';

  $: tr = $t;
  $: displayMessage = interestMessage(message, tr);
</script>

<section class="panel interest" aria-label={tr('interest.regionLabel')}>
  <h3>{tr('interest.title')}</h3>
  {#if displayMessage}
    <p class="muted">{displayMessage}</p>
  {:else}
    <div class="mini-stat-row">
      <div class="mini-stat">
        <span class="label">{tr('interest.paidToDate')}</span>
        <span class="value tabular">{formatMoney(interestPaidMinor ?? 0, currency)}</span>
      </div>
      <div class="mini-stat">
        <span class="label">{tr('interest.remaining')}</span>
        <span class="value tabular">{formatMoney(interestRemainingMinor ?? 0, currency)}</span>
      </div>
    </div>
  {/if}
</section>
