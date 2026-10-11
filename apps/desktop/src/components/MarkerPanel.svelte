<!--
  Settings tab "Lesemarker" (DESIGN §3.7): style tiles with mini previews,
  live preview, colour (picker + presets from the theme accent), opacity,
  size, position, dimming, glow/blur and animation. Writes core settings
  (`Settings.marker`, `focusLine`) via replaceSettings.
-->
<script lang="ts">
  import ReadingMarker from './ReadingMarker.svelte';
  import { FOCUS_LINE_MAX, FOCUS_LINE_MIN, store } from '../lib/store.svelte';
  import { updateMarker } from '../lib/actions';
  import {
    colorPresets,
    MARKER_BOUNDS,
    MARKER_STYLES,
    markerColor,
    markerVars,
    sanitizeMarker,
    type MarkerAnimation,
    type MarkerStyle,
  } from '../lib/marker';
  import { prompterVars, THEMES } from '../lib/tokens';

  const st = $derived(store.state);
  const prefs = $derived(store.prefs);
  const m = $derived(sanitizeMarker(st?.settings.marker));
  const theme = $derived(THEMES[st?.settings.theme ?? 'studioDark'] ?? THEMES.studioDark);
  const color = $derived(markerColor(m, prefs.markerColor, theme));
  const presets = $derived(colorPresets(theme));
  const reduced = $derived(!!st?.settings.reducedMotion);
  const previewVars = $derived(
    st
      ? prompterVars(st.settings.theme, prefs, 72, st.settings.lineHeight, st.settings) +
          ';' +
          markerVars(m, color, prefs.dimRead, reduced) +
          `;--pr-scale:0.28;--focus:0.5`
      : '',
  );
  const pct = (v: number) => `${Math.round(v * 100)} %`;
  const num = (e: Event) => Number((e.currentTarget as HTMLInputElement).value);

  function setFocus(f: number) {
    if (!st) return;
    store.send({ type: 'replaceSettings', value: { ...JSON.parse(JSON.stringify(st.settings)), focusLine: f } });
  }

  function tileKey(e: KeyboardEvent, i: number) {
    const cols = 3;
    const n = MARKER_STYLES.length;
    const d = { ArrowRight: 1, ArrowLeft: -1, ArrowDown: cols, ArrowUp: -cols }[e.key];
    if (d === undefined) return;
    e.preventDefault();
    const j = (i + d + n) % n;
    updateMarker({ style: MARKER_STYLES[j].id });
    (document.getElementById(`mk-tile-${MARKER_STYLES[j].id}`) as HTMLElement | null)?.focus();
  }

  const anims: { id: MarkerAnimation; label: string }[] = [
    { id: 'none', label: 'Aus' },
    { id: 'glide', label: 'Gleiten' },
    { id: 'pulse', label: 'Puls' },
  ];
  const usesSize = (s: MarkerStyle) => s !== 'none' && s !== 'spotlight';
  const usesBlur = (s: MarkerStyle) => s === 'spotlight' || s === 'gradient';
</script>

{#snippet mini(id: MarkerStyle)}
  <svg viewBox="0 0 80 48" width="80" height="48" aria-hidden="true">
    {#each [8, 18, 28, 38] as y, k (y)}
      <rect x="14" {y} width={k === 1 ? 44 : 52} height="4" rx="2" fill="currentColor" opacity={k === 0 ? 0.35 : 0.7} />
    {/each}
    {#if id === 'line'}<rect x="10" y="25" width="60" height="1.5" fill="var(--mk-c)" />
    {:else if id === 'band'}<rect x="10" y="15" width="60" height="10" fill="var(--mk-c)" opacity="0.2" stroke="var(--mk-c)" stroke-opacity="0.5" stroke-width="0.6" />
    {:else if id === 'arrows'}<path d="M2 15l6 5-6 5zM78 15l-6 5 6 5z" fill="var(--mk-c)" />
    {:else if id === 'brackets'}<path d="M13 14h-3v12h3M67 14h3v12h-3" fill="none" stroke="var(--mk-c)" stroke-width="1.5" />
    {:else if id === 'spotlight'}<rect x="0" y="0" width="80" height="14" fill="#000" opacity="0.55" /><rect x="0" y="26" width="80" height="22" fill="#000" opacity="0.55" />
    {:else if id === 'gradient'}<defs><linearGradient id="mk-g" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#000" stop-opacity="0.7" /><stop offset="0.4" stop-color="#000" stop-opacity="0" /><stop offset="0.6" stop-color="#000" stop-opacity="0" /><stop offset="1" stop-color="#000" stop-opacity="0.7" /></linearGradient></defs><rect width="80" height="48" fill="url(#mk-g)" />
    {:else if id === 'underline'}<rect x="24" y="23.5" width="16" height="1.5" fill="var(--mk-c)" />
    {:else if id === 'sidebar'}<rect x="8" y="14" width="2.5" height="12" rx="1" fill="var(--mk-c)" />
    {/if}
  </svg>
{/snippet}

{#if st}
  <section>
    <h3 class="caption" id="mk-style-h">Stil</h3>
    <div class="tiles" role="radiogroup" aria-labelledby="mk-style-h" style:--mk-c={color}>
      {#each MARKER_STYLES as s, i (s.id)}
        <button
          id="mk-tile-{s.id}"
          class="mk-tile"
          role="radio"
          aria-checked={m.style === s.id}
          tabindex={m.style === s.id ? 0 : -1}
          title={s.hint}
          onclick={() => updateMarker({ style: s.id })}
          onkeydown={(e) => tileKey(e, i)}
        >
          <span class="mk-mini" style:background={theme.bg} style:color={theme.text}>{@render mini(s.id)}</span>
          <span class="mk-label">{s.label}</span>
        </button>
      {/each}
    </div>
    {#if m.style === 'underline'}
      <div class="seg" role="group" aria-label="Unterstreichung" style="margin-top:10px">
        <button aria-pressed={m.underlineScope === 'word'} onclick={() => updateMarker({ underlineScope: 'word' })}>Wort</button>
        <button aria-pressed={m.underlineScope === 'sentence'} onclick={() => updateMarker({ underlineScope: 'sentence' })}>Satz</button>
      </div>
    {/if}
  </section>

  <section>
    <h3 class="caption">Vorschau</h3>
    <div class="preview" style={previewVars} role="img" aria-label="Live-Vorschau des Lesemarkers: {MARKER_STYLES.find((x) => x.id === m.style)?.label}">
      <div class="pv-text" aria-hidden="true">
        <p class="read">Guten Abend und willkommen.</p>
        <p class="read">Heute sprechen wir über</p>
        <p class="cur">das <span class:ul={m.style === 'underline'}>Wetter</span><span class:ul={m.style === 'underline' && m.underlineScope === 'sentence'}> von morgen.</span></p>
        <p>Im Norden bleibt es trocken,</p>
        <p>im Süden zieht Regen auf.</p>
      </div>
      <ReadingMarker marker={m} focus={0.5} {reduced} />
    </div>
  </section>

  <section>
    <h3 class="caption">Farbe</h3>
    <div class="field-row presets" role="group" aria-label="Farbvorlagen">
      {#each presets as c, k (c)}
        <button
          class="swatch"
          style:background={c}
          aria-label="{k === 0 ? 'Theme-Akzent ' : ''}{c}"
          aria-pressed={color.toLowerCase() === c}
          onclick={() => updateMarker({ color: k === 0 ? null : c })}
        ></button>
      {/each}
      <label class="picker" title="Eigene Farbe">
        <input type="color" value={color} aria-label="Eigene Markerfarbe" onchange={(e) => updateMarker({ color: (e.currentTarget as HTMLInputElement).value })} />
      </label>
      {#if m.color}<button class="btn" onclick={() => updateMarker({ color: null })}>Theme-Farbe</button>{/if}
    </div>
  </section>

  <section>
    <h3 class="caption">Form</h3>
    <div class="field">
      <span class="lbl" id="mk-op">{m.style === 'spotlight' || m.style === 'gradient' ? 'Abdunklung' : 'Deckkraft'}</span>
      <div class="field-row">
        <input type="range" min={MARKER_BOUNDS.opacity[0]} max={MARKER_BOUNDS.opacity[1]} step="0.05" aria-labelledby="mk-op" value={m.opacity} aria-valuetext={pct(m.opacity)} onchange={(e) => updateMarker({ opacity: num(e) })} />
        <span class="num val">{pct(m.opacity)}</span>
      </div>
    </div>
    {#if usesSize(m.style)}
      <div class="field">
        <span class="lbl" id="mk-size">Dicke / Höhe</span>
        <div class="field-row">
          <input type="range" min={MARKER_BOUNDS.size[0]} max={MARKER_BOUNDS.size[1]} step="0.1" aria-labelledby="mk-size" value={m.size} aria-valuetext="{m.size.toFixed(1)}-fach" onchange={(e) => updateMarker({ size: num(e) })} />
          <span class="num val">{m.size.toFixed(1)}×</span>
        </div>
      </div>
    {/if}
    <div class="field">
      <span class="lbl" id="mk-pos">Position</span>
      <div class="field-row">
        <input type="range" min={FOCUS_LINE_MIN} max={FOCUS_LINE_MAX} step="0.01" aria-labelledby="mk-pos" value={st.settings.focusLine} aria-valuetext="{pct(st.settings.focusLine)} der Höhe" onchange={(e) => setFocus(num(e))} />
        <span class="num val">{pct(st.settings.focusLine)}</span>
      </div>
      <span class="footnote">Auch per Drag auf dem Marker in der Vorschau.</span>
    </div>
  </section>

  <section>
    <h3 class="caption">Dimmen &amp; Effekte</h3>
    <div class="field">
      <span class="lbl" id="mk-dr">Gelesenen Text dimmen</span>
      <div class="field-row">
        <input type="range" min="0" max={MARKER_BOUNDS.dim[1]} step="0.05" aria-labelledby="mk-dr" disabled={!prefs.dimRead}
          value={m.dimRead ?? 1 - theme.readOpacity} aria-valuetext={pct(m.dimRead ?? 1 - theme.readOpacity)} onchange={(e) => updateMarker({ dimRead: num(e) })} />
        <span class="num val">{pct(m.dimRead ?? 1 - theme.readOpacity)}</span>
        {#if m.dimRead != null}<button class="btn" onclick={() => updateMarker({ dimRead: null })}>Theme</button>{/if}
      </div>
      {#if !prefs.dimRead}<span class="footnote">Dimmen ist unter Darstellung ausgeschaltet.</span>{/if}
    </div>
    <div class="field">
      <span class="lbl" id="mk-du">Ungelesenen Text dimmen</span>
      <div class="field-row">
        <input type="range" min="0" max={MARKER_BOUNDS.dim[1]} step="0.05" aria-labelledby="mk-du" value={m.dimUnread} aria-valuetext={pct(m.dimUnread)} onchange={(e) => updateMarker({ dimUnread: num(e) })} />
        <span class="num val">{pct(m.dimUnread)}</span>
      </div>
    </div>
    <div class="field">
      <span class="lbl" id="mk-glow">Glow</span>
      <div class="field-row">
        <input type="range" min="0" max={MARKER_BOUNDS.glow[1]} step="0.05" aria-labelledby="mk-glow" value={m.glow} aria-valuetext={pct(m.glow)} onchange={(e) => updateMarker({ glow: num(e) })} />
        <span class="num val">{pct(m.glow)}</span>
      </div>
    </div>
    {#if usesBlur(m.style)}
      <div class="field">
        <span class="lbl" id="mk-blur">Weichzeichnen außerhalb</span>
        <div class="field-row">
          <input type="range" min="0" max={MARKER_BOUNDS.blur[1]} step="0.5" aria-labelledby="mk-blur" value={m.blur} aria-valuetext="{m.blur} Pixel" onchange={(e) => updateMarker({ blur: num(e) })} />
          <span class="num val">{m.blur} px</span>
        </div>
        <span class="footnote">Kostet GPU-Leistung (backdrop-filter); auf schwachen Geräten aus lassen.</span>
      </div>
    {/if}
    <span class="lbl" id="mk-anim">Animation beim Weiterspringen</span>
    <div class="seg" role="group" aria-labelledby="mk-anim" style="margin-top:6px">
      {#each anims as a (a.id)}
        <button aria-pressed={m.animation === a.id} onclick={() => updateMarker({ animation: a.id })}>{a.label}</button>
      {/each}
    </div>
    {#if reduced}<p class="footnote">„Bewegung reduzieren“ ist aktiv: Animationen sind aus.</p>{/if}
  </section>
{/if}

<style>
  section { padding: 16px 0; border-bottom: 1px solid var(--ui-border); }
  section:last-child { border-bottom: 0; }
  section h3 { margin: 0 0 10px; }
  .lbl { font: 500 12px/16px var(--font-ui); color: var(--ui-text-muted); }
  .val { min-width: 48px; text-align: right; }
  .tiles { display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; }
  .mk-tile {
    display: grid; gap: 4px; justify-items: center; padding: 6px; cursor: pointer;
    border: 1px solid var(--ui-border-strong); border-radius: var(--radius-md);
    background: var(--ui-surface-raised); color: var(--ui-text); font: 500 12px/16px var(--font-ui);
    transition: transform var(--dur-fast) var(--ease-standard), box-shadow var(--dur-fast) var(--ease-standard);
  }
  .mk-tile:hover { transform: translateY(-1px); box-shadow: var(--elev-2); }
  .mk-tile[aria-checked='true'] { outline: 2px solid var(--ui-accent); outline-offset: 2px; }
  .mk-mini { display: block; border-radius: var(--radius-sm); line-height: 0; overflow: hidden; }
  .preview {
    position: relative; height: 150px; overflow: hidden; border-radius: var(--radius-md);
    background: var(--pr-bg); color: var(--pr-text); border: 1px solid var(--ui-border);
  }
  .pv-text {
    position: absolute; left: 0; right: 0; top: calc(50% - 2.5em * var(--pr-line-height));
    font: var(--pr-weight) calc(var(--pr-size) * var(--pr-scale)) / var(--pr-line-height) var(--pr-font);
    text-align: center;
  }
  .pv-text p { margin: 0; }
  .pv-text p.read { opacity: var(--pr-read-opacity); }
  .pv-text .ul { text-decoration: underline; text-decoration-color: var(--pr-marker); text-decoration-thickness: calc(0.07em * var(--mk-size)); text-underline-offset: 0.12em; }
  .presets { flex-wrap: wrap; }
  .swatch { width: 24px; height: 24px; border-radius: 50%; border: 1px solid var(--ui-border-strong); cursor: pointer; padding: 0; }
  .swatch[aria-pressed='true'] { outline: 2px solid var(--ui-accent); outline-offset: 2px; }
  .picker input { width: 32px; height: 26px; padding: 0; border: 0; background: none; cursor: pointer; }
</style>
