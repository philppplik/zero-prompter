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
  import { frameMeter, type FrameStats } from '../lib/frameMeter';
  import type { DiagReport } from '../lib/types';

  let report = $state<DiagReport | null>(null);
  let micTest = $state(false);
  let loadError = $state<string | null>(null);
  // Scroll smoothness of the operator preview in this window (rAF, last ~240 frames).
  let frames = $state<FrameStats | null>(null);

  const voiceOn = $derived(store.state?.voice.enabled ?? false);
  const levelPct = $derived(Math.max(0, Math.min(100, ((store.level + 60) / 60) * 100)));
  const headline = $derived(report ? engineHeadline(report) : '');
  // Raw level samples (every 500 ms, last 30 s). Their 90th percentile stands in
  // for "level while speaking"; below −50 dBFS the microphone is too quiet.
  let rawSamples = $state<number[]>([]);
  const rawP90 = $derived.by(() => {
    if (rawSamples.length < 10) return null;
    const s = [...rawSamples].sort((a, b) => a - b);
    return s[Math.round((s.length - 1) * 0.9)];
  });
  const rawQuiet = $derived(!!report?.running && rawP90 != null && rawP90 > -100 && rawP90 < -50);
  const fmtDb = (db: number | undefined) => (db == null || db <= -100 ? '–' : `${db.toFixed(0)} dBFS`);

  async function refresh() {
    frames = frameMeter.stats();
    try {
      const r = await api.diagSnapshot();
      report = r;
      loadError = null;
      if (r.running && r.rawLevelDb != null) rawSamples = [...rawSamples.slice(-59), r.rawLevelDb];
      else if (!r.running) rawSamples = [];
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
    {#if report?.running}
      <dl class="kv">
        <dt>Mikrofon roh</dt><dd class:warn={rawQuiet}>{fmtDb(report.rawLevelDb)} · Gain {report.gainDb != null ? `${report.gainDb >= 0 ? '+' : ''}${report.gainDb.toFixed(0)} dB` : '–'}</dd>
        {#if report.channelDb?.length}
          <dt>Kanäle</dt>
          <dd>
            {#each report.channelDb as db, i (i)}<span class:active={report.activeChannel === i}>K{i + 1} {fmtDb(db)}</span>{i < report.channelDb.length - 1 ? ' · ' : ''}{/each}
            {report.activeChannel == null ? ' (gemischt)' : ''}
          </dd>
        {/if}
      </dl>
      {#if rawQuiet}
        <p class="footnote warn">Mikrofon sehr leise (&lt; −50 dBFS beim Sprechen). Windows: Einstellungen → System → Sound → Eingabe → Eingangslautstärke erhöhen; Mikrofon-Gain unter Voice &amp; Modelle auf Auto lassen.</p>
      {/if}
    {/if}
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
    <h3 class="caption">Scroll-Laufruhe (Vorschau, letzte {frames?.frames ?? 0} Frames)</h3>
    {#if frames}
      <table class="lat">
        <thead><tr><th>Frame-Zeit</th><th>p50</th><th>p95</th><th>max</th></tr></thead>
        <tbody>
          <tr title="requestAnimationFrame-Abstand im Bedienfenster">
            <td>rAF ({frames.hz.toFixed(0)} Hz)</td><td>{fmtMs(frames.p50)}</td><td>{fmtMs(frames.p95)}</td><td>{fmtMs(frames.max)}</td>
          </tr>
        </tbody>
      </table>
      <dl class="kv">
        <dt>Verworfene Frames</dt><dd class:warn={frames.dropped > 0}>{frames.dropped} im Fenster · {frames.droppedTotal} gesamt (Frame &gt; 1,5 × Median)</dd>
        <dt>Positions-Jitter</dt>
        <dd class:warn={(frames.jitterPx ?? 0) > 0.5}>
          {frames.jitterPx == null ? '– (steht)' : `${frames.jitterPx.toFixed(2)} px`}
          <span class="footnote">Standardabweichung der Bewegung pro Frame gegenüber gleichmäßigem Lauf (0 = perfekt)</span>
        </dd>
      </dl>
      <div class="field-row">
        <button class="btn" onclick={() => { frameMeter.reset(); frames = null; }}>Zurücksetzen</button>
        <span class="footnote">Talent-Fenster: Frametime-Anzeige unter Darstellung zeigt dieselben Werte dort.</span>
      </div>
    {:else}
      <p class="footnote">Noch keine Messwerte (Vorschau nicht sichtbar).</p>
    {/if}
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
  .kv .active { font-weight: 600; }
  .meter { height: 6px; border-radius: 3px; background: var(--ui-surface-sunken); overflow: hidden; margin-bottom: 8px; }
  .meter span { display: block; height: 100%; background: var(--state-tracking); transition: width 66ms linear; }
  .lat { width: 100%; border-collapse: collapse; font-size: 12px; font-variant-numeric: tabular-nums; }
  .lat th, .lat td { text-align: right; padding: 3px 6px; border-bottom: 1px solid var(--ui-border); }
  .lat th:first-child, .lat td:first-child { text-align: left; }
  .errors { list-style: none; margin: 0; padding: 0; font-size: 12px; display: grid; gap: 4px; }
  .errors .t { color: var(--ui-text-muted); font-variant-numeric: tabular-nums; }
</style>
