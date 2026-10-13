import { describe, expect, it } from 'vitest';
import { MOTION, PixelSmoother, ScrollMotion, VoiceRateEstimator, type MotionInput } from './scrollMotion';
import { FrameMeter } from './frameMeter';

const base = (o: Partial<MotionInput> = {}): MotionInput => ({
  now: 0,
  mode: 'constant',
  phase: 'idle',
  playing: true,
  position: 0,
  target: 0,
  receivedAt: 0,
  wpm: 120,
  ...o,
});

/**
 * Simulates `seconds` of rendering at `hz` while a fake Rust core advances the
 * authority at `wpm` and emits frames every ~16 ms (with jittered delivery).
 */
function runClock(hz: number, seconds: number, wpm = 120, jitterMs = 0) {
  const m = new ScrollMotion();
  m.snap(0);
  const dt = 1 / hz;
  let auth = { position: 0, receivedAt: 0 };
  let nextTick = 16;
  let k = 0;
  const xs: number[] = [];
  for (let t = dt; t <= seconds + 1e-9; t += dt) {
    const now = t * 1000;
    while (nextTick <= now) {
      // Delivery jitter: the frame arrives a bit later than it was computed.
      const late = jitterMs ? ((k++ * 7919) % 11) / 10 * jitterMs : 0;
      auth = { position: (wpm / 60) * (nextTick / 1000), receivedAt: nextTick + late };
      nextTick += 16;
    }
    xs.push(m.step(base({ now, wpm, ...auth }), dt));
  }
  return { m, xs };
}

describe('ScrollMotion – clock modes', () => {
  it('is frame-rate independent: same distance after 1 s at 30/60/120/144 Hz', () => {
    const d = [30, 60, 120, 144].map((hz) => runClock(hz, 1).m.x);
    for (const x of d) expect(Math.abs(x - d[1])).toBeLessThan(0.03);
  });

  it('converges to the nominal speed (wpm / 60) and stays with the authority', () => {
    const { m } = runClock(60, 3, 120);
    expect(m.v).toBeCloseTo(2, 2);
    expect(Math.abs(m.x - 6)).toBeLessThan(0.15);
  });

  it('never moves backwards, even with jittered frame delivery', () => {
    const { xs } = runClock(120, 3, 150, 8);
    for (let i = 1; i < xs.length; i++) expect(xs[i]).toBeGreaterThanOrEqual(xs[i - 1]);
  });

  it('per-frame steps are smooth (low variance) at steady state', () => {
    const { xs } = runClock(60, 3, 140, 8);
    const steps = xs.slice(90).map((x, i, a) => (i ? x - a[i - 1] : NaN)).slice(1);
    const mean = steps.reduce((a, b) => a + b, 0) / steps.length;
    const sd = Math.sqrt(steps.reduce((a, b) => a + (b - mean) ** 2, 0) / steps.length);
    expect(sd / mean).toBeLessThan(0.05);
  });

  it('eases speed changes (wheel) instead of jumping', () => {
    const m = new ScrollMotion();
    m.snap(0);
    const dt = 1 / 60;
    let now = 0;
    for (let i = 0; i < 120; i++) {
      now += dt * 1000;
      m.step(base({ now, wpm: 120, position: (now / 1000) * 2, receivedAt: now }), dt);
    }
    const v0 = m.v;
    now += dt * 1000;
    m.step(base({ now, wpm: 240, position: m.x, receivedAt: now }), dt);
    // One frame after doubling the WPM the speed changed only a little.
    expect(m.v - v0).toBeLessThan((4 - 2) * 0.1);
    expect(m.v).toBeGreaterThan(v0);
  });

  it('stops when the authority goes stale (no frames for > 400 ms)', () => {
    expect(ScrollMotion.clockVelocity(base({ now: 1000, receivedAt: 0 }))).toBe(0);
    expect(ScrollMotion.clockVelocity(base({ now: 100, receivedAt: 0 }))).toBe(2);
    expect(ScrollMotion.clockVelocity(base({ mode: 'voiceActivated', phase: 'hold' }))).toBe(0);
    expect(ScrollMotion.clockVelocity(base({ mode: 'voiceActivated', phase: 'tracking' }))).toBe(2);
  });

  it('bounded correction: a small offset is corrected at most at the cap', () => {
    const m = new ScrollMotion();
    m.snap(0);
    const dt = 1 / 60;
    // Authority 3 tokens ahead (below the seek threshold), not moving.
    const x1 = m.step(base({ now: 16, wpm: 0, position: 3, receivedAt: 16 }), dt);
    expect(x1).toBeLessThanOrEqual(MOTION.capAbs * dt + 1e-9);
  });

  it('glides (spring) to far seeks without overshoot', () => {
    const m = new ScrollMotion();
    m.snap(0);
    let max = 0;
    for (let i = 1; i <= 120; i++) {
      max = Math.max(max, m.step(base({ now: i * 16, playing: false, position: 50, receivedAt: 0 }), 1 / 60));
    }
    expect(m.x).toBeCloseTo(50, 1);
    expect(max).toBeLessThan(50.5);
  });
});

/** Voice: cursor updates every `gap` s, `wps` words/s, rendered at `hz`. */
function runVoice(hz: number, seconds: number, wps = 2.5, gap = 0.3, holdAfter = Infinity) {
  const m = new ScrollMotion();
  m.snap(0);
  const dt = 1 / hz;
  let target = 0;
  let next = gap;
  let receivedAt = 0;
  const xs: number[] = [];
  for (let t = dt; t <= seconds + 1e-9; t += dt) {
    if (t >= next && t < holdAfter) {
      target = Math.floor(next * wps);
      receivedAt = t * 1000;
      next += gap;
    }
    const phase = t >= holdAfter + 0.3 ? 'hold' : 'tracking';
    xs.push(m.step(base({ now: t * 1000, mode: 'voiceTracking', phase, target, position: target, receivedAt }), dt));
  }
  return { m, xs, target };
}

describe('ScrollMotion – voice tracking', () => {
  it('never moves backwards under forward cursor updates', () => {
    const { xs } = runVoice(120, 6);
    for (let i = 1; i < xs.length; i++) expect(xs[i]).toBeGreaterThanOrEqual(xs[i - 1] - 1e-12);
  });

  it('no stop-and-go: keeps moving between cursor updates once the rate is known', () => {
    const { xs } = runVoice(60, 6, 2.5, 0.3);
    const steps = xs.slice(120).map((x, i, a) => (i ? x - a[i - 1] : 1)).slice(1);
    const stalled = steps.filter((s) => s < 1e-4).length;
    expect(stalled / steps.length).toBeLessThan(0.05);
  });

  it('follows the speaker within a token', () => {
    const { m, target } = runVoice(60, 6, 2.5, 0.3);
    expect(Math.abs(m.x - target)).toBeLessThan(1.5);
  });

  it('is frame-rate independent (30 vs 60 vs 120 Hz)', () => {
    const r = [30, 60, 120].map((hz) => runVoice(hz, 4).m.x);
    for (const x of r) expect(Math.abs(x - r[1])).toBeLessThan(0.25);
  });

  it('holds smoothly: decelerates to a stop without jumping back', () => {
    const { xs, target } = runVoice(60, 6, 2.5, 0.3, 3);
    const tail = xs.slice(-60);
    // Stopped at the end of the run …
    expect(tail[tail.length - 1] - tail[0]).toBeLessThan(0.05);
    // … close to the last cursor (the speaker really was ~1 word further), never far past it.
    expect(tail[tail.length - 1]).toBeLessThan(target + 2);
    // Deceleration is gradual: no single frame moves more than at full speed.
    const holdIdx = Math.round(3 * 60);
    for (let i = holdIdx; i < xs.length; i++) expect(xs[i] - xs[i - 1]).toBeLessThan((2.5 * 2) / 60);
  });
});

describe('VoiceRateEstimator', () => {
  it('estimates words/s from irregular updates', () => {
    const e = new VoiceRateEstimator();
    const times = [0, 0.25, 0.6, 0.85, 1.2, 1.45, 1.8, 2.1];
    times.forEach((t) => e.push(t, t * 3));
    expect(e.rateAt(2.1, false)).toBeCloseTo(3, 1);
  });

  it('decays to zero on silence and resets on backward jumps', () => {
    const e = new VoiceRateEstimator();
    for (let i = 0; i <= 10; i++) e.push(i * 0.3, i);
    expect(e.rateAt(3.0, false)).toBeGreaterThan(3);
    expect(e.rateAt(6.0, false)).toBeLessThan(0.05);
    e.push(3.3, 2);
    expect(e.rate).toBe(0);
  });
});

describe('PixelSmoother', () => {
  it('approaches a step without (visible) overshoot', () => {
    const p = new PixelSmoother();
    p.snap(0);
    let y = 0;
    for (let i = 1; i <= 120; i++) y = p.step(100, 1 / 60);
    expect(y).toBeCloseTo(100, 1);
    expect(y).toBeLessThanOrEqual(100.05);
  });
});

describe('FrameMeter', () => {
  it('reports percentiles, drops and zero jitter for a perfect scroll', () => {
    const f = new FrameMeter(120);
    for (let i = 0; i < 100; i++) f.push(16.67, 2);
    f.push(50, 6);
    const s = f.stats()!;
    expect(s.p50).toBeCloseTo(16.67, 2);
    expect(s.max).toBe(50);
    expect(s.dropped).toBe(1);
    expect(s.droppedTotal).toBe(1);
    expect(s.hz).toBeCloseTo(60, 0);
    expect(s.jitterPx!).toBeLessThan(0.1);
  });

  it('measures uneven per-frame movement as jitter', () => {
    const f = new FrameMeter(120);
    for (let i = 0; i < 100; i++) f.push(16.67, i % 2 ? 1 : 3);
    expect(f.stats()!.jitterPx!).toBeCloseTo(1, 1);
  });

  it('jitter is null while not moving', () => {
    const f = new FrameMeter();
    for (let i = 0; i < 50; i++) f.push(16.67, 0);
    expect(f.stats()!.jitterPx).toBeNull();
  });
});
