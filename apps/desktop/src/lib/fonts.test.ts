import { describe, expect, it } from 'vitest';
import { effectiveWeight, FONT_IDS, FONTS, fontStack, sanitizeFontName, weightHint } from './fonts';
import { layoutKey, prompterVars, TYPO_RANGES } from './tokens';
import { parseVars, prefs } from '../test/helpers';
// Raw sources (Vite ?raw) to cross-check imports and the Rust bounds.
import fontsSrc from '../fonts.ts?raw';
import settingsRs from '../../../../crates/zt-core/src/settings.rs?raw';

describe('font catalogue', () => {
  it('offers at least ten bundled fonts plus system and custom', () => {
    const bundled = FONT_IDS.filter((id) => FONTS[id].group !== 'System');
    expect(bundled.length).toBeGreaterThanOrEqual(10);
    expect(FONT_IDS).toContain('custom');
    expect(FONT_IDS).toContain('system');
  });

  it('keeps the old preference ids (settings files stay valid)', () => {
    for (const id of ['atkinson', 'inter', 'system', 'serif', 'mono']) expect(FONT_IDS).toContain(id);
  });

  it('every bundled family is imported by src/fonts.ts', () => {
    const src = fontsSrc;
    const pkg: Record<string, string> = {
      atkinson: 'atkinson-hyperlegible-next',
      inter: 'inter',
      lexend: 'lexend',
      sourceSans: 'source-sans-3',
      openSans: 'open-sans',
      robotoFlex: 'roboto-flex',
      plexSans: 'ibm-plex-sans',
      serif: 'source-serif-4',
      literata: 'literata',
      merriweather: 'merriweather',
      openDyslexic: 'opendyslexic',
      mono: 'jetbrains-mono',
    };
    for (const id of FONT_IDS.filter((i) => FONTS[i].group !== 'System')) {
      expect(pkg[id], id).toBeDefined();
      expect(src).toContain(`/${pkg[id]}`);
    }
  });

  it('every font has a generic fallback family', () => {
    for (const id of FONT_IDS) expect(FONTS[id].stack).toMatch(/(sans-serif|serif|monospace)$/);
  });
});

describe('effectiveWeight', () => {
  it('clamps to the variable axis', () => {
    expect(effectiveWeight('openSans', 100)).toBe(300);
    expect(effectiveWeight('openSans', 900)).toBe(800);
    expect(effectiveWeight('inter', 650)).toBe(650);
    expect(effectiveWeight('plexSans', 900)).toBe(700);
  });

  it('snaps static fonts to the nearest real weight', () => {
    expect(effectiveWeight('openDyslexic', 300)).toBe(400);
    expect(effectiveWeight('openDyslexic', 560)).toBe(700);
    expect(effectiveWeight('openDyslexic', 540)).toBe(400);
  });

  it('passes system / custom weights through, bounded to 100–900', () => {
    expect(effectiveWeight('system', 350)).toBe(350);
    expect(effectiveWeight('custom', 2000)).toBe(900);
    expect(effectiveWeight('custom', Number.NaN)).toBe(400);
  });

  it('has a hint for every font', () => {
    for (const id of FONT_IDS) expect(weightHint(id).length).toBeGreaterThan(0);
  });
});

describe('custom font', () => {
  it('strips characters that could break out of the CSS value', () => {
    expect(sanitizeFontName("Arial'; } body { x")).toBe('Arial body x');
    expect(sanitizeFontName('  Segoe   UI  ')).toBe('Segoe UI');
    expect(sanitizeFontName('Noto Sans Ä-1.0')).toBe('Noto Sans Ä-1.0');
  });

  it('puts the free name first, falls back to the system stack', () => {
    expect(fontStack('custom', 'Verdana')).toBe(`'Verdana', ${FONTS.custom.stack}`);
    expect(fontStack('custom', '')).toBe(FONTS.custom.stack);
    expect(fontStack('lexend', 'Verdana')).toBe(FONTS.lexend.stack);
  });
});

describe('typography variables', () => {
  const vars = (over = {}, p = prefs()) => parseVars(prompterVars('studioDark', p, 72, 1.4, over));

  it('defaults: left, no transform, 0.6em paragraphs, token weight', () => {
    const v = vars();
    expect(v['--pr-align']).toBe('left');
    expect(v['--pr-transform']).toBe('none');
    expect(v['--pr-para']).toBe('0.6em');
    expect(v['--pr-weight']).toBe('500');
  });

  it('maps settings to CSS variables', () => {
    const v = vars({ fontWeight: 300, letterSpacingEm: 0.05, wordSpacingEm: 0.2, paragraphSpacingEm: 1.5, textAlign: 'justify', textTransform: 'uppercase' });
    expect(v).toMatchObject({
      '--pr-weight': '300',
      '--pr-letter-spacing': '0.05em',
      '--pr-word-spacing': '0.2em',
      '--pr-para': '1.5em',
      '--pr-align': 'justify',
      '--pr-transform': 'uppercase',
      '--pr-heading-weight': '700',
    });
  });

  it('weight follows the font fallback (OpenDyslexic has 400/700 only)', () => {
    expect(vars({ fontWeight: 800 }, prefs({ fontFamily: 'openDyslexic' }))['--pr-weight']).toBe('700');
  });

  it('a zero spacing keeps the theme value', () => {
    const v = parseVars(prompterVars('dyslexia', prefs(), 72, 1.4, { letterSpacingEm: 0, wordSpacingEm: 0.3 }));
    expect(v['--pr-letter-spacing']).toBe('0.04em');
    expect(v['--pr-word-spacing']).toBe('0.3em');
  });

  it('theme line height is a minimum, larger settings win', () => {
    expect(parseVars(prompterVars('dyslexia', prefs(), 72, 2))['--pr-line-height']).toBe('2');
  });

  it('layoutKey changes with every wrapping-relevant setting', () => {
    const base = { theme: 'studioDark', fontSizePx: 72, lineHeight: 1.4, fontWeight: 500, letterSpacingEm: 0, wordSpacingEm: 0, paragraphSpacingEm: 0.6, textAlign: 'left' as const, textTransform: 'none' as const };
    const k = layoutKey(base, prefs());
    for (const patch of [{ fontWeight: 600 }, { letterSpacingEm: 0.1 }, { wordSpacingEm: 0.1 }, { paragraphSpacingEm: 1 }, { textAlign: 'center' as const }, { textTransform: 'uppercase' as const }]) {
      expect(layoutKey({ ...base, ...patch }, prefs()), JSON.stringify(patch)).not.toBe(k);
    }
    expect(layoutKey(base, prefs({ fontFamily: 'custom', customFont: 'Arial' }))).not.toBe(k);
  });

  it('slider ranges match the Rust sanitizer bounds', () => {
    const rs = settingsRs;
    expect(rs).toContain(`FONT_WEIGHT_MIN: u16 = ${TYPO_RANGES.fontWeight[0]};`);
    expect(rs).toContain(`FONT_WEIGHT_MAX: u16 = ${TYPO_RANGES.fontWeight[1]};`);
    expect(rs).toContain('LETTER_SPACING_RANGE_EM: (f32, f32) = (-0.05, 0.3);');
    expect(rs).toContain('WORD_SPACING_RANGE_EM: (f32, f32) = (-0.1, 1.0);');
    expect(rs).toContain('PARAGRAPH_SPACING_RANGE_EM: (f32, f32) = (0.0, 3.0);');
    expect(TYPO_RANGES.letterSpacingEm.slice(0, 2)).toEqual([-0.05, 0.3]);
    expect(TYPO_RANGES.wordSpacingEm.slice(0, 2)).toEqual([-0.1, 1]);
    expect(TYPO_RANGES.paragraphSpacingEm.slice(0, 2)).toEqual([0, 3]);
  });
});
