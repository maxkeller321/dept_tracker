<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { api } from '../lib/api/client';
  import { t } from '../lib/i18n';
  import PrivacyBanner from './PrivacyBanner.svelte';
  import ThemeToggle from './ThemeToggle.svelte';
  export let needsSetup = false;

  const dispatch = createEventDispatcher<{ loggedIn: void }>();

  type Mode = 'login' | 'register';

  let mode: Mode = needsSetup ? 'register' : 'login';

  $: if (needsSetup) {
    mode = 'register';
  }
  let username = '';
  let password = '';
  let confirmPassword = '';
  let error = '';
  let loading = false;

  $: tr = $t;
  $: accountExists = !needsSetup && mode === 'register';

  function showLogin() {
    error = '';
    mode = 'login';
  }

  function showRegister() {
    error = '';
    mode = 'register';
  }

  function mapRegisterError(message: string): string {
    if (message.toLowerCase().includes('account already exists')) {
      return tr('auth.accountExists');
    }
    return message;
  }

  async function submitLogin() {
    error = '';
    loading = true;
    try {
      await api.login(username.trim(), password);
      dispatch('loggedIn');
    } catch (e) {
      error = e instanceof Error ? e.message : tr('auth.loginFailed');
    } finally {
      loading = false;
    }
  }

  async function submitRegister() {
    error = '';
    if (accountExists) {
      error = tr('auth.accountExists');
      return;
    }
    if (password !== confirmPassword) {
      error = tr('auth.passwordMismatch');
      return;
    }
    if (password.length < 6) {
      error = tr('auth.passwordTooShort');
      return;
    }
    loading = true;
    try {
      await api.register(username.trim(), password);
      dispatch('loggedIn');
    } catch (e) {
      const raw = e instanceof Error ? e.message : tr('auth.registerFailed');
      error = mapRegisterError(raw);
      if (error === tr('auth.accountExists')) {
        mode = 'login';
      }
    } finally {
      loading = false;
    }
  }

  function submit() {
    if (mode === 'login') {
      void submitLogin();
    } else {
      void submitRegister();
    }
  }
</script>

<div class="auth-page">
  <div class="auth-toolbar">
    <ThemeToggle />
  </div>
  <div class="login-shell">
    <div class="card login-card">
      <div class="login-brand">
        <div class="brand-mark" aria-hidden="true">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M4 19V5M8 19v-6m4 6V9m4 10V7m4 12V11" stroke-linecap="round" />
          </svg>
        </div>
        {#if mode === 'register'}
          <h1>{tr('auth.setupTitle')}</h1>
          <p class="muted">{tr('auth.setupSubtitle')}</p>
        {:else}
          <h1>{tr('auth.title')}</h1>
          <p class="muted">{tr('auth.subtitle')}</p>
        {/if}
      </div>

      <div class="auth-mode-tabs" role="tablist" aria-label={tr('auth.modeLabel')}>
        <button
          type="button"
          role="tab"
          aria-selected={mode === 'login'}
          class:active={mode === 'login'}
          on:click={showLogin}
        >
          {tr('auth.signIn')}
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={mode === 'register'}
          class:active={mode === 'register'}
          on:click={showRegister}
        >
          {tr('auth.createAccount')}
        </button>
      </div>

      {#if accountExists}
        <p class="auth-info" role="status">{tr('auth.accountExistsHint')}</p>
      {/if}

      <form on:submit|preventDefault={submit}>
        <div class="field">
          <label for="auth-username">{tr('auth.username')}</label>
          <input
            id="auth-username"
            name="username"
            autocomplete="username"
            bind:value={username}
            required
            minlength={mode === 'register' ? 2 : undefined}
          />
        </div>
        <div class="field">
          <label for="auth-password">{tr('auth.password')}</label>
          <input
            id="auth-password"
            name="password"
            type="password"
            autocomplete={mode === 'register' ? 'new-password' : 'current-password'}
            bind:value={password}
            required
            minlength={mode === 'register' ? 6 : undefined}
          />
        </div>
        {#if mode === 'register'}
          <div class="field">
            <label for="auth-confirm">{tr('auth.confirmPassword')}</label>
            <input
              id="auth-confirm"
              name="confirm"
              type="password"
              autocomplete="new-password"
              bind:value={confirmPassword}
              required
              minlength="6"
            />
          </div>
        {/if}
        {#if error}<p class="error" role="alert">{error}</p>{/if}
        <button type="submit" class="primary-accent" disabled={loading || (accountExists && mode === 'register')}>
          {#if mode === 'register'}
            {loading ? tr('auth.creatingAccount') : tr('auth.createAccount')}
          {:else}
            {loading ? tr('auth.signingIn') : tr('auth.signIn')}
          {/if}
        </button>
      </form>

      <p class="auth-switch">
        {#if mode === 'login'}
          {tr('auth.noAccount')}
          <button type="button" class="link-button" on:click={showRegister}>
            {tr('auth.createAccountLink')}
          </button>
        {:else}
          {tr('auth.alreadyHaveAccount')}
          <button type="button" class="link-button" on:click={showLogin}>
            {tr('auth.signInLink')}
          </button>
        {/if}
      </p>
    </div>
  </div>
  <PrivacyBanner />
</div>
