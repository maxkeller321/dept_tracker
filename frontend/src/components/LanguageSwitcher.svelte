<script lang="ts">
  import { locale, locales, t } from '../lib/i18n';

  $: translate = $t;
</script>

<div class="lang-switcher" role="group" aria-label="Language">
  {#each locales as loc (loc.code)}
    <button
      type="button"
      class="lang-btn"
      class:active={$locale === loc.code}
      title={translate(loc.labelKey)}
      aria-label={translate(loc.labelKey)}
      aria-pressed={$locale === loc.code}
      on:click={() => locale.set(loc.code)}
    >
      <span class="flag" aria-hidden="true">{loc.flag}</span>
    </button>
  {/each}
</div>
