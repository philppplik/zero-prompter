import { afterEach, describe, expect, it } from 'vitest';
import { cubicBezier, dur, EASE, parseBezier, setReducedMotion } from './motion';
import { tokens } from './tokens';

describe('cubicBezier', () => {
  it('hits the endpoints exactly', () => {
    const f = cubicBezier(0.2, 0, 0, 1);
    expect(f(0)).toBe(0);
    expect(f(1)).toBe(1);
    expect(f(-1)).toBe(0);
    expect(f(2)).toBe(1);
  });

  it('linear control points give identity', () => {
    const f = cubicBezier(0.25, 0.25, 0.75, 0.75);
    for (const x of [0.1, 0.33, 0.5, 0.9]) expect(f(x)).toBeCloseTo(x, 4);
  });

  it('matches CSS ease-out shape (decelerating, monotonic)', () => {
    const f = EASE.enter;
    let prev = 0;
    for (let x = 0.05; x <= 1; x += 0.05) {
      const y = f(x);
      expect(y).toBeGreaterThanOrEqual(prev - 1e-9);
      prev = y;
    }
    expect(f(0.5)).toBeGreaterThan(0.5);
    expect(EASE.exit(0.5)).toBeLessThan(0.5);
  });
});

describe('parseBezier', () => {
  it('parses token strings and falls back to linear', () => {
    expect(parseBezier('cubic-bezier(0.25, 0.25, 0.75, 0.75)')(0.4)).toBeCloseTo(0.4, 4);
    expect(parseBezier('ease')(0.4)).toBe(0.4);
  });

  it('every easing token parses', () => {
    for (const v of Object.values(tokens.motion.easing)) if (typeof v === 'string') expect(v).toMatch(/^cubic-bezier\(/);
  });
});

describe('dur', () => {
  afterEach(() => setReducedMotion(false));

  it('returns token durations and 0 under reduced motion', () => {
    expect(dur('base')).toBe(tokens.motion.duration.base);
    setReducedMotion(true);
    expect(dur('base')).toBe(0);
    expect(dur('panel')).toBe(0);
  });
});
