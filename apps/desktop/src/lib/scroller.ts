// Smooth-scroll engine for the talent renderer and the operator preview.
//
// Rust sends the scroll position in *token units* (~60 Hz, `scroll://frame`).
// The frontend owns layout: it measures token boxes, groups them into visual
// lines and maps a fractional token position to a continuous pixel offset.
// A rAF loop (time-delta based, not frame based) extrapolates between frames
// and follows the target with a spring (tokens.json motion.easing.spring),
// rendered with `transform: translate3d` on a single layer (DESIGN §2.8).

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

export interface SpringState {
  x: number;
  v: number;
}

const SPRING = { k: 170, d: 26 }; // tokens.json motion.easing.spring (mass 1)

/** Advances a spring toward `target` by dt seconds (sub-stepped, stable). */
export function stepSpring(s: SpringState, target: number, dt: number): SpringState {
  let { x, v } = s;
  const steps = Math.max(1, Math.ceil(dt / (1 / 240)));
  const h = dt / steps;
  for (let i = 0; i < steps; i++) {
    const a = SPRING.k * (target - x) - SPRING.d * v;
    v += a * h;
    x += v * h;
  }
  return { x, v };
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
  source: FrameSource;
  focusLine: () => number; // 0..1 of viewport height
  reducedMotion: () => boolean;
  onFrame?: (pos: number, layout: LayoutMap | null) => void;
  onStats?: (fps: number, worstMs: number) => void;
}

const isLinux = typeof navigator !== 'undefined' && /Linux/.test(navigator.userAgent) && !/Android/.test(navigator.userAgent);

export class Scroller {
  layout: LayoutMap | null = null;
  /** Current content offset in px (positive = scrolled down). */
  offset = 0;
  private spring: SpringState = { x: 0, v: 0 };
  private raf = 0;
  private last = 0;
  private frames = 0;
  private statT = 0;
  private worst = 0;

  constructor(private o: ScrollerOptions) {}

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
    this.layout = buildLayout(boxes, tokenCount);
    return this.layout;
  }

  /** Jump without animation (script load, seek while stopped). */
  snap(pos: number): void {
    this.spring = { x: pos, v: 0 };
  }

  start(): void {
    const loop = (t: number) => {
      this.raf = requestAnimationFrame(loop);
      const dt = this.last ? Math.min(0.1, (t - this.last) / 1000) : 1 / 60;
      this.last = t;
      this.stats(t, dt);
      const target = predict(this.o.source, performance.now());
      const dist = Math.abs(target - this.spring.x);
      if (dist > 400 || (this.o.reducedMotion() && dist > 3)) {
        this.snap(target); // DESIGN §2.8: reduced motion -> hard resync
      } else {
        this.spring = stepSpring(this.spring, target, dt);
      }
      this.render();
    };
    this.raf = requestAnimationFrame(loop);
  }

  stop(): void {
    cancelAnimationFrame(this.raf);
    this.raf = 0;
  }

  get position(): number {
    return this.spring.x;
  }

  render(): void {
    const l = this.layout;
    const y = l ? yOfPosition(l, this.spring.x) : 0;
    const off = y - this.o.viewport.clientHeight * this.o.focusLine();
    const px = isLinux ? Math.round(off) : off; // Linux: pixel snapping avoids shimmer
    this.offset = px;
    this.o.content.style.transform = `translate3d(0, ${-px}px, 0)`;
    this.o.onFrame?.(this.spring.x, l);
  }

  private stats(t: number, dt: number): void {
    if (!this.o.onStats) return;
    this.frames++;
    this.worst = Math.max(this.worst, dt * 1000);
    if (t - this.statT >= 1000) {
      this.o.onStats(Math.round((this.frames * 1000) / (t - this.statT || 1000)), Math.round(this.worst));
      this.frames = 0;
      this.worst = 0;
      this.statT = t;
    }
  }
}
