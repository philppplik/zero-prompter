// Mouse wheel over the prompter (preview and talent window).
//
// - Voice on (voiceActivated / voiceTracking with voice enabled): wheel moves
//   the position = short manual override (Rust pauses voice for ~2 s).
// - Constant: wheel changes the speed (±5 WPM per notch, Shift ±1);
//   Ctrl/Alt + wheel moves the position instead.
// - Manual: wheel moves the position (like scrolling a document);
//   Ctrl/Alt + wheel changes the speed (prepares the constant mode).
import type { ScrollMode } from './types';

export const WPM_STEP = 5;
export const WPM_STEP_FINE = 1;
/** Pixel delta of one wheel notch (Chromium/WebView2 and WebKit report ~100). */
export const PX_PER_NOTCH = 100;

export interface WheelInput {
  deltaX: number;
  deltaY: number;
  deltaMode: number; // 0 = pixel, 1 = line, 2 = page
  shiftKey: boolean;
  ctrlKey: boolean;
  altKey: boolean;
  metaKey?: boolean;
}

export type WheelTarget = 'position' | 'speed';

/** What the wheel controls in this mode (modifier Ctrl/Alt/Cmd swaps position <-> speed). */
export function wheelTarget(mode: ScrollMode, voiceOn: boolean, e: Pick<WheelInput, 'ctrlKey' | 'altKey' | 'metaKey'>): WheelTarget {
  if (voiceOn && (mode === 'voiceTracking' || mode === 'voiceActivated')) return 'position';
  const base: WheelTarget = mode === 'manual' ? 'position' : 'speed';
  const swap = e.ctrlKey || e.altKey || !!e.metaKey;
  return swap ? (base === 'position' ? 'speed' : 'position') : base;
}

/**
 * Signed wheel delta in notches (down = positive). Shift+wheel is turned into
 * a horizontal scroll by Chromium/WebKit, so deltaX is used when deltaY is 0.
 */
export function notches(e: Pick<WheelInput, 'deltaX' | 'deltaY' | 'deltaMode'>): number {
  const d = e.deltaY !== 0 ? e.deltaY : e.deltaX;
  if (e.deltaMode === 1) return d / 3;
  if (e.deltaMode === 2) return d;
  return d / PX_PER_NOTCH;
}

/**
 * Accumulates fractional notches (touchpads send many tiny deltas) and
 * returns whole steps once a full notch is reached.
 */
export class WheelAccumulator {
  private acc = 0;
  private last = 0;

  constructor(private readonly resetMs = 400) {}

  /** Whole notches to apply now (may be 0). */
  push(n: number, now: number): number {
    if (now - this.last > this.resetMs || Math.sign(n) !== Math.sign(this.acc)) this.acc = 0;
    this.last = now;
    this.acc += n;
    const whole = Math.trunc(this.acc) || 0; // no -0
    this.acc -= whole;
    return whole;
  }

  reset(): void {
    this.acc = 0;
  }
}

/** WPM delta for whole notches: wheel up (negative) = faster. */
export function wpmDelta(steps: number, fine: boolean): number {
  return -steps * (fine ? WPM_STEP_FINE : WPM_STEP);
}

/** Position step in tokens for a pixel-mode wheel event (legacy behaviour: ~1 token per 40 px). */
export function tokenSteps(e: Pick<WheelInput, 'deltaX' | 'deltaY' | 'deltaMode'>): number {
  const n = notches(e);
  if (n === 0) return 0;
  return Math.sign(n) * Math.max(1, Math.round(Math.abs(n) * 2.5));
}
