<script lang="ts">
  import './app.css';
  import { onMount } from 'svelte';
  import { api, type DashboardResponse, type AmortizationRow } from './lib/api/client';
  import { t } from './lib/i18n';
  import HouseholdSummary from './components/HouseholdSummary.svelte';
  import InsightsBanner from './components/InsightsBanner.svelte';
  import LoanBalanceChart from './components/LoanBalanceChart.svelte';
  import PayoffTimelineChart from './components/PayoffTimelineChart.svelte';
  import LoanCard from './components/LoanCard.svelte';
  import AddLoanModal from './components/AddLoanModal.svelte';
  import SettingsPanel from './components/SettingsPanel.svelte';
  import LanguageSwitcher from './components/LanguageSwitcher.svelte';
  import ThemeToggle from './components/ThemeToggle.svelte';
  import AuthScreen from './components/AuthScreen.svelte';
  import PrivacyBanner from './components/PrivacyBanner.svelte';
  import AmortizationTable from './components/AmortizationTable.svelte';

  let dashboard: DashboardResponse | null = null;
  let showArchived = false;
  let showAdd = false;
  let showSettings = false;
  let loading = true;
  let error = '';
  let authChecked = false;
  let authenticated = false;
  let needsSetup = false;

  let showCombinedAmort = false;
  let combinedAmortRows: AmortizationRow[] | null = null;
  let combinedAmortTotal = 0;
  let combinedAmortLoading = false;

  async function toggleCombinedAmort() {
    showCombinedAmort = !showCombinedAmort;
    if (showCombinedAmort && combinedAmortRows === null) {
      combinedAmortLoading = true;
      try {
        const sched = await api.combinedAmortization();
        combinedAmortRows = sched.rows;
        combinedAmortTotal = sched.total_payments;
      } catch {
        combinedAmortRows = [];
      } finally {
        combinedAmortLoading = false;
      }
    }
  }

  $: tr = $t;

  async function checkAuth() {
    const status = await api.authStatus();
    needsSetup = status.needs_setup;
    authenticated = status.authenticated;
    authChecked = true;
  }

  async function load() {
    loading = true;
    error = '';
    try {
      dashboard = await api.dashboard(showArchived);
    } catch (e) {
      error = e instanceof Error ? e.message : tr('errors.loadDashboard');
    } finally {
      loading = false;
    }
  }

  async function onLoggedIn() {
    authenticated = true;
    await load();
  }

  async function logout() {
    await api.logout();
    authenticated = false;
    dashboard = null;
  }

  onMount(async () => {
    try {
      await checkAuth();
      if (authenticated) await load();
    } catch (e) {
      error = e instanceof Error ? e.message : tr('errors.loadDashboard');
      authChecked = true;
    } finally {
      loading = false;
    }
  });
</script>

{#if !authChecked}
  <div class="page page-loading" role="status">
    <span class="spinner" aria-hidden="true"></span>
    {tr('common.loading')}
  </div>
{:else if !authenticated}
  <AuthScreen {needsSetup} on:loggedIn={onLoggedIn} />
{:else}
  <div class="app-shell">
    <div class="app-top-bar">
      <header class="app-header">
        <div class="app-header-inner">
          <div class="brand">
            <div class="brand-mark" aria-hidden="true">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M4 19V5M8 19v-6m4 6V9m4 10V7m4 12V11" stroke-linecap="round" />
              </svg>
            </div>
            <h1 class="brand-title">{tr('app.title')}</h1>
          </div>

          <div class="header-end">
            <div class="header-utilities">
              <ThemeToggle />
              <LanguageSwitcher />
              <label class="toggle-switch" title={tr('nav.showArchived')}>
                <input type="checkbox" bind:checked={showArchived} on:change={load} />
                <span class="toggle-switch-track" aria-hidden="true"></span>
                <span class="toggle-switch-label">{tr('nav.showArchived')}</span>
              </label>
            </div>

            <nav class="header-nav" aria-label={tr('nav.utilityNav')}>
              <button
                type="button"
                class="ghost icon-btn"
                on:click={() => (showSettings = true)}
                title={tr('nav.settings')}
              >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                  <circle cx="12" cy="12" r="3" />
                  <path
                    d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"
                    stroke-linecap="round"
                  />
                </svg>
                <span class="icon-btn-label">{tr('nav.settings')}</span>
              </button>
              <button type="button" class="ghost icon-btn" on:click={logout} title={tr('auth.signOut')}>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                  <path d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4M16 17l5-5-5-5M21 12H9" stroke-linecap="round" stroke-linejoin="round" />
                </svg>
                <span class="icon-btn-label">{tr('auth.signOut')}</span>
              </button>
            </nav>

            <button type="button" class="primary-accent header-cta" on:click={() => (showAdd = true)}>
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.25" aria-hidden="true">
                <path d="M12 5v14M5 12h14" stroke-linecap="round" />
              </svg>
              {tr('nav.addLoan')}
            </button>
          </div>
        </div>
      </header>
    </div>

    <main class="page" id="main-content">
      {#if loading}
        <div class="page-loading" role="status">
          <span class="spinner" aria-hidden="true"></span>
          {tr('common.loading')}
        </div>
      {:else if error}
        <p class="error card" role="alert">{error}</p>
      {:else if dashboard}
        {#if dashboard.loans.length === 0}
          <div class="card empty-state">
            <div class="empty-icon" aria-hidden="true">
              <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75">
                <rect x="3" y="6" width="18" height="14" rx="2" />
                <path d="M3 10h18M8 14h4" stroke-linecap="round" />
              </svg>
            </div>
            <h2>{tr('empty.title')}</h2>
            <p>{tr('empty.body')}</p>
            <button type="button" class="primary-accent" on:click={() => (showAdd = true)}>{tr('empty.cta')}</button>
          </div>
        {:else}
          <HouseholdSummary data={dashboard.household} />
          <InsightsBanner {dashboard} />
          <LoanBalanceChart loans={dashboard.loans} />
          <PayoffTimelineChart
            timeline={dashboard.payoff_timeline}
            currency={dashboard.household.total_balance.currency}
          />

          <div class="combined-amort-toggle-row">
            <button
              type="button"
              class="link-button combined-amort-toggle-btn"
              on:click={toggleCombinedAmort}
              aria-expanded={showCombinedAmort}
            >
              <span class="combined-amort-chevron" class:open={showCombinedAmort}>▶</span>
              {showCombinedAmort ? tr('amort.hideHousehold') : tr('amort.showHousehold')}
            </button>
          </div>

          {#if showCombinedAmort}
            <div class="card combined-amort-card">
              <h2 class="combined-amort-heading">{tr('amort.householdTitle')}</h2>
              {#if combinedAmortLoading}
                <p class="muted" style="font-size:0.85rem">{tr('common.loading')}</p>
              {:else if combinedAmortRows !== null}
                <AmortizationTable
                  rows={combinedAmortRows}
                  currency={dashboard.household.total_balance.currency}
                  totalPayments={combinedAmortTotal}
                />
              {/if}
            </div>
          {/if}

          <p class="section-title">{tr('loans.sectionTitle')}</p>
          <section class="loans" aria-label="Loans">
            {#each dashboard.loans as loan (loan.id)}
              <LoanCard {loan} on:refresh={load} />
            {/each}
          </section>
        {/if}
      {/if}
    </main>
    <PrivacyBanner />
  </div>

  <AddLoanModal open={showAdd} on:close={() => (showAdd = false)} on:saved={load} />
  <SettingsPanel open={showSettings} on:close={() => (showSettings = false)} on:imported={load} />
{/if}
