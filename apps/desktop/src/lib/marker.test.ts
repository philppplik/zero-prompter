import { describe, expect, it } from 'vitest';
import {
  colorPresets,
  DEFAULT_MARKER,
  lineRects,
  MARKER_BOUNDS,
  MARKER_STYLES,
  markerColor,
  markerVars,
  migrateLegacyStyle,
  nextStyle,
  normalizeHex,
  sanitizeMarker,
  sentenceRange,
} from './marker';
import { THEMES } from './tokens';
import { boxes, parseVars } from '../test/helpers';

const theme = THEMES.studioDark;

describe('sanitizeMarker', () => {
  it('fills defaults for missing/old settings', () => {
    expect(sanitizeMarker(undefined)).toEqual(DEFAULT_MARKER);
    expect(sanitizeMarker({ style: 'spotlight' }).style).toBe('spotlight');
  });
  it('clamps like the Rust core', () => {
    const m = sanitizeMarker({ opacity: 0, size: 99, dimUnread: 2, glow: -1, blur: Infinity, dimRead: NaN, color: 'red' });
    expect(m.opacity).toBe(MARKER_BOUNDS.opacity[0]);
    expect(m.size).toBe(MARKER_BOUNDS.size[1]);
    expect(m.dimUnread).toBe(MARKER_BOUNDS.dim[1]);
    expect(m.glow).toBe(0);
    expect(m.blur).toBe(DEFAULT_MARKER.blur);
    expect(m.dimRead).toBeNull();
    expect(m.color).toBeNull();
  });
  it('rejects unknown enums', () => {
    const m = sanitizeMarker({ style: 'laser' as never, animation: 'spin' as never, underlineScope: 'x' as never });
    expect([m.style, m.animation, m.underlineScope]).toEqual(['line', 'glide', 'word']);
  });
});

describe('colours', () => {
  it('normalizes hex', () => {
    expect(normalizeHex('#ABC')).toBe('#aabbcc');
    expect(normalizeHex(' #FF8800 ')).toBe('#ff8800');
    expect(normalizeHex('#12345')).toBeNull();
    expect(normalizeHex('123456')).toBeNull();
  });
  it('resolves marker > pref > theme', () => {
    expect(markerColor(DEFAULT_MARKER, null, theme)).toBe(theme.marker);
    expect(markerColor(DEFAULT_MARKER, '#111111', theme)).toBe('#111111');
    expect(markerColor({ ...DEFAULT_MARKER, color: '#222222' }, '#111111', theme)).toBe('#222222');
  });
  it('presets start with the theme accent and are unique', () => {
    const p = colorPresets(theme);
    expect(p[0]).toBe(normalizeHex(theme.marker));
    expect(new Set(p).size).toBe(p.length);
  });
});

describe('markerVars', () => {
  it('emits vars and respects reduced motion', () => {
    const v = parseVars(markerVars({ ...DEFAULT_MARKER, glow: 0.5 }, '#ff0000', true, false));
    expect(v['--pr-marker']).toBe('#ff0000');
    expect(v['--mk-glow']).toBe('8px');
    expect(v['--mk-dur']).toBe('180ms');
    expect(v['--pr-read-opacity']).toBeUndefined();
    expect(parseVars(markerVars(DEFAULT_MARKER, '#000000', true, true))['--mk-dur']).toBe('0ms');
  });
  it('overrides read opacity only when dimming is on', () => {
    const m = { ...DEFAULT_MARKER, dimRead: 0.6 };
    expect(Number(parseVars(markerVars(m, '#000000', true, false))['--pr-read-opacity'])).toBeCloseTo(0.4);
    expect(parseVars(markerVars(m, '#000000', false, false))['--pr-read-opacity']).toBe('1');
  });
});

describe('styles', () => {
  it('cycles through all styles', () => {
    let s = MARKER_STYLES[0].id;
    const seen = new Set<string>();
    for (let i = 0; i < MARKER_STYLES.length; i++) {
      seen.add(s);
      s = nextStyle(s);
    }
    expect(seen.size).toBe(9);
    expect(s).toBe(MARKER_STYLES[0].id);
  });
  it('migrates the legacy band pref once', () => {
    expect(migrateLegacyStyle('band', DEFAULT_MARKER)?.style).toBe('band');
    expect(migrateLegacyStyle('line', DEFAULT_MARKER)).toBeNull();
    expect(migrateLegacyStyle('band', { ...DEFAULT_MARKER, style: 'arrows' })).toBeNull();
  });
});

describe('sentenceRange', () => {
  const toks = 'Hallo Welt. Das ist ein Test! Ende'.split(' ').map((text) => ({ text }));
  it('finds sentence bounds', () => {
    expect(sentenceRange(toks, 0)).toEqual([0, 2]);
    expect(sentenceRange(toks, 3)).toEqual([2, 6]);
    expect(sentenceRange(toks, 6)).toEqual([6, 7]);
    expect(sentenceRange([], 0)).toEqual([0, 0]);
  });
});

describe('lineRects', () => {
  it('merges tokens per line and skips hidden ones', () => {
    const b = boxes([
      [0, 50],
      [0, 50],
      [60, 0],
      [60, 50],
    ]);
    b[4] = 20; // token 1 left
    const r = lineRects(b, 0, 4);
    expect(r).toHaveLength(2);
    expect(r[0]).toEqual({ x: 0, y: 0, w: 30, h: 50 });
    expect(r[1].y).toBe(60);
  });
});
