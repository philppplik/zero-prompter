<!--
  Reading marker layer (DESIGN §3.7), shared by the talent window, the
  operator preview and the settings live preview. Lives inside the mirrored
  stage, so H/V mirroring applies automatically. Expects the prompter CSS
  vars (`--pr-*`, `--mk-*`, `--focus`) on an ancestor.
  Performance: the layer is positioned with transform only; per-frame work
  (`frame()`) touches the DOM only when the current line / range changes,
  except for underline bars, which live inside the scrolling content.
-->
<script lang="ts">
  import { onDestroy } from 'svelte';
  import { clampFocusLine, FOCUS_LINE_MAX, FOCUS_LINE_MIN } from '../lib/store.svelte';
  import { lineRects, sentenceRange, type MarkerSettings } from '../lib/marker';
  import type { LayoutMap } from '../lib/scroller';

  interface Props {
    marker: MarkerSettings;
    focus: number;
    hold?: boolean;
    reduced?: boolean;
    /** Element whose height maps to 0..1 for dragging (the viewport). */
    viewport?: () => HTMLElement | undefined;
    /** Host for underline bars inside the scrolling content layer. */
    ulHost?: HTMLElement;
    ondrag?: (focus: number) => void;
  }
  let { marker, focus, hold = false, reduced = false, viewport, ulHost, ondrag }: Props = $props();

  let shapes = $state<HTMLDivElement>();
  let dragging = $state(false);

  const style = $derived(marker.style);
  const blur = $derived(marker.blur > 0);

  // ---- per-frame hook (called by Prompter's scroller onFrame) ----
  let lastLine = -1;
  let lastKey = '';
  let lastLayout: LayoutMap | null = null;
  let bars: HTMLDivElement[] = [];

  function hideBars(from = 0) {
    for (let i = from; i < bars.length; i++) bars[i].style.opacity = '0';
  }

  function ensureBars(n: number) {
    if (!ulHost) return;
    while (bars.length < n) {
      const b = document.createElement('div');
      b.className = 'mk-ul-bar';
      b.setAttribute('aria-hidden', 'true');
      Object.assign(b.style, {
        position: 'absolute',
        left: '0',
        top: '0',
        width: '1px',
        height: 'calc(0.07em * var(--mk-size, 1))',
        background: 'var(--pr-marker)',
        transformOrigin: '0 0',
        pointerEvents: 'none',
        opacity: '0',
        filter: 'drop-shadow(0 0 var(--mk-glow, 0px) var(--pr-marker))',
        transition: 'transform var(--mk-dur, 0ms) var(--ease-standard, ease), opacity var(--dur-fast, 120ms)',
        willChange: 'transform',
      });
      ulHost.appendChild(b);
      bars.push(b);
    }
  }

  /** Current reading position (fractional token index) -> marker updates. */
  export function frame(pos: number, layout: LayoutMap | null, tokens: ReadonlyArray<{ text: string }>): void {
    if (!layout || layout.lineOf.length === 0) {
      hideBars();
      return;
    }
    let i = Math.max(0, Math.min(layout.lineOf.length - 1, Math.floor(pos)));
    while (i > 0 && layout.lineOf[i] < 0) i--;
    const li = layout.lineOf[i];
    if (li !== lastLine) {
      if (lastLine >= 0 && marker.animation === 'pulse' && !reduced && shapes) {
        shapes.animate([{ opacity: 0.25 }, { opacity: 1 }], { duration: 260, easing: 'ease-out' });
      }
      lastLine = li;
    }
    if (style !== 'underline') {
      if (lastKey) {
        hideBars();
        lastKey = '';
      }
      return;
    }
    const [s, e] = marker.underlineScope === 'sentence' ? sentenceRange(tokens, i) : [i, i + 1];
    const key = `${s}:${e}`;
    if (key === lastKey && layout === lastLayout) return;
    lastKey = key;
    lastLayout = layout;
    const rects = lineRects(layout.boxes, s, e);
    ensureBars(rects.length);
    rects.forEach((r, k) => {
      const b = bars[k];
      b.style.transform = `translate(${r.x}px, ${r.y + r.h * 0.92}px) scaleX(${r.w})`;
      b.style.opacity = 'var(--mk-opacity, 1)';
    });
    hideBars(rects.length);
  }

  // Style switch away from underline: hide bars immediately.
  $effect(() => {
    if (style !== 'underline') hideBars();
    lastKey = '';
  });

  onDestroy(() => {
    for (const b of bars) b.remove();
    bars = [];
  });

  // ---- drag (operator preview only): 5–70 % of the height ----
  function down(e: PointerEvent) {
    if (!ondrag) return;
    dragging = true;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }
  function move(e: PointerEvent) {
    const vp = viewport?.();
    if (!dragging || !ondrag || !vp) return;
    const r = vp.getBoundingClientRect();
    ondrag(clampFocusLine(Math.round(((e.clientY - r.top) / r.height) * 100) / 100));
  }
  function up() {
    dragging = false;
  }
  function key(e: KeyboardEvent) {
    if (!ondrag) return;
    const d = e.key === 'ArrowUp' ? -0.01 : e.key === 'ArrowDown' ? 0.01 : 0;
    if (d) {
      e.preventDefault();
      e.stopPropagation();
      ondrag(clampFocusLine(Math.round((focus + d) * 100) / 100));
    }
  }
</script>

<div class="rm" class:dragging class:blur>
  <div class="layer">
    {#if style === 'spotlight'}
      <i class="shade up" aria-hidden="true"></i><i class="shade down" aria-hidden="true"></i>
    {:else if style === 'gradient'}
      <i class="fade up" aria-hidden="true"></i><i class="fade down" aria-hidden="true"></i>
    {/if}
    {#if marker.dimUnread > 0 && style !== 'spotlight'}<i class="unread" aria-hidden="true"></i>{/if}
    <div class="shapes" bind:this={shapes} aria-hidden="true">
      {#if style === 'line'}
        <i class="bar"></i>
      {:else if style === 'band'}
        <i class="band"></i>
      {:else if style === 'arrows'}
        <i class="tri l"></i><i class="tri r"></i>
      {:else if style === 'brackets'}
        <i class="brk l"></i><i class="brk r"></i>
      {:else if style === 'sidebar'}
        <i class="side"></i>
      {:else if style === 'gradient'}
        <i class="gband"></i>
      {:else if style === 'spotlight'}
        <i class="edge"></i>
      {/if}
    </div>
    {#if hold}<i class="ring" aria-hidden="true"></i>{/if}
    {#if ondrag}
      <div
        class="handle"
        class:invisible={style === 'none' || style === 'underline'}
        role="slider"
        tabindex="0"
        aria-label="Lesemarker-Position"
        aria-orientation="vertical"
        aria-valuemin={Math.round(FOCUS_LINE_MIN * 100)}
        aria-valuemax={Math.round(FOCUS_LINE_MAX * 100)}
        aria-valuenow={Math.round(focus * 100)}
        aria-valuetext="{Math.round(focus * 100)} Prozent"
        onpointerdown={down}
        onpointermove={move}
        onpointerup={up}
        onpointercancel={up}
        onkeydown={key}
      ></div>
    {/if}
  </div>
</div>

<style>
  .rm {
    position: absolute;
    inset: 0;
    pointer-events: none;
    font-family: var(--pr-font);
    font-size: calc(var(--pr-size) * var(--pr-scale, 1));
    --zone: calc(1em * var(--pr-line-height) * var(--mk-size, 1));
    --col: calc(min(var(--pr-measure), 100% - 48px * var(--pr-scale, 1)) + 24px);
    --edge: calc(50% - var(--col) / 2);
  }
  /* Full-height layer whose top edge sits on the focus line (transform only). */
  .layer {
    position: absolute;
    left: 0;
    right: 0;
    top: 0;
    height: 100%;
    transform: translateY(calc(var(--focus) * 100%));
    transition: transform var(--mk-dur, 0ms) var(--ease-standard, ease);
    will-change: transform;
  }
  .dragging .layer {
    transition: none;
  }
  i {
    position: absolute;
    display: block;
  }
  .shapes {
    position: absolute;
    left: 0;
    right: 0;
    top: 0;
    height: 0;
    opacity: var(--mk-opacity, 1);
    filter: drop-shadow(0 0 var(--mk-glow, 0px) var(--pr-marker));
  }
  .bar {
    left: var(--edge);
    width: var(--col);
    top: 0;
    height: calc(2px * var(--mk-size, 1));
    transform: translateY(-50%);
    border-radius: 1px;
    background: var(--pr-marker);
  }
  .band {
    left: var(--edge);
    width: var(--col);
    top: 0;
    height: var(--zone);
    transform: translateY(-50%);
    background: color-mix(in srgb, var(--pr-marker) 12%, transparent);
    border-top: 1px solid color-mix(in srgb, var(--pr-marker) 40%, transparent);
    border-bottom: 1px solid color-mix(in srgb, var(--pr-marker) 40%, transparent);
  }
  .tri {
    top: 0;
    width: calc(0.3em * var(--mk-size, 1));
    height: calc(0.36em * var(--mk-size, 1));
    background: var(--pr-marker);
    transform: translateY(-50%);
  }
  .tri.l {
    left: 8px;
    clip-path: polygon(0 0, 100% 50%, 0 100%);
  }
  .tri.r {
    right: 8px;
    clip-path: polygon(100% 0, 0 50%, 100% 100%);
  }
  .brk {
    top: 0;
    width: 0.22em;
    height: var(--zone);
    transform: translateY(-50%);
    border: calc(3px * var(--mk-size, 1)) solid var(--pr-marker);
    box-sizing: border-box;
  }
  .brk.l {
    left: var(--edge);
    border-right: 0;
  }
  .brk.r {
    right: var(--edge);
    border-left: 0;
  }
  .side {
    left: calc(var(--edge) - 0.15em);
    top: 0;
    width: calc(4px * var(--mk-size, 1));
    height: var(--zone);
    transform: translateY(-50%);
    border-radius: 2px;
    background: var(--pr-marker);
  }
  .gband {
    left: 0;
    right: 0;
    top: 0;
    height: calc(var(--zone) * 1.6);
    transform: translateY(-50%);
    background: linear-gradient(
      to bottom,
      transparent,
      color-mix(in srgb, var(--pr-marker) 16%, transparent) 50%,
      transparent
    );
  }
  .edge {
    left: 0;
    right: 0;
    top: 0;
    height: var(--zone);
    transform: translateY(-50%);
    border-top: 1px solid color-mix(in srgb, var(--pr-marker) 50%, transparent);
    border-bottom: 1px solid color-mix(in srgb, var(--pr-marker) 50%, transparent);
    box-sizing: border-box;
  }
  .shade,
  .fade,
  .unread {
    left: 0;
    right: 0;
    height: 100%;
  }
  .shade.up,
  .fade.up {
    bottom: calc(100% + var(--zone) / 2);
  }
  .shade.down,
  .fade.down,
  .unread {
    top: calc(var(--zone) / 2);
  }
  .shade {
    background: color-mix(in srgb, var(--pr-bg) calc(var(--mk-opacity, 1) * 85%), transparent);
  }
  .fade.up {
    background: linear-gradient(to top, transparent, var(--pr-bg) calc(var(--zone) * 3));
    opacity: var(--mk-opacity, 1);
  }
  .fade.down {
    background: linear-gradient(to bottom, transparent, var(--pr-bg) calc(var(--zone) * 3));
    opacity: var(--mk-opacity, 1);
  }
  /* Soft focus without backdrop-filter: blurring the scrolling text behind the
     overlay is recomputed every frame and stutters on integrated / ARM GPUs.
     A stronger static veil (compositor only) gives a similar calm. */
  .blur .shade,
  .blur .fade {
    opacity: min(1, calc(var(--mk-opacity, 1) + var(--mk-blur-k, 0)));
  }
  .unread {
    background: var(--pr-bg);
    opacity: var(--mk-dim-unread, 0);
  }
  .ring {
    /* DESIGN §3.6: only visible state on talent = subtle ring at the marker */
    left: 6px;
    top: -14px;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    border: 3px solid color-mix(in srgb, var(--state-hold) 30%, transparent);
    box-sizing: border-box;
  }
  .handle {
    position: absolute;
    left: 0;
    right: 0;
    top: -8px;
    height: 16px;
    pointer-events: auto;
    cursor: ns-resize;
    touch-action: none;
  }
  .handle:focus-visible,
  .handle.invisible:hover {
    outline: 1px dashed color-mix(in srgb, var(--pr-marker) 60%, transparent);
    outline-offset: -4px;
  }
</style>
