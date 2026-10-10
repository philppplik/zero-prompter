import { describe, expect, it } from 'vitest';
import { buildLayout, predict, stepSpring, yOfPosition, type SpringState } from './scroller';
import { boxes, layoutOf, range } from '../test/helpers';

describe('buildLayout', () => {
  it.each([
    { name: 'three lines', spec: [2, 3, 1], expected: [[0, 2], [2, 3], [5, 1]] },
    { name: 'single line', spec: [4], expected: [[0, 4]] },
    { name: 'one token per line', spec: [1, 1, 1], expected: [[0, 1], [1, 1], [2, 1]] },
  ])('groups tokens: $name', ({ spec, expected }) => {
    const l = layoutOf(spec);
    expect(l.lines.map((x) => [x.first, x.count])).toEqual(expected);
  });

  it('treats tops within half a line height as the same line (sub-pixel jitter)', () => {
    const l = buildLayout(boxes([[0, 100], [3, 100], [49, 100], [100, 100]]), 4);
    expect(l.lines).toHaveLength(2);
    expect(Array.from(l.lineOf)).toEqual([0, 0, 0, 1]);
  });

  it('marks zero-height tokens as unrendered', () => {
    const l = layoutOf([3], 100, [1]);
    expect(Array.from(l.lineOf)).toEqual([0, -1, 0]);
  });

  it('handles an empty script', () => {
    const l = buildLayout(new Float32Array(0), 0);
    expect(l.lines).toEqual([]);
    expect(yOfPosition(l, 3)).toBe(0);
  });
});

describe('yOfPosition', () => {
  const l = layoutOf([2, 3, 1]);

  it.each([
    [0, 0],
    [1, 50],
    [2, 100],
    [3, 100 + 100 / 3],
    [5, 200],
    [5.5, 250],
  ])('pos %d -> y %d', (pos, y) => {
    expect(yOfPosition(l, pos)).toBeCloseTo(y, 6);
  });

  it('clamps out-of-range positions', () => {
    expect(yOfPosition(l, -10)).toBe(0);
    expect(yOfPosition(l, 1e9)).toBeLessThanOrEqual(300);
    expect(yOfPosition(l, 1e9)).toBeGreaterThan(250);
  });

  it('is monotonic over the whole script', () => {
    const ys = range(0, 6, 0.01).map((p) => yOfPosition(l, p));
    for (let i = 1; i < ys.length; i++) expect(ys[i]).toBeGreaterThanOrEqual(ys[i - 1] - 1e-6);
  });

  it('is continuous across line boundaries (no visible jump)', () => {
    for (const boundary of [2, 5]) {
      const before = yOfPosition(l, boundary - 1e-4);
      const at = yOfPosition(l, boundary);
      expect(Math.abs(at - before)).toBeLessThan(0.1);
    }
  });

  it('stays finite and non-negative with hidden tokens', () => {
    const h = layoutOf([3], 100, [1]);
    for (const p of range(0, 3, 0.25)) {
      const y = yOfPosition(h, p);
      expect(Number.isFinite(y)).toBe(true);
      expect(y).toBeGreaterThanOrEqual(0);
    }
  });
});

describe('stepSpring', () => {
  const run = (s: SpringState, target: number, frames: number, dt = 1 / 60) => {
    let max = s.x;
    for (let i = 0; i < frames; i++) {
      s = stepSpring(s, target, dt);
      max = Math.max(max, s.x);
    }
    return { s, max };
  };

  it('converges to the target with negligible overshoot (~critically damped)', () => {
    const { s, max } = run({ x: 0, v: 0 }, 10, 120);
    expect(s.x).toBeCloseTo(10, 2);
    expect(max).toBeLessThan(10.5);
  });

  it('is a fixed point at rest on the target', () => {
    expect(stepSpring({ x: 5, v: 0 }, 5, 1 / 60)).toEqual({ x: 5, v: 0 });
  });

  it('dt = 0 changes nothing', () => {
    expect(stepSpring({ x: 1, v: 2 }, 9, 0)).toEqual({ x: 1, v: 2 });
  });

  it('is frame-rate independent (30 vs 144 Hz end up at the same place)', () => {
    const a = run({ x: 0, v: 0 }, 10, 30, 1 / 30).s.x;
    const b = run({ x: 0, v: 0 }, 10, 144, 1 / 144).s.x;
    expect(Math.abs(a - b)).toBeLessThan(0.05);
  });

  it('stays stable for a long frame hitch (sub-stepping)', () => {
    const s = stepSpring({ x: 0, v: 0 }, 100, 0.1);
    expect(Number.isFinite(s.x)).toBe(true);
    expect(s.x).toBeGreaterThan(0);
    expect(s.x).toBeLessThan(100);
  });
});

describe('predict', () => {
  const frame = (o: Partial<Parameters<typeof predict>[0]> = {}) => ({
    position: 10,
    velocity: 2,
    playing: true,
    receivedAt: 0,
    ...o,
  });

  it.each([
    { now: 0, expected: 10 },
    { now: 50, expected: 10.1 },
    { now: 120, expected: 10.24 },
    { now: 1000, expected: 10.24 }, // capped at 120 ms
    { now: -500, expected: 10 }, // clock skew: never extrapolate backwards
  ])('playing, now=$now -> $expected', ({ now, expected }) => {
    expect(predict(frame(), now)).toBeCloseTo(expected, 6);
  });

  it('does not extrapolate while paused', () => {
    expect(predict(frame({ playing: false }), 1000)).toBe(10);
  });
});
