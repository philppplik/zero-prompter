// Frame-rate independent scroll motion (token space). Pure, no DOM.
// (UI transition helpers live in lib/motion.ts.)
//
// The frontend owns the clock: every rAF frame integrates a velocity with the
// real frame delta (performance.now), so 60/90/120/144 Hz panels all move the
// same distance per second. Rust stays the authority for *where* the script
// is, but its ~60 Hz `scroll://frame` positions are only used for a gentle,
// capped correction – never for snapping (except real seeks, which glide with
// the critically damped spring).
//
// - clock modes (constant, voice-activated): velocity = WPM / 60 tokens/s,
//   eased on changes (wheel speed steps never jump).
// - voice tracking: discrete cursor updates (every ~0.2–0.4 s) become a
//   continuous speaking rate (VoiceRateEstimator); the scroll glides at that
//   rate and a capped proportional correction pulls it toward the predicted
//   speaker position. Forward-only (no backward jitter), smooth stop on hold.
// - manual / stopped / seeks: critically damped spring toward the target.

import type { ScrollMode, VoicePhase } from './types';

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

export interface MotionInput {
  /** performance.now() of this frame (ms). */
  now: number;
  mode: ScrollMode;
  phase: VoicePhase;
  playing: boolean;
  /** Authoritative position from Rust (tokens). */
  position: number;
  /** Voice cursor target (tokens). */
  target: number;
  /** performance.now() when `position`/`target` arrived (ms). */
  receivedAt: number;
  wpm: number;
  /** Voice tracking + lost policy "constant speed" (Rust scrolls at WPM). */
  lostConstant?: boolean;
}

/** Tunables (seconds, tokens, tokens/s). Exported for tests. */
export const MOTION = {
  /** Velocity easing time constant for speed changes (wheel, start/stop). */
  tauVelocity: 0.25,
  /** Voice: velocity smoothing (slightly faster to keep latency low). */
  tauVoice: 0.18,
  /** Proportional correction gain toward the authority (1/s). */
  gain: 1.6,
  /** Correction cap: absolute part + fraction of the nominal speed. */
  capAbs: 0.6,
  capRel: 0.4,
  /** Distance that counts as a seek (spring glide instead of correction). */
  seekForward: 8,
  seekBackward: 2.5,
  /** Leave the seek spring when closer than this. */
  seekDone: 0.35,
  /** Max authority extrapolation (s). */
  maxAhead: 0.25,
  /** Integration sub-step (s). */
  subStep: 1 / 240,
} as const;

/**
 * Speaking-rate estimate (tokens/s) from discrete cursor updates. Uses the
 * span of the recent updates (window), which is robust against the irregular
 * update cadence of the ASR. Decays smoothly to 0 when updates stop.
 */
export class VoiceRateEstimator {
  private hist: { t: number; c: number }[] = [];
  rate = 0;
  lastUpdate = -Infinity;
  /** Mean time between updates (s), for prediction. */
  gap = 0.3;

  constructor(
    private windowS = 2.5,
    private maxRate = 8,
  ) {}

  reset(): void {
    this.hist = [];
    this.rate = 0;
    this.lastUpdate = -Infinity;
  }

  /** Feed a cursor update at time `t` (s). Backward / huge jumps reset the window. */
  push(t: number, cursor: number): void {
    const last = this.hist[this.hist.length - 1];
    if (last && (cursor < last.c - 0.5 || cursor - last.c > MOTION.seekForward)) {
      this.hist = [];
      this.rate = 0;
    }
    this.hist.push({ t, c: cursor });
    while (this.hist.length > 2 && t - this.hist[0].t > this.windowS) this.hist.shift();
    const first = this.hist[0];
    const span = t - first.t;
    if (this.hist.length >= 2 && span > 0.05) {
      const r = Math.max(0, Math.min(this.maxRate, (cursor - first.c) / span));
      // Light low-pass so a single late update does not swing the rate.
      this.rate = this.rate === 0 ? r : this.rate + (r - this.rate) * 0.5;
      this.gap = Math.min(1, Math.max(0.05, span / (this.hist.length - 1)));
    }
    this.lastUpdate = t;
  }

  /** Effective rate at time `t` (s): full while updates flow, then decays to 0. */
  rateAt(t: number, holding: boolean): number {
    const age = t - this.lastUpdate;
    const grace = holding ? 0 : Math.max(0.5, this.gap * 2);
    if (age <= grace) return this.rate;
    return this.rate * Math.exp(-(age - grace) / 0.35);
  }
}

type Phase = 'follow' | 'seek' | 'glide';

export class ScrollMotion {
  x = 0;
  /** Smoothed base velocity (tokens/s). */
  v = 0;
  readonly voice = new VoiceRateEstimator();
  private spring: SpringState = { x: 0, v: 0 };
  private state: Phase = 'follow';
  private lastTarget = NaN;
  private lastMode: ScrollMode | null = null;

  snap(pos: number): void {
    this.x = pos;
    this.v = 0;
    this.spring = { x: pos, v: 0 };
    this.state = 'follow';
    this.voice.reset();
    this.lastTarget = NaN;
  }

  /** Nominal velocity the authority is moving at right now (tokens/s). */
  static clockVelocity(i: MotionInput): number {
    if (!i.playing) return 0;
    // Stale authority (no frames for a while): Rust has stopped moving.
    if (i.now - i.receivedAt > 400) return 0;
    const wps = Math.max(0, i.wpm) / 60;
    if (i.mode === 'constant') return wps;
    if (i.mode === 'voiceActivated') return i.phase === 'tracking' || i.phase === 'listening' ? wps : 0;
    if (i.mode === 'voiceTracking' && i.lostConstant && i.phase === 'lost') return wps;
    return 0;
  }

  /** Advances by `dt` seconds. Returns the new position (tokens). */
  step(i: MotionInput, dt: number): number {
    if (!(dt > 0)) return this.x;
    if (i.mode !== this.lastMode) {
      this.lastMode = i.mode;
      this.voice.reset();
      this.lastTarget = NaN;
    }
    const tS = i.now / 1000;
    const voiceMode = i.playing && i.mode === 'voiceTracking' && !(i.lostConstant && i.phase === 'lost');
    if (voiceMode && i.target !== this.lastTarget) {
      this.voice.push(tS, i.target);
      this.lastTarget = i.target;
    }

    let auth: number;
    let nominal: number;
    let tau = MOTION.tauVelocity;
    if (!i.playing || i.mode === 'manual') {
      return this.follow(i.mode === 'voiceTracking' && i.playing ? i.target : i.position, dt);
    } else if (voiceMode) {
      const holding = i.phase === 'hold' || i.phase === 'lost' || i.phase === 'idle';
      nominal = this.voice.rateAt(tS, holding);
      const age = Math.max(0, tS - this.voice.lastUpdate);
      // Predicted speaker position: cursor + rate * time since the update.
      auth = i.target + nominal * Math.min(age, this.voice.gap * 1.5);
      tau = MOTION.tauVoice;
    } else {
      nominal = ScrollMotion.clockVelocity(i);
      const ahead = Math.min(MOTION.maxAhead, Math.max(0, (i.now - i.receivedAt) / 1000));
      auth = i.position + nominal * ahead;
    }

    const err = auth - this.x;
    if (this.state === 'seek' || err > MOTION.seekForward || err < -MOTION.seekBackward) {
      return this.seek(auth, nominal, dt);
    }
    this.state = 'glide';
    const cap = MOTION.capAbs + MOTION.capRel * nominal;
    const steps = Math.max(1, Math.ceil(dt / MOTION.subStep));
    const h = dt / steps;
    const a = 1 - Math.exp(-h / tau);
    for (let k = 0; k < steps; k++) {
      this.v += (nominal - this.v) * a;
      const e = auth + nominal * (k * h) - this.x;
      const corr = Math.max(-cap, Math.min(cap, e * MOTION.gain));
      // Forward only: corrections may slow down or stop, never reverse.
      this.x += Math.max(0, this.v + corr) * h;
    }
    this.spring = { x: this.x, v: this.v };
    return this.x;
  }

  private follow(target: number, dt: number): number {
    this.state = 'follow';
    this.spring = stepSpring({ x: this.x, v: this.spring.v }, target, dt);
    this.x = this.spring.x;
    this.v = 0;
    return this.x;
  }

  private seek(auth: number, nominal: number, dt: number): number {
    this.state = 'seek';
    this.spring = stepSpring({ x: this.x, v: this.spring.v }, auth, dt);
    this.x = this.spring.x;
    if (Math.abs(auth - this.x) < MOTION.seekDone) {
      this.state = 'glide';
      this.v = Math.max(0, Math.min(nominal * 1.5, this.spring.v));
    }
    return this.x;
  }
}

/**
 * Critically damped follower in pixel space (render stage). Smooths the slope
 * changes of the token -> pixel mapping at line boundaries (lines with fewer
 * words would otherwise scroll visibly faster/slower) without overshoot.
 */
export class PixelSmoother {
  y = 0;
  private v = 0;
  constructor(private omega = 14) {}
  snap(y: number): void {
    this.y = y;
    this.v = 0;
  }
  step(target: number, dt: number): number {
    if (!(dt > 0)) return this.y;
    const steps = Math.max(1, Math.ceil(dt / MOTION.subStep));
    const h = dt / steps;
    const w = this.omega;
    for (let k = 0; k < steps; k++) {
      const a = w * w * (target - this.y) - 2 * w * this.v;
      this.v += a * h;
      this.y += this.v * h;
    }
    return this.y;
  }
}
