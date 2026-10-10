<!--
  Settings modal body (DESIGN §3.5, replaces the former right-hand Inspector):
  Tabs Darstellung · Scroll · Voice & Modelle · Tastenkürzel · Ausgabe.
  Core settings go through ReplaceSettings, UI-only prefs through
  store.setPrefs (broadcast to the talent window). Rendered inside <Dialog>,
  so hardware is (re)loaded each time the dialog opens.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { fade } from 'svelte/transition';
  import Icon from './Icon.svelte';
  import HotkeyEditor from './HotkeyEditor.svelte';
  import ModelList from './ModelList.svelte';
  import { FOCUS_LINE_MAX, FOCUS_LINE_MIN, store, type SettingsTab } from '../lib/store.svelte';
  import { api, errorText } from '../lib/bridge';
  import { setMode } from '../lib/actions';
  import { displayChord } from '../lib/hotkeys';
  import { dur, EASE } from '../lib/motion';
  import { contrastRatio, FONT_LABELS, THEME_IDS, THEME_LABELS, THEMES, tokens } from '../lib/tokens';
  import type { AudioDevice, FontChoice, ModelRow, MonitorInfo, ScrollMode, Settings } from '../lib/types';

  interface Props {
    onclose: () => void;
  }
  let { onclose }: Props = $props();

  const st = $derived(store.state);
  const prefs = $derived(store.prefs);
  const pt = tokens.type.prompter;

  const TABS: { id: SettingsTab; label: string; icon: 'type' | 'paraNext' | 'mic' | 'keyboard' | 'monitor' }[] = [
    { id: 'appearance', label: 'Darstellung', icon: 'type' },
    { id: 'scroll', label: 'Scroll', icon: 'paraNext' },
    { id: 'voice', label: 'Voice & Modelle', icon: 'mic' },
    { id: 'hotkeys', label: 'Tastenkürzel', icon: 'keyboard' },
    { id: 'output', label: 'Ausgabe', icon: 'monitor' },
  ];
  const tab = $derived(store.settingsTab);

  let monitors = $state<MonitorInfo[]>([]);
  let devices = $state<AudioDevice[]>([]);
  let models = $state<ModelRow[]>([]);
  let selectedMonitor = $state<number | null>(null);
  let micTest = $state(false);

  function update(patch: Partial<Settings>) {
    if (!st) return;
    store.send({ type: 'replaceSettings', value: { ...$state.snapshot(st.settings), ...patch } });
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

  function onTabKey(e: KeyboardEvent, i: number) {
    let n = -1;
    if (e.key === 'ArrowDown' || e.key === 'ArrowRight') n = (i + 1) % TABS.length;
    else if (e.key === 'ArrowUp' || e.key === 'ArrowLeft') n = (i - 1 + TABS.length) % TABS.length;
    else if (e.key === 'Home') n = 0;
    else if (e.key === 'End') n = TABS.length - 1;
    if (n < 0) return;
    e.preventDefault();
    store.settingsTab = TABS[n].id;
    document.getElementById(`set-tab-${TABS[n].id}`)?.focus();
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

{#if st}
  <div class="settings">
    <header class="head">
      <h2 id="settings-title">Einstellungen</h2>
      <span class="kbd">{displayChord('Ctrl+,')}</span>
      <span class="spacer"></span>
      <button class="btn icon round" aria-label="Einstellungen schließen (Esc)" title="Schließen (Esc)" onclick={onclose}><Icon name="x" /></button>
    </header>

    <div class="cols">
      <div class="tabs" role="tablist" aria-orientation="vertical" aria-label="Bereiche">
        {#each TABS as t, i (t.id)}
          <button
            id="set-tab-{t.id}"
            role="tab"
            class="tab"
            aria-selected={tab === t.id}
            aria-controls="set-panel"
            tabindex={tab === t.id ? 0 : -1}
            data-autofocus={tab === t.id ? '' : undefined}
            onclick={() => (store.settingsTab = t.id)}
            onkeydown={(e) => onTabKey(e, i)}
          >
            <Icon name={t.icon} /><span>{t.label}</span>
          </button>
        {/each}
      </div>

      <div class="body" id="set-panel" role="tabpanel" aria-labelledby="set-tab-{tab}" tabindex="-1">
        {#key tab}
          <div class="pane" in:fade={{ duration: dur('base'), easing: EASE.enter }}>
            {#if tab === 'appearance'}
              <section>
                <h3 class="caption">Prompter-Theme</h3>
                <div class="themes" role="radiogroup" aria-label="Theme">
                  {#each THEME_IDS as id, i (id)}
                    {@const t = THEMES[id]}
                    <button
                      class="tile"
                      role="radio"
                      aria-checked={st.settings.theme === id}
                      aria-label="{THEME_LABELS[id]}, Kontrast {fmtRatio(t.contrast)}"
                      title="{THEME_LABELS[id]} · {fmtRatio(t.contrast)} {t.wcag} · {displayChord(`Ctrl+${i + 1}`)}"
                      onclick={() => store.send({ type: 'setTheme', value: id })}
                      style:background={t.bg}
                      style:color={t.text}
                    >
                      <span class="aa">Aa</span>
                      <span class="mk" style:background={t.marker}></span>
                    </button>
                  {/each}
                </div>
                <p class="footnote">{THEME_LABELS[st.settings.theme] ?? st.settings.theme} · {fmtRatio(theme.contrast)} {theme.wcag}</p>
              </section>

              <section>
                <h3 class="caption">Schrift</h3>
                <label class="field">
                  <span class="lbl">Schriftart</span>
                  <select value={prefs.fontFamily} onchange={(e) => store.setPrefs({ fontFamily: (e.currentTarget as HTMLSelectElement).value as FontChoice })}>
                    {#each Object.entries(FONT_LABELS) as [k, v] (k)}<option value={k}>{v}</option>{/each}
                  </select>
                </label>
                <div class="field">
                  <span class="lbl" id="lbl-size">Größe</span>
                  <div class="field-row">
                    <input type="range" min={pt.sizeMin} max={pt.sizeMax} step="2" aria-labelledby="lbl-size"
                      value={st.settings.fontSizePx} aria-valuetext="{st.settings.fontSizePx} Pixel"
                      oninput={(e) => store.send({ type: 'setFontSize', value: Number((e.currentTarget as HTMLInputElement).value) })} />
                    <input type="number" min={pt.sizeMin} max={pt.sizeMax} value={st.settings.fontSizePx} aria-label="Schriftgröße in Pixel"
                      onchange={(e) => store.send({ type: 'setFontSize', value: Number((e.currentTarget as HTMLInputElement).value) })} />
                  </div>
                </div>
                <div class="field">
                  <span class="lbl" id="lbl-measure">Spaltenbreite</span>
                  <div class="field-row">
                    <input type="range" min="16" max="60" aria-labelledby="lbl-measure" value={prefs.measureCh} aria-valuetext="{prefs.measureCh} Zeichen"
                      oninput={(e) => store.setPrefs({ measureCh: Number((e.currentTarget as HTMLInputElement).value) })} />
                    <span class="num val">{prefs.measureCh} ch</span>
                  </div>
                  <span class="footnote">Empfohlen 25–40 Zeichen pro Zeile</span>
                </div>
                <div class="field">
                  <span class="lbl" id="lbl-lh">Zeilenabstand</span>
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
              </section>

              <section>
                <h3 class="caption">Farben</h3>
                <div class="colors">
                  <label title="Text"><input type="color" value={effText} oninput={(e) => store.setPrefs({ textColor: (e.currentTarget as HTMLInputElement).value })} /> Text</label>
                  <label title="Hintergrund"><input type="color" value={effBg} oninput={(e) => store.setPrefs({ bgColor: (e.currentTarget as HTMLInputElement).value })} /> Hintergrund</label>
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
              </section>

              <section>
                <h3 class="caption">Lesemarker</h3>
                <div class="seg" role="group" aria-label="Marker-Stil">
                  <button aria-pressed={prefs.markerStyle === 'line'} onclick={() => store.setPrefs({ markerStyle: 'line' })}>Linie</button>
                  <button aria-pressed={prefs.markerStyle === 'band'} onclick={() => store.setPrefs({ markerStyle: 'band' })}>Band</button>
                </div>
                <div class="field" style="margin-top:12px">
                  <span class="lbl" id="lbl-focus">Position</span>
                  <div class="field-row">
                    <input type="range" min={FOCUS_LINE_MIN} max={FOCUS_LINE_MAX} step="0.01" aria-labelledby="lbl-focus" value={st.settings.focusLine}
                      aria-valuetext="{Math.round(st.settings.focusLine * 100)} Prozent"
                      onchange={(e) => update({ focusLine: Number((e.currentTarget as HTMLInputElement).value) })} />
                    <span class="num val">{Math.round(st.settings.focusLine * 100)} %</span>
                  </div>
                  <span class="footnote">Auch per Drag auf dem Marker in der Vorschau.</span>
                </div>
              </section>

              <section>
                <h3 class="caption">Oberfläche</h3>
                <div class="seg" role="group" aria-label="Erscheinungsbild">
                  <button aria-pressed={prefs.uiTheme === 'system'} onclick={() => store.setPrefs({ uiTheme: 'system' })}>System</button>
                  <button aria-pressed={prefs.uiTheme === 'light'} onclick={() => store.setPrefs({ uiTheme: 'light' })}>Hell</button>
                  <button aria-pressed={prefs.uiTheme === 'dark'} onclick={() => store.setPrefs({ uiTheme: 'dark' })}>Dunkel</button>
                </div>
                <label class="check" style="margin-top:12px"><input type="checkbox" checked={prefs.showFrameStats} onchange={() => store.setPrefs({ showFrameStats: !prefs.showFrameStats })} /> Frametime-Anzeige</label>
              </section>
            {:else if tab === 'scroll'}
              <section>
                <label class="field">
                  <span class="lbl">Modus</span>
                  <select value={st.scroll.mode} onchange={(e) => setMode((e.currentTarget as HTMLSelectElement).value as ScrollMode)}>
                    {#each scrollModes as m (m.id)}<option value={m.id}>{m.label}</option>{/each}
                  </select>
                </label>
                <div class="field">
                  <span class="lbl" id="lbl-wpm">Tempo</span>
                  <div class="field-row">
                    <input type="range" min="40" max="400" step="5" aria-labelledby="lbl-wpm" value={st.scroll.wpm} aria-valuetext="{st.scroll.wpm} Wörter pro Minute"
                      oninput={(e) => store.send({ type: 'setWpm', value: Number((e.currentTarget as HTMLInputElement).value) })} />
                    <span class="num val">{st.scroll.wpm} WPM</span>
                  </div>
                  <span class="footnote">Gilt für „Konstant“ und als Rückfall bei „Verloren“.</span>
                </div>
                <label class="field">
                  <span class="lbl">Countdown</span>
                  <select value={String(st.settings.countdownSecs)} onchange={(e) => update({ countdownSecs: Number((e.currentTarget as HTMLSelectElement).value) })}>
                    <option value="0">Aus</option><option value="3">3 s</option><option value="5">5 s</option>
                  </select>
                </label>
                <label class="check"><input type="checkbox" checked={st.settings.reducedMotion} onchange={() => update({ reducedMotion: !st.settings.reducedMotion })} /> Bewegung reduzieren (auch UI-Animationen)</label>
              </section>
            {:else if tab === 'voice'}
              <section>
                <label class="field">
                  <span class="lbl">Sprache</span>
                  <select value={st.settings.language} onchange={(e) => update({ language: (e.currentTarget as HTMLSelectElement).value })}>
                    <option value="de">Deutsch</option><option value="en">English</option>
                  </select>
                </label>
                <label class="field">
                  <span class="lbl">Mikrofon</span>
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
                  <span class="lbl">Wenn verloren</span>
                  <select value={st.settings.lostPolicy} onchange={(e) => update({ lostPolicy: (e.currentTarget as HTMLSelectElement).value as Settings['lostPolicy'] })}>
                    <option value="stop">Anhalten</option><option value="constantSpeed">Konstant weiter</option>
                  </select>
                </label>
              </section>
              <section>
                <h3 class="caption">Modelle</h3>
                <ModelList {models} onchange={refreshHardware} />
              </section>
            {:else if tab === 'hotkeys'}
              <section>
                <HotkeyEditor />
                <p class="footnote">Feste Kürzel: {displayChord('Ctrl+,')} Einstellungen · {displayChord('Ctrl+B')} Skripte · {displayChord('Ctrl+K')} Befehle</p>
              </section>
            {:else if tab === 'output'}
              <section>
                <h3 class="caption">Monitor</h3>
                <div class="monitors" role="radiogroup" aria-label="Ausgabe-Monitor">
                  {#each monitors as m (m.index)}
                    <button class="mon" role="radio" aria-checked={selectedMonitor === m.index} onclick={() => chooseMonitor(m.index)} title="{m.name} · {m.width}×{m.height}">
                      <span class="num big">{m.index + 1}</span>
                      <span class="footnote">{m.width}×{m.height}{m.primary ? ' · Haupt' : ''}</span>
                    </button>
                  {:else}
                    <p class="footnote">Keine Monitore gefunden.</p>
                  {/each}
                </div>
                <div class="field-row wrap">
                  <button class="btn outline" onclick={() => (store.talentVisible ? api.closeTalent() : api.openTalent(selectedMonitor, false))}>
                    {store.talentVisible ? 'Talent-Fenster schließen' : 'Talent-Fenster öffnen'}
                  </button>
                  <label class="check"><input type="checkbox" checked={st.output.fullscreen} onchange={() => api.setTalentFullscreen(!st.output.fullscreen).catch(() => {})} /> Vollbild</label>
                </div>
              </section>
              <section>
                <h3 class="caption">Spiegeln</h3>
                <label class="check"><input type="checkbox" checked={st.output.mirrorH} onchange={() => store.send({ type: 'setMirror', value: { h: !st.output.mirrorH, v: st.output.mirrorV } })} /> Horizontal spiegeln</label>
                <label class="check"><input type="checkbox" checked={st.output.mirrorV} onchange={() => store.send({ type: 'setMirror', value: { h: st.output.mirrorH, v: !st.output.mirrorV } })} /> Vertikal spiegeln</label>
                <button class="btn outline" style="margin-top:8px" onclick={refreshHardware}>Monitore & Geräte neu laden</button>
              </section>
            {/if}
          </div>
        {/key}
      </div>
    </div>
  </div>
{/if}

<style>
  .settings { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .head { display: flex; align-items: center; gap: 10px; padding: 16px 16px 12px 24px; border-bottom: 1px solid var(--ui-border); }
  .head h2 { margin: 0; font: 600 17px/24px var(--font-ui); letter-spacing: -0.01em; }
  .spacer { flex: 1; }
  .round { border-radius: var(--radius-pill); }
  .cols { flex: 1; min-height: 0; display: grid; grid-template-columns: 200px 1fr; }
  .tabs { display: flex; flex-direction: column; gap: 2px; padding: 12px; border-right: 1px solid var(--ui-border); overflow-y: auto; min-height: 0; }
  .tab {
    display: flex; align-items: center; gap: 10px; height: 34px; padding: 0 10px;
    border: 0; border-radius: var(--radius-sm); background: transparent; color: var(--ui-text-muted);
    font: 500 13px/20px var(--font-ui); text-align: left; cursor: pointer;
    transition: background var(--dur-fast) var(--ease-standard), color var(--dur-fast) var(--ease-standard), transform var(--dur-fast) var(--ease-standard);
  }
  .tab:hover { background: var(--ui-surface-sunken); color: var(--ui-text); }
  .tab:active { transform: scale(.98); }
  .tab[aria-selected='true'] { background: var(--ui-accent-subtle); color: var(--ui-text); font-weight: 600; }
  .body { min-height: 0; overflow-y: auto; overscroll-behavior: contain; padding: 8px 24px 24px; }
  .body:focus { outline: none; }
  section { padding: 16px 0; border-bottom: 1px solid var(--ui-border); }
  section:last-child { border-bottom: 0; }
  section h3 { margin: 0 0 10px; }
  .lbl { font: 500 12px/16px var(--font-ui); color: var(--ui-text-muted); }
  .themes { display: grid; grid-template-columns: repeat(6, 1fr); gap: 8px; }
  .tile {
    position: relative; height: 52px; border-radius: var(--radius-md); cursor: pointer;
    border: 1px solid var(--ui-border-strong); display: grid; place-items: center; padding: 0;
    transition: transform var(--dur-fast) var(--ease-standard), box-shadow var(--dur-fast) var(--ease-standard);
  }
  .tile:hover { transform: translateY(-1px); box-shadow: var(--elev-2); }
  .tile:active { transform: scale(.97); }
  .tile[aria-checked='true'] { outline: 2px solid var(--ui-accent); outline-offset: 2px; }
  .tile .aa { font: 700 15px/1 var(--font-ui); }
  .tile .mk { position: absolute; left: 8px; right: 8px; bottom: 10px; height: 2px; border-radius: 1px; }
  .val { min-width: 64px; text-align: right; font-size: 12px; color: var(--ui-text-muted); }
  .indent { margin-left: 24px; }
  .colors { display: flex; gap: 16px; margin-bottom: 8px; }
  .colors label { display: flex; align-items: center; gap: 6px; font-size: 12px; }
  .bad { color: var(--state-lost); }
  .warn { color: var(--state-hold); }
  .meter { height: 6px; border-radius: 3px; background: var(--ui-surface-sunken); overflow: hidden; margin-bottom: 8px; }
  .meter span { display: block; height: 100%; background: var(--state-tracking); transition: width 66ms linear; }
  .transcript { font-style: italic; margin: 6px 0 0; color: var(--ui-text-muted); }
  .monitors { display: flex; gap: 8px; flex-wrap: wrap; margin-bottom: 12px; }
  .mon {
    width: 96px; height: 64px; border-radius: var(--radius-md); border: 1px solid var(--ui-border-strong);
    background: var(--ui-surface-sunken); color: var(--ui-text); cursor: pointer; display: grid; place-items: center; gap: 0;
    transition: border-color var(--dur-fast) var(--ease-standard), background var(--dur-fast) var(--ease-standard), transform var(--dur-fast) var(--ease-standard);
  }
  .mon:active { transform: scale(.97); }
  .mon[aria-checked='true'] { border-color: var(--ui-accent); background: var(--ui-accent-subtle); }
  .big { font-size: 17px; font-weight: 600; }
  .wrap { flex-wrap: wrap; margin-bottom: 8px; }
  label.check { margin: 6px 0; }
  @media (max-width: 720px) {
    .cols { grid-template-columns: 1fr; grid-template-rows: auto 1fr; }
    .tabs { flex-direction: row; overflow-x: auto; border-right: 0; border-bottom: 1px solid var(--ui-border); }
    .tab span { white-space: nowrap; }
  }
</style>
