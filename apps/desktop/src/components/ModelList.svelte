<!--
  Speech model catalogue with install / resume / remove (Inspector section
  "Voice"). `onchange` lets the parent reload the list after a removal.
-->
<script lang="ts">
  import { store } from '../lib/store.svelte';
  import { api, errorText } from '../lib/bridge';
  import type { ModelRow } from '../lib/types';

  let { models, onchange }: { models: ModelRow[]; onchange: () => void } = $props();

  const mb = (n: number) => `${Math.round(n / 1_000_000)} MB`;
  const fail = (e: unknown) => store.toast(errorText(e), 'error');

  async function remove(id: string) {
    await api.modelsRemove(id).catch(fail);
    onchange();
  }

  function install(id: string) {
    api.modelsInstall(id).catch(fail);
  }
</script>

<div class="field">
  <span class="caption">Sprachmodelle</span>
  {#if models.length === 0}
    <span class="footnote">Kein Modellkatalog verfügbar. Ohne Modell läuft Voice im Demo-Modus.</span>
  {/if}
  {#each models as m (m.entry.id)}
    {@const p = store.modelProgress[m.entry.id]}
    <div class="model">
      <div>
        <div>
          {m.entry.name}
          <span class="footnote">· {m.entry.languages.join(', ')} · {mb(m.totalSize)}</span>
        </div>
        <div class="footnote">
          Lizenz {m.entry.license}{m.requiresAttribution && m.entry.attribution
            ? ` · ${m.entry.attribution}`
            : ''}
        </div>
        {#if p && !p.finished}<progress max={p.total} value={p.done}></progress>{/if}
      </div>
      {#if m.status === 'installed'}
        <button class="btn" onclick={() => remove(m.entry.id)}>Entfernen</button>
      {:else}
        <button
          class="btn outline"
          disabled={!!p && !p.finished}
          onclick={() => install(m.entry.id)}
        >
          {m.status === 'incomplete' ? 'Fortsetzen' : 'Laden'}
        </button>
      {/if}
    </div>
  {/each}
</div>

<style>
  .model {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
    padding: 8px 0;
    border-bottom: 1px solid var(--ui-border);
  }
  .model progress {
    width: 100%;
    height: 4px;
    accent-color: var(--ui-accent);
  }
</style>
