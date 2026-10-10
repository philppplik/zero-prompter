<!--
  Diagnose tab of the settings dialog (ADR-007): live level, active engine /
  model, tracker state, latency p50/p95 (rolling, measured in the Rust voice
  thread), recent errors, log export and microphone test. Polls
  `diag_snapshot` every 500 ms while visible.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { save } from '@tauri-apps/plugin-dialog';
  import Icon from './Icon.svelte';
  import { store } from '../lib/store.svelte';
  import { api, errorText } from '../lib/bridge';
  import { engineHeadline, fmtMs, LATENCY_ROWS } from '../lib/diagnostics';
  import type { DiagReport } from '../lib/types';

  let report = $state<DiagReport | null>(null);
  let micTest = $state(false);
  let loadError = $state<string | null>(null);

  const voiceOn = $derived(store.state?.voice.enabled ?? false);
  const levelPct = $derived(Math.max(0, Math.min(100, ((store.level + 60) / 60) * 100)));
  const headline = $derived(report ? engineHeadline(report) : '');

  async function refresh() {
    try {
      report = await api.diagSnapshot();
      loadError = null;
    } catch (e) {
      loadError = errorText(e);
    }
  }

  async function toggleMic() {
    micTest = !micTest;
    try {
      await api.micTest(micTest);
    } catch (e) {
      micTest = false;
      store.toast(`Mikrofontest: ${errorText(e)}`, 'error');
    }
  }

  async function exportLog() {
    try {
      const day = new Date().toISOString().slice(0, 10);
      const dest = await save({
        defaultPath: `zero-teleprompter-log-${day}.txt`,
        filters: [{ name: 'Log', extensions: ['txt', 'log'] }],
      });
      if (!dest) return;
      const bytes = await api.diagExportLog(dest);
      store.toast(`Log exportiert (${Math.round(bytes / 1024)} KB)`, 'info');
    } catch (e) {
      store.toast(`Log-Export: ${errorText(e)}`, 'error');
    }
  }

  async function copyLogDir() {
    if (!report?.logDir) return;
    try {
      await navigator.clipboard.writeText(report.logDir);
      store.toast('Log-Ordner kopiert', 'info');
    } catch (e) {
      store.toast(errorText(e), 'error');
    }
  }

  onMount(() => {
    refresh();
    const id = setInterval(refresh, 500);
    return () => {
      clearInterval(id);
      if (micTest) api.micTest(false).catch(() => {});
    };
  });
</script>

<div class="diag">
  {#if loadError}<p class="footnote warn">Diagnose nicht verfügbar: {loadError}</p>{/if}

  <section>
    <h3 class="caption">Engine</h3>
    <p class="headline" class:warn={report?.engine?.demo || report?.engine?.reason === 'engineError'}>{headline}</p>
    {#if report?.engine?.demo}
      <button class="btn primary" onclick={() => (store.settingsTab = 'voice')}><Icon name="file" />Modell laden</button>
    {/if}
    <dl class="kv">
      <dt>Build</dt><dd>v{report?.version ?? '–'} · {report?.os}/{report?.arch} · STT {report?.sherpa ? 'sherpa-onnx' : 'fehlt'} · Mikrofon {report?.cpal ? 'cpal' : 'fehlt'}</dd>
      <dt>Modelle</dt><dd>{report?.selection?.streamingChain.length ? report.selection.streamingChain.join(' → ') : 'kein Streaming-Modell installiert'}{report?.selection ? ` · Sprache ${report.selection.language}` : ''}</dd>
      <dt>Audio</dt><dd>{report?.audio ?? (report?.running ? 'kein Mikrofon' : 'nicht aktiv')}</dd>
      <dt>Frames/s</dt><dd>{report?.running ? report.framesPerS.toFixed(0) : '–'} · Tick {report?.tickHz ? report.tickHz.toFixed(0) + ' Hz' : '–'}</dd>
      <dt>Zustand</dt><dd>{report?.running ? (report.tracking ? `Tracking: ${report.voiceState || 'listening'} · Konfidenz ${Math.round((report.confidence ?? 0) * 100)} %` : 'Mikrofontest') : 'Voice aus'}</dd>
      {#if report?.lastPartial}<dt>Erkannt</dt><dd class="partial">„{report.lastPartial}“</dd>{/if}
    </dl>
  </section>

  <section>
    <h3 class="caption">Pegel</h3>
    <div class="meter" role="meter" aria-label="Eingangspegel" aria-valuemin={-60} aria-valuemax={0} aria-valuenow={Math.round(store.level)}>
      <span style:width="{levelPct}%"></span>
    </div>
    <div class="field-row">
      <button class="btn outline" disabled={voiceOn} onclick={toggleMic}><Icon name="mic" />{micTest ? 'Test beenden' : 'Mikrofontest'}</button>
      <span class="footnote">{Math.round(store.level)} dBFS{voiceOn ? ' · Test nicht während Voice-Tracking' : ''}</span>
    </div>
  </section>

  <section>
    <h3 class="caption">Latenz (rollierend, letzte 200)</h3>
    <table class="lat">
      <thead><tr><th>Strecke</th><th>p50</th><th>p95</th><th>n</th></tr></thead>
      <tbody>
        {#each LATENCY_ROWS as row (row.key)}
          {@const s = report?.latency[row.key]}
          <tr title={row.hint}>
            <td>{row.label}</td><td>{fmtMs(s?.p50Ms)}</td><td>{fmtMs(s?.p95Ms)}</td><td>{s?.count ?? 0}</td>
          </tr>
        {/each}
      </tbody>
    </table>
    <p class="footnote">Gemessen im Rust-Voice-Thread. Scroll-Animation im Fenster (~1–2 Frames) kommt hinzu.</p>
  </section>

  <section>
    <h3 class="caption">Letzte Fehler</h3>
    {#if report?.errors.length}
      <ul class="errors">
        {#each report.errors.slice(-8).reverse() as e (e.atMs + e.message)}
          <li><span class="t">{new Date(e.atMs).toLocaleTimeString()}</span> <b>{e.source}</b> {e.message}</li>
        {/each}
      </ul>
    {:else}
      <p class="footnote">Keine Fehler seit dem Start.</p>
    {/if}
  </section>

  <section>
    <h3 class="caption">Log</h3>
    <div class="field-row">
      <button class="btn outline" onclick={exportLog}><Icon name="file" />Log exportieren</button>
      {#if report?.logDir}<button class="btn" onclick={copyLogDir} title="Pfad kopieren">Ordner: {report.logDir}</button>{/if}
    </div>
  </section>
</div>

<style>
  .diag section { margin-bottom: 18px; }
  .headline { font: 600 13px/18px var(--font-ui); margin: 4px 0 8px; }
  .warn { color: var(--state-lost, #d97706); }
  .kv { display: grid; grid-template-columns: max-content 1fr; gap: 4px 12px; margin: 8px 0 0; font-size: 12px; }
  .kv dt { color: var(--ui-text-muted); }
  .kv dd { margin: 0; overflow-wrap: anywhere; }
  .partial { font-style: italic; }
  .meter { height: 6px; border-radius: 3px; background: var(--ui-surface-sunken); overflow: hidden; margin-bottom: 8px; }
  .meter span { display: block; height: 100%; background: var(--state-tracking); transition: width 66ms linear; }
  .lat { width: 100%; border-collapse: collapse; font-size: 12px; font-variant-numeric: tabular-nums; }
  .lat th, .lat td { text-align: right; padding: 3px 6px; border-bottom: 1px solid var(--ui-border); }
  .lat th:first-child, .lat td:first-child { text-align: left; }
  .errors { list-style: none; margin: 0; padding: 0; font-size: 12px; display: grid; gap: 4px; }
  .errors .t { color: var(--ui-text-muted); font-variant-numeric: tabular-nums; }
</style>
