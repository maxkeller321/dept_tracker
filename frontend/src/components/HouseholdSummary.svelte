<script lang="ts">
  import type { DashboardResponse } from '../lib/api/client';
  import { formatMoney } from '../lib/format';
  import { t } from '../lib/i18n';

  export let data: DashboardResponse['household'];

  $: tr = $t;
</script>

<div class="stat-grid" role="region" aria-label={tr('household.title')}>
  <div class="stat-card">
    <span class="stat-label">{tr('household.totalRemaining')}</span>
    <span class="stat-value tabular">{formatMoney(data.total_balance.amount_minor, data.total_balance.currency)}</span>
  </div>
  <div class="stat-card">
    <span class="stat-label">{tr('household.monthlyObligation')}</span>
    <span class="stat-value tabular">{formatMoney(data.total_monthly_obligation.amount_minor, data.total_monthly_obligation.currency)}</span>
  </div>
</div>
