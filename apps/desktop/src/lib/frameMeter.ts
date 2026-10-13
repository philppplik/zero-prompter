// Frame-timing metric (Diagnostics tab + frame-stats overlay): rAF frame time
// p50/p95/max, dropped frames and scroll position jitter. Pure, no DOM.
//
// Jitter = standard deviation (px) of the per-frame scroll delta against the
// expected delta (mean velocity of the window x frame time). A perfectly
// smooth constant scroll scores 0 regardless of the refresh rate.

export interface FrameStats {
  frames: number;
  p50: number;
  p95: number;
  max: number;
  /** Frames longer than 1.5x the median frame time (window). */
  dropped: number;
  /** Total dropped frames since reset. */
  droppedTotal: number;
  /** Std of per-frame px delta vs expected (px), null while not moving. */
  jitterPx: number | null;
  /** Estimated refresh rate (Hz) from the median frame time. */
  hz: number;
}

export class FrameMeter {
  private dt: Float64Array;
  private dy: Float64Array;
  private n = 0;
  private i = 0;
  droppedTotal = 0;

  constructor(private size = 240) {
    this.dt = new Float64Array(size);
    this.dy = new Float64Array(size);
  }

  reset(): void {
    this.n = 0;
    this.i = 0;
    this.droppedTotal = 0;
  }

  /** One rendered frame: frame time (ms) and scroll movement (px). */
  push(dtMs: number, dyPx: number): void {
    if (!(dtMs > 0) || !Number.isFinite(dtMs)) return;
    // Median of the current window as reference for the drop check.
    if (this.n >= 30) {
      const med = this.median();
      if (dtMs > med * 1.5) this.droppedTotal++;
    }
    this.dt[this.i] = dtMs;
    this.dy[this.i] = dyPx;
    this.i = (this.i + 1) % this.size;
    this.n = Math.min(this.size, this.n + 1);
  }

  private sorted(): number[] {
    return Array.from(this.dt.subarray(0, this.n)).sort((a, b) => a - b);
  }

  private median(): number {
    const s = this.sorted();
    return s[Math.floor((s.length - 1) * 0.5)];
  }

  stats(): FrameStats | null {
    if (this.n === 0) return null;
    const s = this.sorted();
    const q = (p: number) => s[Math.min(s.length - 1, Math.round((s.length - 1) * p))];
    const p50 = q(0.5);
    let dropped = 0;
    let sumDt = 0;
    let sumDy = 0;
    for (let k = 0; k < this.n; k++) {
      if (this.dt[k] > p50 * 1.5) dropped++;
      sumDt += this.dt[k];
      sumDy += this.dy[k];
    }
    let jitter: number | null = null;
    const vel = sumDy / sumDt; // px per ms
    if (Math.abs(sumDy) > 1 && this.n >= 10) {
      let acc = 0;
      for (let k = 0; k < this.n; k++) {
        const r = this.dy[k] - vel * this.dt[k];
        acc += r * r;
      }
      jitter = Math.sqrt(acc / this.n);
    }
    return {
      frames: this.n,
      p50,
      p95: q(0.95),
      max: s[s.length - 1],
      dropped,
      droppedTotal: this.droppedTotal,
      jitterPx: jitter,
      hz: p50 > 0 ? 1000 / p50 : 0,
    };
  }
}

/** Meter of the scroller in this window (operator preview in the control window). */
export const frameMeter = new FrameMeter();
