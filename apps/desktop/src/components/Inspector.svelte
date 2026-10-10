<!--
  Inspector / settings (DESIGN §3.5): Darstellung · Scroll · Lesemarker · Voice ·
  Ausgabe · Tastenkürzel · Oberfläche. Core settings go through ReplaceSettings,
  UI-only prefs through store.setPrefs (broadcast to the talent window).
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import Icon from './Icon.svelte';
  import HotkeyEditor from './HotkeyEditor.svelte';
  import ModelList from './ModelList.svelte';
  import { FOCUS_LINE_MAX, FOCUS_LINE_MIN, store } from '../lib/store.svelte';
  import { api, errorText } from '../lib/bridge';
  import { setMode } from '../lib/actions';
  import { displayChord } from '../lib/hotkeys';
  import { contrastRatio, FONT_LABELS, THEME_IDS, THEME_LABELS, THEMES } from '../lib/tokens';
  import { tokens } from '../lib/tokens';
  import type { AudioDevice, FontChoice, ModelRow, MonitorInfo, ScrollMode, Settings } from '../lib/types';

  const st = $derived(store.state);
  const prefs = $derived(store.prefs);
  const pt = tokens.type.prompter;

  let monitors = $state<MonitorInfo[]>([]);
  let devices = $state<AudioDevice[]>([]);
  let models = $state<ModelRow[]>([]);
  let selectedMonitor = $state<number | null>(null);
  let micTest = $state(false);

  function update(patch: Partial<Settings>) {
    if (!st) return;
    store.send({ type: 'replaceSettings', value: { ...$state.snapshot(st.settings), ...patch } });
  }

  function toggle(section: string) {
    store.setPrefs({ inspectorOpen: { ...prefs.inspectorOpen, [section]: !prefs.inspectorOpen[section] } });
  }

  async function refreshHardware() {
    try {
      [monitors, devices] = await Promise.all([api.listMonitors(), api.audioDevices()]);
    } catch (e) {
      store.toast(errorText(e), 'error');
    }
    try {
      models = await api.modelsList();
    } catch (e) {
      models = [];
      store.toast(`Modelle: ${errorText(e)}`, 'error');
    }
  }

  async function setMicTest(on: boolean) {
    micTest = on;
    await api.micTest(on).catch(() => {});
    if (!on) store.transcript = '';
  }

  const theme = $derived(THEMES[st?.settings.theme ?? 'studioDark'] ?? THEMES.studioDark);
  const effText = $derived(prefs.textColor ?? theme.text);
  const effBg = $derived(prefs.bgColor ?? theme.bg);
  const ratio = $derived(contrastRatio(effText, effBg));
  const fmtRatio = (r: number) => r.toFixed(1).replace('.', ',') + ' : 1';
  const levelPct = $derived(Math.max(0, Math.min(100, ((store.level + 60) / 60) * 100)));

  async function chooseMonitor(i: number | null) {
    selectedMonitor = i;
    try {
      await api.openTalent(i, st?.output.fullscreen ?? false);
    } catch (e) {
      store.toast(errorText(e), 'error');
    }
  }

  onMount(() => {
    refreshHardware();
    return () => {
      if (micTest) api.micTest(false).catch(() => {});
    };
  });

  const scrollModes: { id: ScrollMode; label: string }[] = [
    { id: 'manual', label: 'Manuell' },
    { id: 'constant', label: 'Konstant' },
    { id: 'voiceActivated', label: 'Sprachaktiviert' },
    { id: 'voiceTracking', label: 'Voice-Tracking' },
  ];
</script>

{#snippet section(id: string, title: string)}
  <button class="sec-head" aria-expanded={!!prefs.inspectorOpen[id]} onclick={() => toggle(id)}>
    <span class="caption">{title}</span>
    <span class="chev" class:open={prefs.inspectorOpen[id]}><Icon name="chevron" size={14} /></span>
  </button>
{/snippet}

{#if st}
  <aside class="inspector" aria-label="Einstellungen">
    <!-- DARSTELLUNG -->
    {@render section('appearance', 'Darstellung')}
    {#if prefs.inspectorOpen.appearance}
      <div class="sec">
        <div class="themes" role="radiogroup" aria-label="Theme">
          {#each THEME_IDS as id, i (id)}
            {@const t = THEMES[id]}
            <button
              class="tile"
              role="radio"
              aria-checked={st.settings.theme === id}
              title="{THEME_LABELS[id]} · {fmtRatio(t.contrast)} {t.wcag} · {displayChord(`Ctrl+${i + 1}`)}"
              onclick={() => store.send({ type: 'setTheme', value: id })}
              style:background={t.bg}
              style:color={t.text}
            >
              <span class="aa">Aa</span>
              <span class="mk" style:background={t.marker}></span>
              <span class="badge">{t.contrast.toFixed(1).replace('.', ',')}</span>
            </button>
          {/each}
        </div>
        <p class="footnote">{THEME_LABELS[st.settings.theme] ?? st.settings.theme}</p>

        <label class="field">
          <span class="caption">Schrift</span>
          <select value={prefs.fontFamily} onchange={(e) => store.setPrefs({ fontFamily: (e.currentTarget as HTMLSelectElement).value as FontChoice })}>
            {#each Object.entries(FONT_LABELS) as [k, v] (k)}<option value={k}>{v}</option>{/each}
          </select>
        </label>

        <div class="field">
          <span class="caption" id="lbl-size">Größe</span>
          <div class="field-row">
            <input type="range" min={pt.sizeMin} max={pt.sizeMax} step="2" aria-labelledby="lbl-size"
              value={st.settings.fontSizePx} aria-valuetext="{st.settings.fontSizePx} Pixel"
              oninput={(e) => store.send({ type: 'setFontSize', value: Number((e.currentTarget as HTMLInputElement).value) })} />
            <input type="number" min={pt.sizeMin} max={pt.sizeMax} value={st.settings.fontSizePx} aria-label="Schriftgröße in Pixel"
              onchange={(e) => store.send({ type: 'setFontSize', value: Number((e.currentTarget as HTMLInputElement).value) })} />
          </div>
        </div>

        <div class="field">
          <span class="caption" id="lbl-measure">Spaltenbreite</span>
          <div class="field-row">
            <input type="range" min="16" max="60" aria-labelledby="lbl-measure" value={prefs.measureCh} aria-valuetext="{prefs.measureCh} Zeichen"
              oninput={(e) => store.setPrefs({ measureCh: Number((e.currentTarget as HTMLInputElement).value) })} />
            <span class="num val">{prefs.measureCh} ch</span>
          </div>
          <span class="footnote">Empfohlen 25–40 Zeichen pro Zeile</span>
        </div>

        <div class="field">
          <span class="caption" id="lbl-lh">Zeilenabstand</span>
          <div class="field-row">
            <input type="range" min="1" max="2.4" step="0.05" aria-labelledby="lbl-lh" value={st.settings.lineHeight}
              onchange={(e) => update({ lineHeight: Number((e.currentTarget as HTMLInputElement).value) })} />
            <span class="num val">{st.settings.lineHeight.toFixed(2)}</span>
          </div>
        </div>

        <label class="check"><input type="checkbox" checked={prefs.dimRead} onchange={() => store.setPrefs({ dimRead: !prefs.dimRead })} /> Gelesenen Text dimmen</label>
        {#if prefs.dimRead}
          <div class="field indent">
            <div class="field-row">
              <input type="range" min="0.3" max="1" step="0.05" aria-label="Gelesen-Opazität" value={prefs.readOpacity ?? theme.readOpacity}
                oninput={(e) => store.setPrefs({ readOpacity: Number((e.currentTarget as HTMLInputElement).value) })} />
              <span class="num val">{Math.round((prefs.readOpacity ?? theme.readOpacity) * 100)} %</span>
            </div>
          </div>
        {/if}

        <div class="field">
          <span class="caption">Farben</span>
          <div class="colors">
            <label title="Text"><input type="color" value={effText} oninput={(e) => store.setPrefs({ textColor: (e.currentTarget as HTMLInputElement).value })} /> Text</label>
            <label title="Hintergrund"><input type="color" value={effBg} oninput={(e) => store.setPrefs({ bgColor: (e.currentTarget as HTMLInputElement).value })} /> Grund</label>
            <label title="Marker"><input type="color" value={prefs.markerColor ?? theme.marker} oninput={(e) => store.setPrefs({ markerColor: (e.currentTarget as HTMLInputElement).value })} /> Marker</label>
          </div>
          <div class="field-row">
            <span class="footnote num" class:bad={ratio < 4.5} class:warn={ratio >= 4.5 && ratio < 7}>
              Kontrast {fmtRatio(ratio)} {ratio >= 7 ? '(AAA)' : ratio >= 4.5 ? '(AA – unter 7:1)' : '– zu gering'}
            </span>
            {#if prefs.textColor || prefs.bgColor || prefs.markerColor}
              <button class="btn" onclick={() => store.setPrefs({ textColor: null, bgColor: null, markerColor: null })}>Zurücksetzen</button>
            {/if}
          </div>
        </div>
      </div>
    {/if}

    <!-- SCROLL -->
    {@render section('scroll', 'Scroll')}
    {#if prefs.inspectorOpen.scroll}
      <div class="sec">
        <label class="field">
          <span class="caption">Modus</span>
          <select value={st.scroll.mode} onchange={(e) => setMode((e.currentTarget as HTMLSelectElement).value as ScrollMode)}>
            {#each scrollModes as m (m.id)}<option value={m.id}>{m.label}</option>{/each}
          </select>
        </label>
        <div class="field">
          <span class="caption" id="lbl-wpm">Tempo</span>
          <div class="field-row">
            <input type="range" min="40" max="400" step="5" aria-labelledby="lbl-wpm" value={st.scroll.wpm} aria-valuetext="{st.scroll.wpm} Wörter pro Minute"
              oninput={(e) => store.send({ type: 'setWpm', value: Number((e.currentTarget as HTMLInputElement).value) })} />
            <span class="num val">{st.scroll.wpm} WPM</span>
          </div>
          <span class="footnote">Gilt für „Konstant“ und als Rückfall bei „Verloren“.</span>
        </div>
        <label class="field">
          <span class="caption">Countdown</span>
          <select value={String(st.settings.countdownSecs)} onchange={(e) => update({ countdownSecs: Number((e.currentTarget as HTMLSelectElement).value) })}>
            <option value="0">Aus</option><option value="3">3 s</option><option value="5">5 s</option>
          </select>
        </label>
        <label class="check"><input type="checkbox" checked={st.settings.reducedMotion} onchange={() => update({ reducedMotion: !st.settings.reducedMotion })} /> Bewegung reduzieren</label>
      </div>
    {/if}

    <!-- LESEMARKER -->
    {@render section('marker', 'Lesemarker')}
    {#if prefs.inspectorOpen.marker}
      <div class="sec">
        <div class="seg" role="group" aria-label="Marker-Stil">
          <button aria-pressed={prefs.markerStyle === 'line'} onclick={() => store.setPrefs({ markerStyle: 'line' })}>Linie</button>
          <button aria-pressed={prefs.markerStyle === 'band'} onclick={() => store.setPrefs({ markerStyle: 'band' })}>Band</button>
        </div>
        <div class="field" style="margin-top:12px">
          <span class="caption" id="lbl-focus">Position</span>
          <div class="field-row">
            <input type="range" min={FOCUS_LINE_MIN} max={FOCUS_LINE_MAX} step="0.01" aria-labelledby="lbl-focus" value={st.settings.focusLine}
              aria-valuetext="{Math.round(st.settings.focusLine * 100)} Prozent"
              onchange={(e) => update({ focusLine: Number((e.currentTarget as HTMLInputElement).value) })} />
            <span class="num val">{Math.round(st.settings.focusLine * 100)} %</span>
          </div>
          <span class="footnote">Auch per Drag auf dem Marker in der Vorschau.</span>
        </div>
      </div>
    {/if}

    <!-- VOICE -->
    {@render section('voice', 'Voice')}
    {#if prefs.inspectorOpen.voice}
      <div class="sec">
        <label class="field">
          <span class="caption">Sprache</span>
          <select value={st.settings.language} onchange={(e) => update({ language: (e.currentTarget as HTMLSelectElement).value })}>
            <option value="de">Deutsch</option><option value="en">English</option>
          </select>
        </label>
        <label class="field">
          <span class="caption">Mikrofon</span>
          <select onchange={(e) => api.setAudioDevice((e.currentTarget as HTMLSelectElement).value || null)}>
            <option value="">Systemstandard</option>
            {#each devices as d (d.id)}<option value={d.id}>{d.name}{d.isDefault ? ' (Standard)' : ''}</option>{/each}
          </select>
        </label>
        <div class="field">
          <div class="meter" role="meter" aria-label="Eingangspegel" aria-valuemin={-60} aria-valuemax={0} aria-valuenow={Math.round(store.level)}>
            <span style:width="{levelPct}%"></span>
          </div>
          <div class="field-row">
            <button class="btn outline" disabled={st.voice.enabled} onclick={() => setMicTest(!micTest)}>
              <Icon name="mic" />{micTest ? 'Test beenden' : 'Mikro testen'}
            </button>
            {#if store.engine}<span class="footnote">{store.engine.demo ? 'Demo-Engine' : store.engine.name}</span>{/if}
          </div>
          {#if micTest && store.transcript}<p class="transcript">„{store.transcript}“</p>{/if}
        </div>
        <label class="field">
          <span class="caption">Wenn verloren</span>
          <select value={st.settings.lostPolicy} onchange={(e) => update({ lostPolicy: (e.currentTarget as HTMLSelectElement).value as Settings['lostPolicy'] })}>
            <option value="stop">Anhalten</option><option value="constantSpeed">Konstant weiter</option>
          </select>
        </label>
        <ModelList {models} onchange={refreshHardware} />
      </div>
    {/if}

    <!-- AUSGABE -->
    {@render section('output', 'Ausgabe')}
    {#if prefs.inspectorOpen.output}
      <div class="sec">
        <div class="monitors" role="radiogroup" aria-label="Ausgabe-Monitor">
          {#each monitors as m (m.index)}
            <button class="mon" role="radio" aria-checked={selectedMonitor === m.index} onclick={() => chooseMonitor(m.index)} title="{m.name} · {m.width}×{m.height}">
              <span class="num big">{m.index + 1}</span>
              <span class="footnote">{m.width}×{m.height}{m.primary ? ' · Haupt' : ''}</span>
            </button>
          {/each}
        </div>
        <div class="field-row wrap">
          <button class="btn outline" onclick={() => (store.talentVisible ? api.closeTalent() : api.openTalent(selectedMonitor, false))}>
            {store.talentVisible ? 'Talent-Fenster schließen' : 'Talent-Fenster öffnen'}
          </button>
          <label class="check"><input type="checkbox" checked={st.output.fullscreen} onchange={() => api.setTalentFullscreen(!st.output.fullscreen).catch(() => {})} /> Vollbild</label>
        </div>
        <label class="check"><input type="checkbox" checked={st.output.mirrorH} onchange={() => store.send({ type: 'setMirror', value: { h: !st.output.mirrorH, v: st.output.mirrorV } })} /> Horizontal spiegeln</label>
        <label class="check"><input type="checkbox" checked={st.output.mirrorV} onchange={() => store.send({ type: 'setMirror', value: { h: st.output.mirrorH, v: !st.output.mirrorV } })} /> Vertikal spiegeln</label>
        <button class="btn" onclick={refreshHardware}>Monitore & Geräte neu laden</button>
      </div>
    {/if}

    <!-- TASTENKÜRZEL -->
    {@render section('hotkeys', 'Tastenkürzel')}
    {#if prefs.inspectorOpen.hotkeys}
      <div class="sec">
        <HotkeyEditor />
      </div>
    {/if}

    <!-- OBERFLÄCHE -->
    {@render section('ui', 'Oberfläche')}
    {#if prefs.inspectorOpen.ui}
      <div class="sec">
        <div class="seg" role="group" aria-label="Erscheinungsbild">
          <button aria-pressed={prefs.uiTheme === 'system'} onclick={() => store.setPrefs({ uiTheme: 'system' })}>System</button>
          <button aria-pressed={prefs.uiTheme === 'light'} onclick={() => store.setPrefs({ uiTheme: 'light' })}>Hell</button>
          <button aria-pressed={prefs.uiTheme === 'dark'} onclick={() => store.setPrefs({ uiTheme: 'dark' })}>Dunkel</button>
        </div>
        <label class="check" style="margin-top:12px"><input type="checkbox" checked={prefs.showFrameStats} onchange={() => store.setPrefs({ showFrameStats: !prefs.showFrameStats })} /> Frametime-Anzeige</label>
      </div>
    {/if}
  </aside>
{/if}

<style>
  .inspector {
    width: var(--layout-inspector);
    border-left: 1px solid var(--ui-border);
    background: var(--ui-surface);
    overflow-y: auto;
    padding-bottom: 24px;
  }
  .sec-head {
    width: 100%; display: flex; align-items: center; justify-content: space-between;
    height: 40px; padding: 0 16px; border: 0; border-top: 1px solid var(--ui-border);
    background: none; cursor: pointer; color: var(--ui-text);
  }
  .sec-head:first-of-type { border-top: 0; }
  .chev { display: inline-flex; color: var(--ui-text-muted); transition: transform var(--dur-fast) var(--ease-standard); }
  .chev.open { transform: rotate(90deg); }
  .sec { padding: 4px 16px 16px; }
  .themes { display: grid; grid-template-columns: repeat(6, 1fr); gap: 6px; }
  .tile {
    position: relative; height: 40px; border-radius: var(--radius-md); cursor: pointer;
    border: 1px solid var(--ui-border-strong); display: grid; place-items: center; padding: 0;
  }
  .tile[aria-checked='true'] { outline: 2px solid var(--ui-accent); outline-offset: 2px; }
  .tile .aa { font: 700 13px/1 var(--font-ui); }
  .tile .mk { position: absolute; left: 4px; right: 4px; bottom: 9px; height: 2px; border-radius: 1px; }
  .tile .badge { position: absolute; right: 2px; top: 1px; font: 600 8px/10px var(--font-mono); opacity: .8; }
  .val { min-width: 64px; text-align: right; font-size: 12px; color: var(--ui-text-muted); }
  .indent { margin-left: 24px; }
  .colors { display: flex; gap: 12px; }
  .colors label { display: flex; align-items: center; gap: 6px; font-size: 12px; }
  .bad { color: var(--state-lost); }
  .warn { color: var(--state-hold); }
  .meter { height: 6px; border-radius: 3px; background: var(--ui-surface-sunken); overflow: hidden; margin-bottom: 8px; }
  .meter span { display: block; height: 100%; background: var(--state-tracking); transition: width 66ms linear; }
  .transcript { font-style: italic; margin: 6px 0 0; color: var(--ui-text-muted); }
  .monitors { display: flex; gap: 8px; flex-wrap: wrap; margin-bottom: 12px; }
  .mon {
    width: 84px; height: 56px; border-radius: var(--radius-md); border: 1px solid var(--ui-border-strong);
    background: var(--ui-surface-sunken); color: var(--ui-text); cursor: pointer; display: grid; place-items: center; gap: 0;
  }
  .mon[aria-checked='true'] { border-color: var(--ui-accent); background: var(--ui-accent-subtle); }
  .big { font-size: 17px; font-weight: 600; }
  .wrap { flex-wrap: wrap; margin-bottom: 8px; }
  label.check { margin: 6px 0; }
</style>
