import { describe, expect, it } from 'vitest';
import {
  colorPresets,
  DEFAULT_MARKER,
  lineRects,
  MARKER_BOUNDS,
  MARKER_STYLES,
  markerColor,
  markerVars,
  migrateDisplaySettings,
  migrateLegacyStyle,
  nextStyle,
  normalizeHex,
  READ_OPACITY_MIN,
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
  it('resolves marker > theme', () => {
    expect(markerColor(DEFAULT_MARKER, theme)).toBe(theme.marker);
    expect(markerColor({ ...DEFAULT_MARKER, color: '#222222' }, theme)).toBe('#222222');
  });
  it('presets start with the theme accent and are unique', () => {
    const p = colorPresets(theme);
    expect(p[0]).toBe(normalizeHex(theme.marker));
    expect(new Set(p).size).toBe(p.length);
  });
});

describe('markerVars', () => {
  it('emits vars and respects reduced motion', () => {
    const v = parseVars(markerVars({ ...DEFAULT_MARKER, glow: 0.5 }, '#ff0000', false));
    expect(v['--pr-marker']).toBe('#ff0000');
    expect(v['--mk-glow']).toBe('8px');
    expect(v['--mk-dur']).toBe('180ms');
    expect(v['--pr-read-opacity']).toBeUndefined();
    expect(parseVars(markerVars(DEFAULT_MARKER, '#000000', true))['--mk-dur']).toBe('0ms');
  });
  it('never touches read opacity (dimming lives in the Darstellung prefs)', () => {
    const m = { ...DEFAULT_MARKER, dimRead: 0.6 };
    expect(parseVars(markerVars(m, '#000000', false))['--pr-read-opacity']).toBeUndefined();
  });
});

describe('migrateDisplaySettings', () => {
  const p = (o: Partial<{ markerColor: string | null; readOpacity: number | null }> = {}) => ({
    markerColor: null,
    readOpacity: null,
    ...o,
  });

  it('does nothing for clean settings', () => {
    expect(migrateDisplaySettings(p(), DEFAULT_MARKER)).toBeNull();
  });

  it('moves the old Darstellung marker colour into marker.color', () => {
    const r = migrateDisplaySettings(p({ markerColor: '#FF0000' }), DEFAULT_MARKER)!;
    expect(r.marker?.color).toBe('#ff0000');
    expect(r.prefs).toEqual({ markerColor: null });
  });

  it('keeps an existing marker colour and only clears the pref', () => {
    const r = migrateDisplaySettings(p({ markerColor: '#ff0000' }), { ...DEFAULT_MARKER, color: '#00ff00' })!;
    expect(r.marker).toBeNull();
    expect(r.prefs).toEqual({ markerColor: null });
  });

  it('moves marker.dimRead into readOpacity and clears it', () => {
    const r = migrateDisplaySettings(p(), { ...DEFAULT_MARKER, dimRead: 0.6 })!;
    expect(r.prefs?.readOpacity).toBeCloseTo(0.4);
    expect(r.marker?.dimRead).toBeNull();
  });

  it('keeps an existing readOpacity, clamps to the slider range', () => {
    const r = migrateDisplaySettings(p({ readOpacity: 0.8 }), { ...DEFAULT_MARKER, dimRead: 0.6 })!;
    expect(r.prefs).toBeNull();
    expect(r.marker?.dimRead).toBeNull();
    const c = migrateDisplaySettings(p(), { ...DEFAULT_MARKER, dimRead: 0.9 })!;
    expect(c.prefs?.readOpacity).toBeGreaterThanOrEqual(READ_OPACITY_MIN);
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
