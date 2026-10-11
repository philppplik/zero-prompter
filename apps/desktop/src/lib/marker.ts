// Reading marker (DESIGN §3.7): types, bounds, sanitizing and pure geometry
// helpers. Bounds mirror `zt_core::marker::MarkerSettings` (Rust is the
// source of truth; `replaceSettings` sanitizes again in the core).
import type { PrompterTheme } from './tokens';

export type MarkerStyle =
  | 'line'
  | 'band'
  | 'arrows'
  | 'brackets'
  | 'spotlight'
  | 'gradient'
  | 'underline'
  | 'sidebar'
  | 'none';
export type MarkerAnimation = 'none' | 'glide' | 'pulse';
export type UnderlineScope = 'word' | 'sentence';

export interface MarkerSettings {
  style: MarkerStyle;
  /** `#rrggbb`; null = UI pref `markerColor` / theme marker. */
  color: string | null;
  opacity: number;
  /** Thickness / height multiplier. */
  size: number;
  /** Dimming strength for read text; null = theme `readOpacity`. */
  dimRead: number | null;
  dimUnread: number;
  glow: number;
  /** Soft-focus blur outside the reading zone (px, spotlight/gradient). */
  blur: number;
  animation: MarkerAnimation;
  underlineScope: UnderlineScope;
}

export const MARKER_BOUNDS = {
  opacity: [0.1, 1],
  size: [0.5, 3],
  dim: [0, 0.9],
  glow: [0, 1],
  blur: [0, 8],
} as const;

export const DEFAULT_MARKER: MarkerSettings = {
  style: 'line',
  color: null,
  opacity: 1,
  size: 1,
  dimRead: null,
  dimUnread: 0,
  glow: 0,
  blur: 0,
  animation: 'glide',
  underlineScope: 'word',
};

export const MARKER_STYLES: { id: MarkerStyle; label: string; hint: string }[] = [
  { id: 'line', label: 'Linie', hint: 'Dünne Leselinie über die Spaltenbreite' },
  { id: 'band', label: 'Band', hint: 'Halbtransparente Zeile mit Kanten' },
  { id: 'arrows', label: 'Pfeile', hint: 'Dreiecke links und rechts am Rand' },
  { id: 'brackets', label: 'Klammern', hint: 'Eckige Klammern um die Lesezeile' },
  { id: 'spotlight', label: 'Fokus-Fenster', hint: 'Alles außer der Lesezone abgedunkelt' },
  { id: 'gradient', label: 'Verlauf', hint: 'Weiches Band, Text oben und unten blendet aus' },
  { id: 'underline', label: 'Unterstreichung', hint: 'Unterstreicht das aktuelle Wort oder den Satz' },
  { id: 'sidebar', label: 'Seitenbalken', hint: 'Vertikaler Akzentbalken links der Zeile' },
  { id: 'none', label: 'Kein Marker', hint: 'Nur Position, nichts sichtbar' },
];

const STYLE_IDS = new Set<string>(MARKER_STYLES.map((s) => s.id));
const clamp = (v: unknown, [lo, hi]: readonly [number, number], def: number): number =>
  typeof v === 'number' && Number.isFinite(v) ? Math.min(hi, Math.max(lo, v)) : def;

/** `#RGB`/`#RRGGBB` -> lowercase `#rrggbb`, else null (same as Rust `normalize_hex`). */
export function normalizeHex(s: unknown): string | null {
  if (typeof s !== 'string') return null;
  const m = /^#([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(s.trim());
  if (!m) return null;
  const h = m[1].length === 3 ? [...m[1]].map((c) => c + c).join('') : m[1];
  return '#' + h.toLowerCase();
}

/** Complete and clamp (tolerates missing/old settings objects). */
export function sanitizeMarker(raw: Partial<MarkerSettings> | null | undefined): MarkerSettings {
  const m = raw ?? {};
  const d = DEFAULT_MARKER;
  return {
    style: STYLE_IDS.has(m.style as string) ? (m.style as MarkerStyle) : d.style,
    color: normalizeHex(m.color),
    opacity: clamp(m.opacity, MARKER_BOUNDS.opacity, d.opacity),
    size: clamp(m.size, MARKER_BOUNDS.size, d.size),
    dimRead: m.dimRead == null || !Number.isFinite(m.dimRead) ? null : clamp(m.dimRead, MARKER_BOUNDS.dim, 0),
    dimUnread: clamp(m.dimUnread, MARKER_BOUNDS.dim, d.dimUnread),
    glow: clamp(m.glow, MARKER_BOUNDS.glow, d.glow),
    blur: clamp(m.blur, MARKER_BOUNDS.blur, d.blur),
    animation: m.animation === 'none' || m.animation === 'pulse' ? m.animation : 'glide',
    underlineScope: m.underlineScope === 'sentence' ? 'sentence' : 'word',
  };
}

/** Effective marker colour: marker setting > UI pref > theme. */
export function markerColor(m: MarkerSettings, prefColor: string | null, theme: PrompterTheme): string {
  return m.color ?? prefColor ?? theme.marker;
}

/** Colour presets: theme accents first, then a few broadcast classics (deduplicated). */
export function colorPresets(theme: PrompterTheme): string[] {
  const all = [theme.marker, theme.direction, theme.wordHighlight, '#ff3b30', '#ffcc00', '#34c759', '#0a84ff', '#ffffff'];
  const out: string[] = [];
  for (const c of all) {
    const h = normalizeHex(c);
    if (h && !out.includes(h)) out.push(h);
  }
  return out;
}

/**
 * CSS custom properties for the marker layer. Appended after `prompterVars`,
 * so `--pr-marker` / `--pr-read-opacity` overrides win.
 */
export function markerVars(m: MarkerSettings, color: string, dimReadEnabled: boolean, reduced: boolean): string {
  const vars: Record<string, string> = {
    '--pr-marker': color,
    '--mk-opacity': String(m.opacity),
    '--mk-size': String(m.size),
    '--mk-glow': `${Math.round(m.glow * 16)}px`,
    '--mk-blur': `${m.blur}px`,
    '--mk-dim-unread': String(m.dimUnread),
    '--mk-dur': reduced || m.animation !== 'glide' ? '0ms' : '180ms',
  };
  if (m.dimRead != null) vars['--pr-read-opacity'] = String(dimReadEnabled ? 1 - m.dimRead : 1);
  return Object.entries(vars).map(([k, v]) => `${k}:${v}`).join(';');
}

/** Next style in the list (command palette cycling). */
export function nextStyle(s: MarkerStyle): MarkerStyle {
  const i = MARKER_STYLES.findIndex((x) => x.id === s);
  return MARKER_STYLES[(i + 1) % MARKER_STYLES.length].id;
}

const SENTENCE_END = /[.!?…]["'»«“”)\]]*$/;
const MAX_SENTENCE = 80;

/** Token range `[start, end)` of the sentence containing token `i`. */
export function sentenceRange(tokens: ReadonlyArray<{ text: string }>, i: number): [number, number] {
  const n = tokens.length;
  if (n === 0) return [0, 0];
  i = Math.max(0, Math.min(n - 1, i));
  let s = i;
  while (s > 0 && i - s < MAX_SENTENCE && !SENTENCE_END.test(tokens[s - 1].text)) s--;
  let e = i;
  while (e < n - 1 && e - i < MAX_SENTENCE && !SENTENCE_END.test(tokens[e].text)) e++;
  return [s, e + 1];
}

export interface Rect {
  x: number;
  y: number;
  w: number;
  h: number;
}

/**
 * One rect per visual line for tokens `[start, end)` from the scroller's
 * `boxes` (left, top, width, height per token). Hidden tokens (height 0) are skipped.
 */
export function lineRects(boxes: ArrayLike<number>, start: number, end: number, max = 8): Rect[] {
  const out: Rect[] = [];
  const count = Math.floor(boxes.length / 4);
  for (let t = Math.max(0, start); t < Math.min(end, count); t++) {
    const k = t * 4;
    const [x, y, w, h] = [boxes[k], boxes[k + 1], boxes[k + 2], boxes[k + 3]];
    if (!(h > 0)) continue;
    const last = out[out.length - 1];
    if (last && Math.abs(last.y - y) < h / 2) {
      const r = Math.max(last.x + last.w, x + w);
      last.x = Math.min(last.x, x);
      last.w = r - last.x;
    } else {
      if (out.length >= max) break;
      out.push({ x, y, w, h });
    }
  }
  return out;
}

/** One-time migration of the old UI pref (`markerStyle: 'band'`) into the core settings. */
export function migrateLegacyStyle(prefStyle: string | undefined, m: MarkerSettings): MarkerSettings | null {
  if (prefStyle === 'band' && m.style === 'line') return { ...m, style: 'band' };
  return null;
}
