<script lang="ts">
  import { t } from '../lib/i18n';
  import { theme, toggleTheme } from '../lib/theme';

  $: tr = $t;
  $: isDark = $theme === 'dark';
  $: label = isDark ? tr('theme.switchToLight') : tr('theme.switchToDark');
</script>

<button
  type="button"
  class="ghost icon-btn theme-toggle"
  on:click={toggleTheme}
  aria-label={label}
  title={label}
>
  {#if isDark}
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
      <circle cx="12" cy="12" r="4" />
      <path
        d="M12 2v2M12 20v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M2 12h2M20 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"
        stroke-linecap="round"
      />
    </svg>
  {:else}
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
      <path d="M21 14.5A7.5 7.5 0 0112 19a7.5 7.5 0 010-15 6.5 6.5 0 009 9.5z" stroke-linejoin="round" />
    </svg>
  {/if}
</button>
