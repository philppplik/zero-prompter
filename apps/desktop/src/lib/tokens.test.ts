import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import {
  applyUiTokens,
  contrastRatio,
  FONT_LABELS,
  FONT_STACKS,
  prompterVars,
  setUiTheme,
  STATE_STYLE,
  THEME_IDS,
  THEME_LABELS,
  THEMES,
  tokens,
} from './tokens';
import { installFakeDocument, parseVars, prefs, type FakeDocument } from '../test/helpers';

describe('theme presets (tokens.json)', () => {
  it('has exactly the six documented presets, each with a label', () => {
    expect([...THEME_IDS].sort()).toEqual(
      ['broadcastClassic', 'daylight', 'dyslexia', 'highContrast', 'lowLight', 'studioDark'].sort(),
    );
    expect(Object.keys(THEME_LABELS).sort()).toEqual([...THEME_IDS].sort());
  });

  it.each(Object.keys(THEMES))('%s: AAA contrast and stored value matches computed', (id) => {
    const t = THEMES[id];
    const c = contrastRatio(t.text, t.bg);
    expect(c).toBeGreaterThanOrEqual(7);
    expect(Math.abs(c - t.contrast)).toBeLessThan(0.15);
    expect(t.wcag).toBe('AAA');
    expect(t.readOpacity).toBeGreaterThan(0);
    expect(t.readOpacity).toBeLessThan(1);
  });

  it('defines all six voice states with colour, shape and label (never colour only)', () => {
    for (const k of ['idle', 'listening', 'tracking', 'hold', 'lost', 'resync'] as const) {
      const s = STATE_STYLE[k];
      expect(s, k).toBeDefined();
      expect(s.color).toMatch(/^#|^rgb|^var\(/);
      expect(s.shape.length).toBeGreaterThan(0);
      expect(s.label.length).toBeGreaterThan(0);
    }
  });

  it('every font choice has a stack and a label', () => {
    expect(Object.keys(FONT_STACKS).sort()).toEqual(Object.keys(FONT_LABELS).sort());
  });
});

describe('contrastRatio', () => {
  it.each([
    ['#000000', '#FFFFFF', 21],
    ['#FFFFFF', '#000000', 21],
    ['#777777', '#777777', 1],
    ['000000', 'ffffff', 21], // no hash, lower case
  ])('%s vs %s = %d', (a, b, expected) => {
    expect(contrastRatio(a, b)).toBeCloseTo(expected, 2);
  });

  it('treats invalid colours as black instead of throwing', () => {
    expect(contrastRatio('not-a-colour', '#000000')).toBeCloseTo(1, 6);
  });
});

describe('prompterVars (theme loader + user overrides)', () => {
  const vars = (themeId: string, p = prefs(), size = 72, lh = 1.4) => parseVars(prompterVars(themeId, p, size, lh));

  it('uses the preset colours when no overrides are set', () => {
    const v = vars('daylight');
    expect(v['--pr-bg']).toBe(THEMES.daylight.bg);
    expect(v['--pr-text']).toBe(THEMES.daylight.text);
    expect(v['--pr-marker']).toBe(THEMES.daylight.marker);
    expect(v['--pr-size']).toBe('72px');
  });

  it('falls back to studioDark for unknown theme ids', () => {
    expect(vars('doesNotExist')['--pr-bg']).toBe(THEMES.studioDark.bg);
  });

  it('applies user colour overrides on top of the preset', () => {
    const v = vars('studioDark', prefs({ bgColor: '#123456', textColor: '#ABCDEF', markerColor: '#FF0000' }));
    expect(v).toMatchObject({ '--pr-bg': '#123456', '--pr-text': '#ABCDEF', '--pr-marker': '#FF0000' });
  });

  it.each([
    { dimRead: false, readOpacity: null, expected: '1' },
    { dimRead: true, readOpacity: null, expected: String(THEMES.studioDark.readOpacity) },
    { dimRead: true, readOpacity: 0.7, expected: '0.7' },
  ])('read opacity: dimRead=$dimRead override=$readOpacity -> $expected', ({ dimRead, readOpacity, expected }) => {
    expect(vars('studioDark', prefs({ dimRead, readOpacity }))['--pr-read-opacity']).toBe(expected);
  });

  it('theme line height wins over the setting; otherwise the setting is used', () => {
    expect(vars('dyslexia', prefs(), 72, 1.2)['--pr-line-height']).toBe('1.6');
    expect(vars('studioDark', prefs(), 72, 1.2)['--pr-line-height']).toBe('1.2');
  });

  it('dyslexia preset brings spacing, others default to normal', () => {
    expect(vars('dyslexia')['--pr-letter-spacing']).toBe('0.04em');
    expect(vars('studioDark')['--pr-letter-spacing']).toBe('normal');
  });

  it('font stack follows the font preference', () => {
    expect(vars('studioDark', prefs({ fontFamily: 'mono' }))['--pr-font']).toBe(FONT_STACKS.mono);
  });
});

describe('applyUiTokens / setUiTheme', () => {
  let doc: FakeDocument;
  let restore: () => void;
  beforeEach(() => {
    ({ doc, restore } = installFakeDocument());
  });
  afterEach(() => restore());

  it('injects exactly one <style id="zt-tokens"> and is idempotent', () => {
    applyUiTokens();
    applyUiTokens();
    expect(doc.styles).toHaveLength(1);
    expect(doc.styles[0].id).toBe('zt-tokens');
  });

  it('emits light, dark and media-query blocks plus state/space vars', () => {
    applyUiTokens();
    const css = doc.styles[0].textContent;
    expect(css).toContain(':root[data-ui="dark"]{');
    expect(css).toContain('@media (prefers-color-scheme: dark)');
    for (const k of Object.keys(tokens.state)) expect(css).toContain(`--state-${k}:`);
    for (const k of Object.keys(tokens.space)) expect(css).toContain(`--space-${k}:`);
    expect(css).not.toMatch(/undefined|NaN/);
  });

  it('maps the tweakcn "Minimal Neutral" UI theme (fonts, card radius, card gradient)', () => {
    applyUiTokens();
    const css = doc.styles[0].textContent;
    expect(css).toContain("--font-ui:'DM Sans Variable'");
    expect(css).toContain("--font-mono:'Geist Mono Variable'");
    expect(css).toContain('--radius-card:24px');
    expect(css).toContain('--radius-lg:16px'); // tweakcn --radius: 1rem
    expect(css).toContain('--ui-card-from:');
    expect(css).toContain('--dur-panel:');
    expect(css).toContain('--ease-emphasized:');
    expect(Object.keys(tokens.ui.light).sort()).toEqual(Object.keys(tokens.ui.dark).sort());
  });

  it.each([
    ['light', tokens.ui.light],
    ['dark', tokens.ui.dark],
  ] as const)('%s UI text meets AA on background, surface and accent', (_mode, ui) => {
    expect(contrastRatio(ui.text, ui.bg)).toBeGreaterThanOrEqual(7);
    expect(contrastRatio(ui.textMuted, ui.surface)).toBeGreaterThanOrEqual(4.5);
    expect(contrastRatio(ui.textOnAccent, ui.accent)).toBeGreaterThanOrEqual(4.5);
  });

  it.each([
    ['dark', 'dark'],
    ['light', 'light'],
    ['system', undefined],
  ] as const)('setUiTheme(%s) -> data-ui=%s', (mode, attr) => {
    setUiTheme('dark');
    setUiTheme(mode);
    expect(doc.attrs.get('data-ui')).toBe(attr);
  });
});
