import { describe, expect, it } from 'vitest';
import { notches, tokenSteps, WheelAccumulator, wheelTarget, wpmDelta } from './wheel';

const mods = (m: Partial<{ ctrlKey: boolean; altKey: boolean; metaKey: boolean }> = {}) => ({ ctrlKey: false, altKey: false, ...m });

describe('wheelTarget', () => {
  it.each([
    ['manual', false, 'position'],
    ['constant', false, 'speed'],
    ['voiceActivated', false, 'speed'],
    ['voiceTracking', true, 'position'],
    ['voiceActivated', true, 'position'],
  ] as const)('%s (voice=%s) -> %s', (mode, voice, expected) => {
    expect(wheelTarget(mode, voice, mods())).toBe(expected);
  });

  it('Ctrl / Alt / Cmd swap position and speed outside voice mode', () => {
    expect(wheelTarget('manual', false, mods({ ctrlKey: true }))).toBe('speed');
    expect(wheelTarget('manual', false, mods({ altKey: true }))).toBe('speed');
    expect(wheelTarget('constant', false, mods({ ctrlKey: true }))).toBe('position');
    expect(wheelTarget('constant', false, mods({ metaKey: true }))).toBe('position');
  });

  it('voice mode always stays the manual override', () => {
    expect(wheelTarget('voiceTracking', true, mods({ ctrlKey: true }))).toBe('position');
  });
});

describe('notches / wpmDelta', () => {
  it('normalises pixel, line and page deltas', () => {
    expect(notches({ deltaX: 0, deltaY: 100, deltaMode: 0 })).toBe(1);
    expect(notches({ deltaX: 0, deltaY: -3, deltaMode: 1 })).toBe(-1);
    expect(notches({ deltaX: 0, deltaY: 1, deltaMode: 2 })).toBe(1);
  });

  it('Shift+wheel arrives as deltaX and still counts', () => {
    expect(notches({ deltaX: -100, deltaY: 0, deltaMode: 0 })).toBe(-1);
  });

  it('wheel up = faster: ±5 per notch, Shift ±1', () => {
    expect(wpmDelta(-1, false)).toBe(5);
    expect(wpmDelta(2, false)).toBe(-10);
    expect(wpmDelta(-1, true)).toBe(1);
    expect(wpmDelta(3, true)).toBe(-3);
  });

  it('position steps keep the legacy ~1 token per 40 px', () => {
    expect(tokenSteps({ deltaX: 0, deltaY: 100, deltaMode: 0 })).toBe(3);
    expect(tokenSteps({ deltaX: 0, deltaY: -10, deltaMode: 0 })).toBe(-1);
    expect(tokenSteps({ deltaX: 0, deltaY: 0, deltaMode: 0 })).toBe(0);
  });
});

describe('WheelAccumulator', () => {
  it('collects touchpad fractions into whole notches', () => {
    const a = new WheelAccumulator();
    expect(a.push(0.4, 0)).toBe(0);
    expect(a.push(0.4, 10)).toBe(0);
    expect(a.push(0.4, 20)).toBe(1);
    expect(a.push(2.2, 30)).toBe(2);
  });

  it('resets on direction change and after a pause', () => {
    const a = new WheelAccumulator(400);
    a.push(0.8, 0);
    expect(a.push(-0.5, 10)).toBe(0);
    a.push(0.9, 20);
    expect(a.push(0.5, 1000)).toBe(0);
  });

  it('a full mouse notch counts immediately', () => {
    const a = new WheelAccumulator();
    expect(a.push(-1, 0)).toBe(-1);
    expect(a.push(-1, 5)).toBe(-1);
  });
});
