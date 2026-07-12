<script lang="ts">
  import { t } from '../lib/i18n';

  $: tr = $t;
</script>

<footer class="privacy-footer" role="note" aria-label={tr('privacy.onDeviceAria')}>
  <p class="privacy-footer-inner">
    <svg class="privacy-footer-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
      <path
        d="M12 3l8 4v5c0 5-3.5 8.5-8 9-4.5-.5-8-4-8-9V7l8-4z"
        stroke-linejoin="round"
      />
      <path d="M9 12l2 2 4-4" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
    <span>{tr('privacy.onDevice')}</span>
  </p>
</footer>
