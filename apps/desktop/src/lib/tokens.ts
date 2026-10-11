// Design tokens: docs/design/tokens.json is the single source of truth for
// Rust (embedded in zt-core) and the UI (imported here, applied as CSS vars).
import raw from '../../../../docs/design/tokens.json';
import { effectiveWeight, fontStack, FONT_IDS, FONTS } from './fonts';
import type { FontChoice, Typography, UiPrefs, VoicePhase } from './types';

export const tokens = raw;

export interface PrompterTheme {
  bg: string;
  text: string;
  readOpacity: number;
  marker: string;
  wordHighlight: string;
  direction: string;
  contrast: number;
  wcag: string;
  letterSpacing?: string;
  wordSpacing?: string;
  lineHeight?: number;
}

export const THEMES = raw.prompter as Record<string, PrompterTheme>;
export const THEME_IDS = Object.keys(THEMES);

export const THEME_LABELS: Record<string, string> = {
  studioDark: 'Studio Dark',
  broadcastClassic: 'Broadcast Classic',
  daylight: 'Daylight',
  highContrast: 'High Contrast',
  lowLight: 'Low Light',
  dyslexia: 'Dyslexia',
};

export const STATE_STYLE = raw.state as Record<VoicePhase, { color: string; shape: string; label: string }>;

/** Static font stacks per choice (`custom` without a name = system stack). */
export const FONT_STACKS = Object.fromEntries(FONT_IDS.map((id) => [id, FONTS[id].stack])) as Record<FontChoice, string>;
export const FONT_LABELS = Object.fromEntries(FONT_IDS.map((id) => [id, FONTS[id].label])) as Record<FontChoice, string>;

/** Typography slider ranges; must match zt_core::Settings::*_RANGE* (sanitized()). */
export const TYPO_RANGES = {
  fontWeight: [100, 900, 10],
  letterSpacingEm: [-0.05, 0.3, 0.01],
  wordSpacingEm: [-0.1, 1, 0.02],
  paragraphSpacingEm: [0, 3, 0.1],
  lineHeight: [1, 3, 0.05],
} as const;
const kebab = (s: string) => s.replace(/[A-Z]/g, (m) => '-' + m.toLowerCase());

/** UI chrome tokens -> `--ui-*` on :root for light & dark (DESIGN §2). */
export function applyUiTokens(root: HTMLElement = document.documentElement): void {
  const css: string[] = [];
  const block = (sel: string, vars: Record<string, string>) =>
    css.push(`${sel}{${Object.entries(vars).map(([k, v]) => `--ui-${kebab(k)}:${v}`).join(';')}}`);
  block(':root', raw.ui.light);
  block(':root[data-ui="dark"]', raw.ui.dark);
  css.push(`@media (prefers-color-scheme: dark){:root:not([data-ui="light"]){${Object.entries(raw.ui.dark).map(([k, v]) => `--ui-${kebab(k)}:${v}`).join(';')}}}`);
  const vars: string[] = [];
  for (const [k, v] of Object.entries(raw.space)) vars.push(`--space-${k}:${v}px`);
  for (const [k, v] of Object.entries(raw.radius)) vars.push(`--radius-${k}:${v}px`);
  for (const [k, v] of Object.entries(raw.elevation)) vars.push(`--elev-${k}:${v}`);
  for (const [k, v] of Object.entries(raw.motion.duration)) vars.push(`--dur-${kebab(k)}:${v}ms`);
  for (const [k, v] of Object.entries(raw.motion.easing)) if (typeof v === 'string') vars.push(`--ease-${kebab(k)}:${v}`);
  for (const [k, v] of Object.entries(raw.state)) vars.push(`--state-${k}:${v.color}`);
  for (const [k, v] of Object.entries(raw.layout)) if (typeof v === 'number') vars.push(`--layout-${kebab(k)}:${v < 1 ? v : v + 'px'}`);
  vars.push(`--font-ui:${raw.type.fontUi}`, `--font-mono:${raw.type.fontMono}`);
  vars.push(`--focus-ring-width:${raw.focus.ringWidth}px`, `--focus-ring-offset:${raw.focus.ringOffset}px`);
  css.push(`:root{${vars.join(';')}}`);
  let el = document.getElementById('zt-tokens') as HTMLStyleElement | null;
  if (!el) {
    el = document.createElement('style');
    el.id = 'zt-tokens';
    document.head.prepend(el);
  }
  el.textContent = css.join('\n');
  void root;
}

export function setUiTheme(mode: UiPrefs['uiTheme']): void {
  const r = document.documentElement;
  if (mode === 'system') r.removeAttribute('data-ui');
  else r.setAttribute('data-ui', mode);
}

const em = (v: number) => `${Math.round(v * 1000) / 1000}em`;

/**
 * Resolved prompter style = theme preset + core typography + user overrides.
 * Spacing: a non-zero setting wins, 0 keeps the theme value (dyslexia preset).
 * Line height: the theme value acts as a minimum.
 */
export function prompterVars(
  themeId: string,
  prefs: UiPrefs,
  fontSizePx: number,
  lineHeight: number,
  typo: Partial<Typography> = {},
): string {
  const t = THEMES[themeId] ?? THEMES.studioDark;
  const lh = Math.max(t.lineHeight ?? 0, lineHeight);
  const weight = effectiveWeight(prefs.fontFamily, typo.fontWeight ?? raw.type.prompter.weight);
  const vars: Record<string, string> = {
    '--pr-bg': prefs.bgColor ?? t.bg,
    '--pr-text': prefs.textColor ?? t.text,
    '--pr-marker': prefs.markerColor ?? t.marker,
    '--pr-word': t.wordHighlight,
    '--pr-direction': t.direction,
    '--pr-read-opacity': String(prefs.dimRead ? (prefs.readOpacity ?? t.readOpacity) : 1),
    '--pr-letter-spacing': typo.letterSpacingEm ? em(typo.letterSpacingEm) : (t.letterSpacing ?? 'normal'),
    '--pr-word-spacing': typo.wordSpacingEm ? em(typo.wordSpacingEm) : (t.wordSpacing ?? 'normal'),
    '--pr-line-height': String(lh),
    '--pr-size': `${fontSizePx}px`,
    '--pr-measure': `${prefs.measureCh}ch`,
    '--pr-font': fontStack(prefs.fontFamily, prefs.customFont),
    '--pr-weight': String(weight),
    '--pr-heading-weight': String(Math.min(900, Math.max(700, weight + 200))),
    '--pr-align': typo.textAlign ?? 'left',
    '--pr-transform': typo.textTransform ?? 'none',
    '--pr-para': em(typo.paragraphSpacingEm ?? 0.6),
  };
  return Object.entries(vars).map(([k, v]) => `${k}:${v}`).join(';');
}

/** Everything that changes line wrapping (Prompter re-measures when this key changes). */
export function layoutKey(s: Partial<Typography> & { fontSizePx: number; lineHeight: number; theme: string }, prefs: UiPrefs): string {
  return [
    s.theme, s.fontSizePx, s.lineHeight, s.fontWeight, s.letterSpacingEm, s.wordSpacingEm,
    s.paragraphSpacingEm, s.textAlign, s.textTransform, prefs.fontFamily, prefs.customFont, prefs.measureCh,
  ].join('|');
}
/** WCAG 2.x relative-luminance contrast ratio for #RRGGBB colours. */
export function contrastRatio(a: string, b: string): number {
  const lum = (hex: string) => {
    const m = /^#?([0-9a-f]{6})$/i.exec(hex.trim());
    if (!m) return 0;
    const n = parseInt(m[1], 16);
    const ch = [(n >> 16) & 255, (n >> 8) & 255, n & 255].map((c) => {
      const s = c / 255;
      return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
    });
    return 0.2126 * ch[0] + 0.7152 * ch[1] + 0.0722 * ch[2];
  };
  const [l1, l2] = [lum(a), lum(b)].sort((x, y) => y - x);
  return (l1 + 0.05) / (l2 + 0.05);
}
