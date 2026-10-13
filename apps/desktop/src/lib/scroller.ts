// Smooth-scroll engine for the talent renderer and the operator preview.
//
// Rust sends the scroll position in *token units* (~60 Hz, `scroll://frame`).
// The frontend owns layout: it measures token boxes, groups them into visual
// lines and maps a fractional token position to a continuous pixel offset.
// The rAF loop owns the clock (lib/scrollMotion.ts: frame-rate independent
// integration, Rust frames only correct gently) and renders with a single
// sub-pixel `translate3d` on one composited layer (DESIGN §2.8).

import { FrameMeter, type FrameStats } from './frameMeter';
import { PixelSmoother, ScrollMotion, type MotionInput } from './scrollMotion';

export { stepSpring, type SpringState } from './scrollMotion';

export interface Line {
  top: number;
  height: number;
  first: number; // first token id on the line
  count: number; // tokens on the line
}

export interface LayoutMap {
  lines: Line[];
  lineOf: Int32Array; // token id -> line index (-1 = not rendered)
  boxes: Float32Array; // token id -> [left, top, width, height]
}

/** Groups measured token boxes into lines (tokens whose tops differ < half a line). */
export function buildLayout(boxes: Float32Array, count: number): LayoutMap {
  const lines: Line[] = [];
  const lineOf = new Int32Array(count).fill(-1);
  let cur: Line | null = null;
  for (let i = 0; i < count; i++) {
    const top = boxes[i * 4 + 1];
    const h = boxes[i * 4 + 3];
    if (h <= 0) continue;
    if (!cur || Math.abs(top - cur.top) > cur.height / 2) {
      cur = { top, height: h, first: i, count: 0 };
      lines.push(cur);
    }
    cur.count = i - cur.first + 1;
    lineOf[i] = lines.length - 1;
  }
  return { lines, lineOf, boxes };
}

/** Continuous pixel y (relative to content top) for a fractional token position. */
export function yOfPosition(layout: LayoutMap, pos: number): number {
  const n = layout.lineOf.length;
  if (n === 0 || layout.lines.length === 0) return 0;
  const p = Math.max(0, Math.min(n - 1e-6, pos));
  let i = Math.floor(p);
  // Skip unrendered tokens (should not happen, but stay robust).
  while (i > 0 && layout.lineOf[i] < 0) i--;
  const li = layout.lineOf[i];
  if (li < 0) return 0;
  const line = layout.lines[li];
  const within = (i - line.first + (p - Math.floor(p))) / Math.max(1, line.count);
  return line.top + Math.min(1, within) * line.height;
}

export interface FrameSource {
  position: number;
  velocity: number;
  playing: boolean;
  receivedAt: number;
}

/** Where the scroll should be "now": last frame + velocity extrapolation (max 120 ms). */
export function predict(f: FrameSource, now: number): number {
  if (!f.playing) return f.position;
  const ahead = Math.min(0.12, Math.max(0, (now - f.receivedAt) / 1000));
  return f.position + f.velocity * ahead;
}

export interface ScrollerOptions {
  content: HTMLElement; // translated layer
  viewport: HTMLElement; // clipping box
  /** Per-frame motion inputs (authority frame + mode/WPM/voice phase), without `now`. */
  input: () => Omit<MotionInput, 'now'>;
  focusLine: () => number; // 0..1 of viewport height
  reducedMotion: () => boolean;
  onFrame?: (pos: number, layout: LayoutMap | null) => void;
  /** Once per second: fps + worst frame (ms) + full frame stats. */
  onStats?: (fps: number, worstMs: number, stats: FrameStats | null) => void;
  /** Optional shared meter (Diagnostics tab). */
  meter?: FrameMeter;
}

/**
 * rAF driver. Per frame: integrate motion (token space, frame-rate
 * independent) -> map to pixels via the cached layout -> critically damped
 * pixel smoothing -> one `translate3d` on the composited content layer.
 * No layout reads per frame (viewport height is cached on measure/resize),
 * sub-pixel offsets (no rounding), no allocations in the hot path except the
 * transform string.
 */
export class Scroller {
  layout: LayoutMap | null = null;
  /** Current content offset in px (positive = scrolled down). */
  offset = 0;
  readonly motion = new ScrollMotion();
  private px = new PixelSmoother();
  private pxInit = false;
  private viewportH = 0;
  private raf = 0;
  private last = 0;
  private frames = 0;
  private statT = 0;
  private worst = 0;
  private lastTransform = NaN;
  private meter: FrameMeter;

  constructor(private o: ScrollerOptions) {
    this.meter = o.meter ?? new FrameMeter();
  }

  /** Measures all `[data-t]` spans. Call after script/font/size/width changes. */
  measure(tokenCount: number): LayoutMap {
    const boxes = new Float32Array(tokenCount * 4);
    const spans = this.o.content.querySelectorAll<HTMLElement>('[data-t]');
    spans.forEach((el) => {
      const id = Number(el.dataset.t);
      if (id >= 0 && id < tokenCount) {
        boxes[id * 4] = el.offsetLeft;
        boxes[id * 4 + 1] = el.offsetTop;
        boxes[id * 4 + 2] = el.offsetWidth;
        boxes[id * 4 + 3] = el.offsetHeight;
      }
    });
    this.viewportH = this.o.viewport.clientHeight;
    this.layout = buildLayout(boxes, tokenCount);
    // Layout changed: re-anchor the pixel stage (no glide across a reflow).
    this.pxInit = false;
    return this.layout;
  }

  get viewportHeight(): number {
    return this.viewportH;
  }

  /** Viewport resized (ResizeObserver) – cached so the frame loop never reads layout. */
  setViewportHeight(h: number): void {
    this.viewportH = h;
  }

  /** Jump without animation (script load, seek while stopped). */
  snap(pos: number): void {
    this.motion.snap(pos);
    this.pxInit = false;
  }

  start(): void {
    const loop = (t: number) => {
      this.raf = requestAnimationFrame(loop);
      const dt = this.last ? Math.min(0.1, (t - this.last) / 1000) : 1 / 60;
      this.last = t;
      const inp = this.o.input();
      const now = performance.now();
      const reduced = this.o.reducedMotion();
      if (reduced) {
        // DESIGN §2.8: reduced motion -> follow the authority without animation.
        const p = inp.playing && inp.mode === 'voiceTracking' ? inp.target : inp.position;
        if (Math.abs(p - this.motion.x) > 3 || !inp.playing) this.motion.snap(p);
        else this.motion.step({ ...inp, now }, dt);
        this.pxInit = false;
      } else {
        const big = Math.abs(inp.position - this.motion.x) > 400;
        if (big) this.snap(inp.position);
        else this.motion.step({ ...inp, now }, dt);
      }
      const before = this.offset;
      this.render(dt);
      this.stats(t, dt, this.offset - before);
    };
    this.raf = requestAnimationFrame(loop);
  }

  stop(): void {
    cancelAnimationFrame(this.raf);
    this.raf = 0;
    this.last = 0;
  }

  get position(): number {
    return this.motion.x;
  }

  /** `dt` = 0: re-render at the current position without smoothing (after measure). */
  render(dt = 0): void {
    const l = this.layout;
    const y = l ? yOfPosition(l, this.motion.x) : 0;
    if (!this.pxInit || dt === 0 || Math.abs(y - this.px.y) > this.viewportH) {
      this.px.snap(y);
      this.pxInit = !!l;
    } else {
      this.px.step(y, dt);
    }
    const off = this.px.y - this.viewportH * this.o.focusLine();
    this.offset = off;
    // Sub-pixel transform; skip the style write when nothing moved.
    if (Math.abs(off - this.lastTransform) > 1e-3) {
      this.lastTransform = off;
      this.o.content.style.transform = `translate3d(0, ${(-off).toFixed(3)}px, 0)`;
    }
    this.o.onFrame?.(this.motion.x, l);
  }

  private stats(t: number, dt: number, dy: number): void {
    this.meter.push(dt * 1000, dy);
    if (!this.o.onStats) return;
    this.frames++;
    this.worst = Math.max(this.worst, dt * 1000);
    if (t - this.statT >= 1000) {
      this.o.onStats(
        Math.round((this.frames * 1000) / (t - this.statT || 1000)),
        Math.round(this.worst),
        this.meter.stats(),
      );
      this.frames = 0;
      this.worst = 0;
      this.statT = t;
    }
  }
}
