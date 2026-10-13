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
  import { fade } from 'svelte/transition';
  import { Scroller, type LayoutMap } from '../lib/scroller';
  import { frameMeter, type FrameStats } from '../lib/frameMeter';
  import { frame, store } from '../lib/store.svelte';
  import { api } from '../lib/bridge';
  import { layoutKey, prompterVars, THEMES } from '../lib/tokens';
  import { notches, tokenSteps, WheelAccumulator, wheelTarget, wpmDelta } from '../lib/wheel';
  import { markerColor, markerVars, sanitizeMarker } from '../lib/marker';
  import ReadingMarker from './ReadingMarker.svelte';
  import type { AppState, Token, UiPrefs } from '../lib/types';

  interface Props {
    app: AppState;
    prefs: UiPrefs;
    /** Font scale relative to the talent output (preview = width / 1920). */
    scale?: number;
    mirror?: boolean;
    interactive?: boolean;
    /** Mouse wheel: position / speed (defaults to `interactive`). */
    wheel?: boolean;
    /** Only the talent window owns the layout report (CONCEPT §8.3). */
    report?: boolean;
    onword?: (token: number) => void;
    onfocusdrag?: (focus: number) => void;
  }
  let { app, prefs, scale = 1, mirror = false, interactive = false, wheel, report = false, onword, onfocusdrag }: Props = $props();

  let viewport: HTMLDivElement;
  let content: HTMLDivElement;
  let dim: HTMLDivElement;
  let hl: HTMLDivElement;
  let scroller: Scroller | null = null;
  let layoutRev = $state(0);
  let stats = $state<{ fps: number; worst: number; full: FrameStats | null } | null>(null);

  const reduced = () =>
    app.settings.reducedMotion || (typeof matchMedia !== 'undefined' && matchMedia('(prefers-reduced-motion: reduce)').matches);
  // Cached for the frame loop (refreshed on setting / media-query change).
  let reducedNow = false;
  $effect(() => {
    reducedNow = reduced();
  });

  const paragraphs = $derived(
    app.script.paragraphs.map((p) => app.script.tokens.slice(p.tokens.start, p.tokens.end)),
  );
  const isMdMark = (t: Token) => /^#{1,6}$/.test(t.text) || /^[-*>]$/.test(t.text);
  const isHeading = (toks: Token[]) => toks.length > 0 && /^#{1,6}$/.test(toks[0].text);

  const marker = $derived(sanitizeMarker(app.settings.marker));
  let rm = $state<ReturnType<typeof ReadingMarker>>();
  let ulHost = $state<HTMLDivElement>();
  const style = $derived(
    prompterVars(app.settings.theme, prefs, app.settings.fontSizePx, app.settings.lineHeight, app.settings) +
      ';' +
      markerVars(marker, markerColor(marker, THEMES[app.settings.theme] ?? THEMES.studioDark), reduced()) +
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
    scroller.render(0);
    if (report) {
      api.reportLayout(app.script.version, layout.lines.length).catch(() => {});
    }
  }

  // Re-measure whenever anything that affects wrapping changes.
  $effect(() => {
    // Typography (weight, spacing, alignment, case, font) changes line breaks:
    // the layout map / scroll mathematics must be rebuilt.
    void [app.script.version, layoutKey(app.settings, prefs), scale];
    remeasure();
  });

  // Snap (no animation) on script change.
  $effect(() => {
    void app.script.version;
    scroller?.snap(frame.position);
  });

  // Read-dimming overlay: full-height layer moved with transform only (no
  // per-frame height change -> no layout/paint, compositor only).
  let dimY = NaN;
  function placeDim(pos: number, layout: LayoutMap | null) {
    rm?.frame(pos, layout, app.script.tokens);
    if (!dim) return;
    const vh = scroller?.viewportHeight ?? 0;
    let edge = 0;
    if (layout && layout.lines.length > 0) {
      let i = Math.max(0, Math.min(layout.lineOf.length - 1, Math.floor(pos)));
      while (i > 0 && layout.lineOf[i] < 0) i--;
      const li = layout.lineOf[i];
      const line = li >= 0 ? layout.lines[li] : null;
      if (line) edge = Math.max(0, line.top - (scroller?.offset ?? 0));
    }
    const y = Math.min(edge, vh) - vh;
    if (Math.abs(y - dimY) < 1e-3) return;
    dimY = y;
    dim.style.transform = `translate3d(0, ${y.toFixed(3)}px, 0)`;
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

  // Speed HUD ("140 WPM"), shown briefly after a wheel speed change.
  let hud = $state(false);
  let hudTimer: ReturnType<typeof setTimeout> | undefined;
  const speedWheel = new WheelAccumulator();

  function onWheel(e: WheelEvent) {
    if (!(wheel ?? interactive)) return;
    e.preventDefault();
    const target = wheelTarget(app.scroll.mode, voiceOn, e);
    if (target === 'speed') {
      const steps = speedWheel.push(notches(e), e.timeStamp);
      if (steps === 0) return;
      store.send({ type: 'nudgeWpm', value: wpmDelta(steps, e.shiftKey) });
      hud = true;
      clearTimeout(hudTimer);
      hudTimer = setTimeout(() => (hud = false), 1200);
      return;
    }
    // Position. With voice on this is the manual override (Rust starts the 2 s voice hold).
    const step = tokenSteps(e);
    if (step === 0) return;
    const n = app.script.tokens.length;
    const to = Math.max(0, Math.min(n - 1, store.currentToken + step));
    store.send({ type: 'seekToken', value: to });
  }

  onMount(() => {
    scroller = new Scroller({
      content,
      viewport,
      // Plain reads (no reactivity in the hot path): authority frame + mode.
      input: () => ({
        mode: app.scroll.mode,
        phase: app.voice.phase,
        playing: frame.playing,
        position: frame.position,
        target: frame.target,
        receivedAt: frame.receivedAt,
        wpm: app.scroll.wpm,
        lostConstant: app.settings.lostPolicy === 'constantSpeed',
      }),
      focusLine: () => app.settings.focusLine,
      reducedMotion: () => reducedNow,
      onFrame: placeDim,
      meter: frameMeter,
      onStats: (fps, worst, full) => {
        reducedNow = reduced();
        if (prefs.showFrameStats) stats = { fps, worst, full };
        else if (stats) stats = null;
      },
    });
    scroller.snap(frame.position);
    remeasure();
    scroller.start();
    // prefers-reduced-motion is cached (matchMedia per frame is not free).
    const mq = typeof matchMedia !== 'undefined' ? matchMedia('(prefers-reduced-motion: reduce)') : null;
    const onMq = () => (reducedNow = reduced());
    mq?.addEventListener?.('change', onMq);
    // Non-passive so preventDefault works (Svelte may attach wheel listeners passively).
    const wheel = (e: WheelEvent) => onWheel(e);
    viewport.addEventListener('wheel', wheel, { passive: false });
    const ro = new ResizeObserver(() => remeasure());
    ro.observe(viewport);
    document.fonts?.ready.then(() => remeasure());
    // Fonts load lazily on first use (font switch, new weight): re-measure then.
    const fontsDone = () => remeasure();
    document.fonts?.addEventListener?.('loadingdone', fontsDone);
    return () => {
      mq?.removeEventListener?.('change', onMq);
      clearTimeout(hudTimer);
      document.fonts?.removeEventListener?.('loadingdone', fontsDone);
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
      <div class="mk-ul" bind:this={ulHost} aria-hidden="true"></div>
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
    <ReadingMarker
      bind:this={rm}
      {marker}
      focus={app.settings.focusLine}
      {hold}
      reduced={reduced()}
      viewport={() => viewport}
      {ulHost}
      ondrag={onfocusdrag}
    />
    {#if countdown > 0}
      {#key countdown}
        <div class="countdown" aria-live="assertive">{countdown}</div>
      {/key}
    {/if}
  </div>
  {#if hud}
    <div class="hud mono" role="status" aria-live="polite" transition:fade={{ duration: reduced() ? 0 : 160 }}>
      {app.scroll.wpm} WPM
    </div>
  {/if}
  {#if stats}
    <div class="stats mono" aria-hidden="true">
      {stats.fps} fps · max {stats.worst} ms{#if stats.full} · p95 {stats.full.p95.toFixed(1)} ms · drop {stats.full.dropped}{#if stats.full.jitterPx != null} · jitter {stats.full.jitterPx.toFixed(2)} px{/if}{/if}
    </div>
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
    text-align: var(--pr-align);
    text-transform: var(--pr-transform);
    will-change: transform;
    padding-bottom: 60vh; /* DESIGN §3.6: 60 % run-out */
    overflow-wrap: break-word;
    hyphens: manual;
  }
  p {
    margin: 0 0 var(--pr-para);
  }
  p.heading {
    font-weight: var(--pr-heading-weight);
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
    height: 100%;
    transform: translate3d(0, -100%, 0);
    will-change: transform;
    background: var(--pr-bg);
    opacity: calc(1 - var(--pr-read-opacity));
    pointer-events: none;
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
  .hud {
    position: absolute;
    left: 50%;
    bottom: 24px;
    transform: translateX(-50%);
    padding: 6px 14px;
    border-radius: 999px;
    font-size: 15px;
    font-weight: 600;
    background: rgba(0, 0, 0, 0.62);
    color: #fff;
    pointer-events: none;
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
