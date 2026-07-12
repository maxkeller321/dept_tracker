<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { api } from '../lib/api/client';
  import { t } from '../lib/i18n';

  export let open = false;

  const dispatch = createEventDispatcher<{ close: void; imported: void }>();

  let error = '';
  let success = '';
  let fileInput: HTMLInputElement;

  $: tr = $t;

  async function exportJson() {
    error = '';
    try {
      const data = await api.exportData();
      const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
      const a = document.createElement('a');
      a.href = URL.createObjectURL(blob);
      a.download = `dept-tracker-backup-${new Date().toISOString().slice(0, 10)}.json`;
      a.click();
      success = tr('settings.backupDownloaded');
    } catch (e) {
      error = e instanceof Error ? e.message : tr('settings.exportFailed');
    }
  }

  async function onFileSelected() {
    const file = fileInput?.files?.[0];
    if (!file) return;
    error = '';
    success = '';
    if (!confirm(tr('settings.confirmImport'))) return;
    try {
      const text = await file.text();
      const bundle = JSON.parse(text);
      await api.importData(bundle);
      success = tr('settings.importComplete');
      dispatch('imported');
    } catch (e) {
      error = e instanceof Error ? e.message : tr('settings.importFailed');
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
    <div class="modal" role="dialog" aria-modal="true" aria-labelledby="settings-title" tabindex="0">
      <div class="modal-header">
        <h2 id="settings-title">{tr('settings.title')}</h2>
        <p class="muted">{tr('settings.body')}</p>
      </div>
      <div class="modal-body">
        <div class="actions-col">
          <button type="button" class="secondary" on:click={exportJson}>{tr('settings.export')}</button>
          <label class="import-label">
            {tr('settings.import')}
            <input type="file" accept="application/json,.json" bind:this={fileInput} on:change={onFileSelected} />
          </label>
        </div>
        {#if success}<p class="success">{success}</p>{/if}
        {#if error}<p class="error">{error}</p>{/if}
        <div class="modal-actions">
          <button type="button" class="secondary" on:click={() => dispatch('close')}>{tr('common.close')}</button>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .actions-col {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin: 1rem 0;
  }
  .import-label {
    font-size: 0.875rem;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }
  .import-label input[type='file'] {
    font-size: 0.8125rem;
    margin-top: 0.25rem;
  }
</style>
