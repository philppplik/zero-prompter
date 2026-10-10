// Motion helpers for Svelte transitions (DESIGN §2.6). Durations and curves
// come from tokens.json; everything collapses to 0 ms when the OS asks for
// reduced motion or the user enabled "Bewegung reduzieren" in the settings.
import { tokens } from './tokens';

export type DurationName = keyof typeof tokens.motion.duration;
export type Easing = (t: number) => number;

/** CSS-style cubic-bezier(x1, y1, x2, y2) as an easing function t∈[0,1] -> progress. */
export function cubicBezier(x1: number, y1: number, x2: number, y2: number): Easing {
  const cx = 3 * x1, bx = 3 * (x2 - x1) - cx, ax = 1 - cx - bx;
  const cy = 3 * y1, by = 3 * (y2 - y1) - cy, ay = 1 - cy - by;
  const sx = (t: number) => ((ax * t + bx) * t + cx) * t;
  const sy = (t: number) => ((ay * t + by) * t + cy) * t;
  const dx = (t: number) => (3 * ax * t + 2 * bx) * t + cx;
  return (x: number) => {
    if (x <= 0) return 0;
    if (x >= 1) return 1;
    let t = x;
    for (let i = 0; i < 8; i++) {
      const err = sx(t) - x;
      if (Math.abs(err) < 1e-6) return sy(t);
      const d = dx(t);
      if (Math.abs(d) < 1e-6) break;
      t -= err / d;
    }
    // Bisection fallback for flat derivatives.
    let lo = 0, hi = 1;
    t = x;
    for (let i = 0; i < 30; i++) {
      const v = sx(t);
      if (Math.abs(v - x) < 1e-6) break;
      if (v < x) lo = t;
      else hi = t;
      t = (lo + hi) / 2;
    }
    return sy(t);
  };
}

/** Parses "cubic-bezier(a, b, c, d)"; anything else falls back to linear. */
export function parseBezier(css: string): Easing {
  const m = /cubic-bezier\(\s*([-\d.]+)\s*,\s*([-\d.]+)\s*,\s*([-\d.]+)\s*,\s*([-\d.]+)\s*\)/.exec(css);
  if (!m) return (t) => t;
  return cubicBezier(Number(m[1]), Number(m[2]), Number(m[3]), Number(m[4]));
}

const e = tokens.motion.easing;
export const EASE = {
  standard: parseBezier(e.standard),
  enter: parseBezier(e.enter),
  exit: parseBezier(e.exit),
  emphasized: parseBezier(e.emphasized),
};

let userReduced = false;
/** Mirrors Settings.reducedMotion (set from App). */
export function setReducedMotion(on: boolean): void {
  userReduced = on;
}

export function reducedMotion(): boolean {
  if (userReduced) return true;
  return typeof matchMedia === 'function' && matchMedia('(prefers-reduced-motion: reduce)').matches;
}

/** Token duration in ms, 0 under reduced motion. */
export function dur(name: DurationName): number {
  return reducedMotion() ? 0 : tokens.motion.duration[name];
}
