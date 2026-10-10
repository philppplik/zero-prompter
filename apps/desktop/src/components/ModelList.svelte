<!--
  Speech model catalogue with install / resume / remove (Inspector section
  "Voice"), plus the hybrid STT choice (ADR-006): streaming model per primary
  language (automatic = manifest default + fallback chain) and the optional
  offline resync switch (tier L, Parakeet). `onchange` lets the parent reload
  the list after a removal.
-->
<script lang="ts">
  import { store } from '../lib/store.svelte';
  import { api, errorText, isTauri } from '../lib/bridge';
  import type { ModelRow, ModelSelection, VoiceModelPrefs } from '../lib/types';

  let { models, onchange }: { models: ModelRow[]; onchange: () => void } = $props();

  const LANGS = ['de', 'en'] as const;
  const LANG_LABELS: Record<string, string> = { de: 'Deutsch', en: 'Englisch' };

  let prefs = $state<VoiceModelPrefs>({ streaming: {}, resync: false, resyncModel: null });
  let selection = $state<ModelSelection | null>(null);

  const mb = (n: number) => `${Math.round(n / 1_000_000)} MB`;
  const fail = (e: unknown) => store.toast(errorText(e), 'error');
  const nameOf = (id: string) => models.find((m) => m.entry.id === id)?.entry.name ?? id;
  const roleLabel = (m: ModelRow) =>
    m.entry.engine === 'sherpaVad' ? 'Segmentierung (wird automatisch mitgeladen)' : m.roles.includes('resync') ? 'Resync' : 'Streaming';
  const installed = (id: string) => models.find((m) => m.entry.id === id)?.status === 'installed';

  const streamingFor = (lang: string) =>
    models
      .filter((m) => m.roles.includes('streaming') && m.entry.languages.includes(lang))
      .sort((a, b) => (a.entry.defaultRank ?? 99) - (b.entry.defaultRank ?? 99));
  const resyncModels = $derived(models.filter((m) => m.roles.includes('resync')));

  async function reload() {
    if (!isTauri) return;
    try {
      prefs = await api.voiceModelsGet();
      selection = await api.voiceModelsSelection(null);
    } catch (e) {
      fail(e);
    }
  }

  async function save(next: VoiceModelPrefs) {
    prefs = next;
    try {
      await api.voiceModelsSet($state.snapshot(next));
      selection = await api.voiceModelsSelection(null);
    } catch (e) {
      fail(e);
    }
  }

  function setStreaming(lang: string, id: string) {
    const streaming = { ...prefs.streaming };
    if (id) streaming[lang] = id;
    else delete streaming[lang];
    save({ ...prefs, streaming });
  }

  async function remove(id: string) {
    await api.modelsRemove(id).catch(fail);
    onchange();
  }

  function install(id: string) {
    api.modelsInstall(id).catch(fail);
  }

  // Reload choice + effective selection whenever the catalogue changes.
  $effect(() => {
    void models;
    reload();
  });
</script>

<div class="field">
  <span class="caption">Spracherkennung</span>
  {#each LANGS as lang (lang)}
    <label class="field-row">
      <span class="lang">{LANG_LABELS[lang]}</span>
      <select
        value={prefs.streaming[lang] ?? ''}
        onchange={(e) => setStreaming(lang, (e.currentTarget as HTMLSelectElement).value)}
      >
        <option value="">Automatisch</option>
        {#each streamingFor(lang) as m (m.entry.id)}
          <option value={m.entry.id}>{m.entry.name}{installed(m.entry.id) ? '' : ' (nicht geladen)'}</option>
        {/each}
      </select>
    </label>
  {/each}
  {#if selection}
    <span class="footnote">
      Aktiv ({selection.language}): {selection.streamingChain.length
        ? selection.streamingChain.map(nameOf).join(' → ')
        : 'kein Modell installiert, Demo-Modus'}
    </span>
  {/if}
  <label class="field-row">
    <input
      type="checkbox"
      checked={prefs.resync}
      onchange={(e) => save({ ...prefs, resync: (e.currentTarget as HTMLInputElement).checked })}
    />
    <span>Resync im Hintergrund (Parakeet, Stufe L)</span>
  </label>
  {#if prefs.resync}
    {#if resyncModels.length > 1}
      <select
        value={prefs.resyncModel ?? ''}
        onchange={(e) => save({ ...prefs, resyncModel: (e.currentTarget as HTMLSelectElement).value || null })}
      >
        <option value="">Automatisch</option>
        {#each resyncModels as m (m.entry.id)}<option value={m.entry.id}>{m.entry.name}</option>{/each}
      </select>
    {/if}
    <span class="footnote">
      {selection?.resync
        ? `Resync: ${nameOf(selection.resync)}`
        : 'Resync-Modell nicht installiert, nur Streaming aktiv.'}
    </span>
  {/if}
  {#if selection}
    <span class="footnote">
      Segmentierung: {selection.sileroVad ? 'Silero VAD aktiv' : 'Pegel (RMS), Silero VAD nicht installiert'}
    </span>
  {/if}
  {#if store.engine?.notes?.length}
    {#each store.engine.notes as n}<span class="footnote">{n}</span>{/each}
  {/if}
  <span class="footnote">Änderungen gelten beim nächsten Start von Voice.</span>
</div>

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
          <span class="footnote">
            · {m.entry.languages.join(', ')} · {mb(m.totalSize)} · {roleLabel(m)}
          </span>
        </div>
        <div class="footnote">
          Lizenz {m.entry.license}{m.requiresAttribution && m.entry.attribution
            ? ` · ${m.entry.attribution}`
            : ''}
        </div>
        {#if m.unverified.length}
          <div class="footnote" title={m.unverified.map(([f, h]) => `${f}: ${h ?? 'noch nicht berechnet'}`).join('\n')}>
            {m.unverified.length} Datei(en) ohne Manifest-Hash (SHA-256 nach Download berechnet)
          </div>
        {/if}
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
  .lang {
    min-width: 72px;
  }
</style>
