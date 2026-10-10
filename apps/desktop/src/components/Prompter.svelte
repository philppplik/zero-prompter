<!--
  Prompter renderer shared by the talent window (/talent, scale 1, mirrored)
  and the operator preview (scaled, unmirrored, interactive).
  DESIGN §3.6/3.7/3.9: focus line, read-dimming, word highlight, hold ring,
  countdown, end marker, "no script" glyph. Pure view: all state comes from Rust.
  Attribution: the current-word highlight follows the UX idea of f/textream
  (MIT, https://github.com/f/textream) – concept only, no code copied; see THIRD_PARTY_NOTICES.md.
-->
<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { Scroller, type LayoutMap } from '../lib/scroller';
  import { clampFocusLine, FOCUS_LINE_MAX, FOCUS_LINE_MIN, frame, store } from '../lib/store.svelte';
  import { api } from '../lib/bridge';
  import { prompterVars } from '../lib/tokens';
  import type { AppState, Token, UiPrefs } from '../lib/types';

  interface Props {
    app: AppState;
    prefs: UiPrefs;
    /** Font scale relative to the talent output (preview = width / 1920). */
    scale?: number;
    mirror?: boolean;
    interactive?: boolean;
    /** Only the talent window owns the layout report (CONCEPT §8.3). */
    report?: boolean;
    onword?: (token: number) => void;
    onfocusdrag?: (focus: number) => void;
  }
  let { app, prefs, scale = 1, mirror = false, interactive = false, report = false, onword, onfocusdrag }: Props = $props();

  let viewport: HTMLDivElement;
  let content: HTMLDivElement;
  let dim: HTMLDivElement;
  let hl: HTMLDivElement;
  let scroller: Scroller | null = null;
  let layoutRev = $state(0);
  let stats = $state<{ fps: number; worst: number } | null>(null);

  const reduced = () =>
    app.settings.reducedMotion || (typeof matchMedia !== 'undefined' && matchMedia('(prefers-reduced-motion: reduce)').matches);

  const paragraphs = $derived(
    app.script.paragraphs.map((p) => app.script.tokens.slice(p.tokens.start, p.tokens.end)),
  );
  const isMdMark = (t: Token) => /^#{1,6}$/.test(t.text) || /^[-*>]$/.test(t.text);
  const isHeading = (toks: Token[]) => toks.length > 0 && /^#{1,6}$/.test(toks[0].text);

  const style = $derived(
    prompterVars(app.settings.theme, prefs, app.settings.fontSizePx, app.settings.lineHeight) +
      `;--pr-scale:${scale};--focus:${app.settings.focusLine}`,
  );
  const mirrorTransform = $derived(
    mirror ? `scale(${app.output.mirrorH ? -1 : 1}, ${app.output.mirrorV ? -1 : 1})` : 'none',
  );
  const countdown = $derived(app.playback.state.kind === 'countdown' ? app.playback.state.remaining : 0);
  const ended = $derived(app.playback.state.kind === 'ended');
  const voiceOn = $derived(app.voice.enabled);
  const hold = $derived(voiceOn && app.voice.phase === 'hold');

  async function remeasure() {
    if (!scroller) return;
    await tick();
    const layout = scroller.measure(app.script.tokens.length);
    layoutRev++;
    scroller.render();
    if (report) {
      api.reportLayout(app.script.version, layout.lines.length).catch(() => {});
    }
  }

  // Re-measure whenever anything that affects wrapping changes.
  $effect(() => {
    void [app.script.version, app.settings.fontSizePx, app.settings.lineHeight, app.settings.theme, prefs.fontFamily, prefs.measureCh, scale];
    remeasure();
  });

  // Snap (no animation) on script change.
  $effect(() => {
    void app.script.version;
    scroller?.snap(frame.position);
  });

  function placeDim(pos: number, layout: LayoutMap | null) {
    if (!dim || !layout || layout.lines.length === 0) {
      if (dim) dim.style.height = '0px';
      return;
    }
    let i = Math.max(0, Math.min(layout.lineOf.length - 1, Math.floor(pos)));
    while (i > 0 && layout.lineOf[i] < 0) i--;
    const li = layout.lineOf[i];
    const line = li >= 0 ? layout.lines[li] : null;
    if (!line) return;
    const off = scroller?.offset ?? 0;
    dim.style.height = `${Math.max(0, line.top - off)}px`;
  }

  // Voice word highlight (moves with the content layer; 120 ms transition).
  $effect(() => {
    void layoutRev;
    const w = store.word;
    const show = voiceOn && w && ['tracking', 'resync', 'hold'].includes(app.voice.phase);
    const l = scroller?.layout;
    if (!hl) return;
    if (!show || !l || !w || w.token >= l.lineOf.length) {
      hl.style.opacity = '0';
      return;
    }
    const b = l.boxes;
    const k = w.token * 4;
    hl.style.opacity = '1';
    hl.style.transform = `translate(${b[k] - 4}px, ${b[k + 1]}px)`;
    hl.style.width = `${b[k + 2] + 8}px`;
    hl.style.height = `${b[k + 3]}px`;
  });

  function onClick(e: MouseEvent) {
    if (!interactive || !onword) return;
    const el = (e.target as HTMLElement).closest<HTMLElement>('[data-t]');
    if (el) onword(Number(el.dataset.t));
  }

  function onWheel(e: WheelEvent) {
    if (!interactive) return;
    e.preventDefault();
    // Mouse wheel = manual override (Rust starts the 2 s voice hold).
    const step = Math.sign(e.deltaY) * Math.max(1, Math.round(Math.abs(e.deltaY) / 40));
    const n = app.script.tokens.length;
    const to = Math.max(0, Math.min(n - 1, store.currentToken + step));
    store.send({ type: 'seekToken', value: to });
  }

  // Marker drag (preview only): 5–70 % of the height (DESIGN §3.7).
  let dragging = false;
  function markerDown(e: PointerEvent) {
    if (!onfocusdrag) return;
    dragging = true;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }
  function markerMove(e: PointerEvent) {
    if (!dragging || !onfocusdrag) return;
    const r = viewport.getBoundingClientRect();
    onfocusdrag(clampFocusLine(Math.round(((e.clientY - r.top) / r.height) * 100) / 100));
  }
  function markerUp() {
    dragging = false;
  }
  function markerKey(e: KeyboardEvent) {
    if (!onfocusdrag) return;
    const d = e.key === 'ArrowUp' ? -0.01 : e.key === 'ArrowDown' ? 0.01 : 0;
    if (d) {
      e.preventDefault();
      e.stopPropagation();
      onfocusdrag(clampFocusLine(Math.round((app.settings.focusLine + d) * 100) / 100));
    }
  }

  onMount(() => {
    scroller = new Scroller({
      content,
      viewport,
      source: frame,
      focusLine: () => app.settings.focusLine,
      reducedMotion: reduced,
      onFrame: placeDim,
      onStats: (fps, worst) => {
        if (prefs.showFrameStats) stats = { fps, worst };
        else if (stats) stats = null;
      },
    });
    scroller.snap(frame.position);
    remeasure();
    scroller.start();
    // Non-passive so preventDefault works (Svelte may attach wheel listeners passively).
    const wheel = (e: WheelEvent) => onWheel(e);
    viewport.addEventListener('wheel', wheel, { passive: false });
    const ro = new ResizeObserver(() => remeasure());
    ro.observe(viewport);
    document.fonts?.ready.then(() => remeasure());
    return () => {
      ro.disconnect();
      viewport.removeEventListener('wheel', wheel);
      scroller?.stop();
    };
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions -->
<div
  class="viewport"
  class:interactive
  bind:this={viewport}
  {style}
  onclick={onClick}
  aria-label="Teleprompter-Text"
  role="document"
>
  <div class="stage" style:transform={mirrorTransform}>
    <div class="content" bind:this={content} lang={app.settings.language}>
      <div class="hl" bind:this={hl} aria-hidden="true"></div>
      {#if app.script.tokens.length === 0}
        <div class="empty" aria-label="Kein Skript geladen">
          <svg viewBox="0 0 64 64" width="160" height="160" aria-hidden="true">
            <rect x="8" y="14" width="48" height="36" rx="6" fill="none" stroke="currentColor" stroke-width="3" />
            <path d="M18 26h28M18 34h20M18 42h24" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
          </svg>
        </div>
      {:else}
        {#each paragraphs as toks, pi (pi)}
          <p class:heading={isHeading(toks)}>
            {#each toks as t (t.id)}<span data-t={t.id} class:cue={t.is_cue} class:md={isMdMark(t)}>{t.text}</span>{' '}{/each}
          </p>
        {/each}
        <div class="end" class:show={ended}>— Ende —</div>
      {/if}
    </div>
    {#if prefs.dimRead}<div class="dim" bind:this={dim} aria-hidden="true"></div>{/if}
    <!-- svelte-ignore a11y_no_static_element_interactions a11y_no_noninteractive_tabindex -->
    <div
      class="marker {prefs.markerStyle}"
      class:hold
      class:draggable={!!onfocusdrag}
      role={onfocusdrag ? 'slider' : undefined}
      tabindex={onfocusdrag ? 0 : undefined}
      aria-label={onfocusdrag ? 'Lesemarker-Position' : undefined}
      aria-valuemin={onfocusdrag ? Math.round(FOCUS_LINE_MIN * 100) : undefined}
      aria-valuemax={onfocusdrag ? Math.round(FOCUS_LINE_MAX * 100) : undefined}
      aria-valuenow={onfocusdrag ? Math.round(app.settings.focusLine * 100) : undefined}
      aria-valuetext={onfocusdrag ? `${Math.round(app.settings.focusLine * 100)} Prozent` : undefined}
      onpointerdown={markerDown}
      onpointermove={markerMove}
      onpointerup={markerUp}
      onkeydown={markerKey}
    >
      <span class="arrow" aria-hidden="true"></span>
    </div>
    {#if countdown > 0}
      {#key countdown}
        <div class="countdown" aria-live="assertive">{countdown}</div>
      {/key}
    {/if}
  </div>
  {#if stats}
    <div class="stats mono" aria-hidden="true">{stats.fps} fps · max {stats.worst} ms</div>
  {/if}
</div>

<style>
  .viewport {
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
    background: var(--pr-bg);
    color: var(--pr-text);
    contain: strict;
  }
  .stage {
    position: absolute;
    inset: 0;
  }
  .content {
    position: relative; /* offsetParent for token measurement */
    margin: 0 auto;
    width: min(var(--pr-measure), calc(100% - 48px * var(--pr-scale)));
    font-family: var(--pr-font);
    font-weight: var(--pr-weight);
    font-size: calc(var(--pr-size) * var(--pr-scale));
    line-height: var(--pr-line-height);
    letter-spacing: var(--pr-letter-spacing);
    word-spacing: var(--pr-word-spacing);
    will-change: transform;
    padding-bottom: 60vh; /* DESIGN §3.6: 60 % run-out */
    overflow-wrap: break-word;
    hyphens: manual;
  }
  p {
    margin: 0 0 0.6em;
  }
  p.heading {
    font-weight: 700;
  }
  span.cue {
    font-style: italic;
    opacity: 0.55;
  }
  span.md {
    display: none;
  }
  .interactive span[data-t] {
    cursor: pointer;
  }
  .interactive span[data-t]:hover {
    text-decoration: underline;
    text-decoration-thickness: 0.06em;
    text-underline-offset: 0.12em;
  }
  .hl {
    position: absolute;
    left: 0;
    top: 0;
    border-radius: 4px;
    background: var(--pr-word);
    opacity: 0;
    pointer-events: none;
    transition:
      transform var(--dur-fast) var(--ease-standard),
      width var(--dur-fast) var(--ease-standard),
      opacity var(--dur-fast);
  }
  .dim {
    position: absolute;
    left: 0;
    right: 0;
    top: 0;
    height: 0;
    background: var(--pr-bg);
    opacity: calc(1 - var(--pr-read-opacity));
    pointer-events: none;
  }
  .marker {
    position: absolute;
    left: 0;
    right: 0;
    top: calc(var(--focus) * 100%);
    pointer-events: none;
    display: flex;
    justify-content: center;
  }
  .marker::before {
    content: '';
    display: block;
  }
  .marker.line::before {
    height: 2px;
    background: var(--pr-marker);
    width: calc(min(100%, var(--pr-measure) + 24px));
    font-size: calc(var(--pr-size) * var(--pr-scale));
    margin-top: -1px;
  }
  .marker.band::before {
    height: calc(var(--pr-size) * var(--pr-scale) * var(--pr-line-height));
    transform: translateY(-50%);
    width: calc(min(100%, var(--pr-measure) + 24px));
    font-size: calc(var(--pr-size) * var(--pr-scale));
    background: color-mix(in srgb, var(--pr-marker) 12%, transparent);
    border-top: 1px solid color-mix(in srgb, var(--pr-marker) 40%, transparent);
    border-bottom: 1px solid color-mix(in srgb, var(--pr-marker) 40%, transparent);
  }
  .marker .arrow {
    position: absolute;
    left: 8px;
    top: -8px;
    border: 8px solid transparent;
    border-left: 12px solid var(--pr-marker);
  }
  .marker.hold::after {
    /* DESIGN §3.6: only visible state on talent = subtle ring at the marker */
    content: '';
    position: absolute;
    left: 6px;
    top: -14px;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    border: 3px solid color-mix(in srgb, var(--state-hold) 30%, transparent);
  }
  .marker.draggable {
    pointer-events: auto;
    cursor: ns-resize;
    height: 16px;
    margin-top: -8px;
    align-items: center;
  }
  .marker.draggable .arrow {
    top: 0;
  }
  .countdown {
    position: absolute;
    left: 0;
    right: 0;
    top: calc(var(--focus) * 100%);
    transform: translateY(-50%);
    text-align: center;
    font: 700 min(calc(var(--pr-size) * var(--pr-scale) * 2), 40vh) / 1 var(--font-ui);
    font-variant-numeric: tabular-nums;
    color: var(--pr-marker);
    pointer-events: none;
    animation: count var(--dur-countdown) var(--ease-standard) both;
  }
  @keyframes count {
    from { opacity: 0; transform: translateY(-50%) scale(0.9); }
    30% { opacity: 1; transform: translateY(-50%) scale(1); }
    to { opacity: 0.9; transform: translateY(-50%) scale(1); }
  }
  @media (prefers-reduced-motion: reduce) {
    @keyframes count {
      from { opacity: 0; }
      to { opacity: 1; }
    }
  }
  .end {
    color: var(--pr-direction);
    text-align: center;
    margin-top: 1em;
    font-size: 0.6em;
    opacity: 0.6;
  }
  .end.show {
    opacity: 1;
  }
  .empty {
    display: grid;
    place-items: center;
    height: 60vh;
    opacity: 0.1;
  }
  .stats {
    position: absolute;
    right: 8px;
    bottom: 8px;
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 11px;
    background: rgba(0, 0, 0, 0.6);
    color: #fff;
  }
</style>
