// Prompter font catalogue. All bundled fonts come from @fontsource /
// @fontsource-variable (OFL-1.1, offline, see THIRD_PARTY_NOTICES.md) and are
// loaded by src/fonts.ts in both windows. Variable fonts bind `font-weight`
// to their `wght` axis; static fonts get snapped to the nearest real weight.

export type FontChoice =
  | 'atkinson'
  | 'inter'
  | 'lexend'
  | 'sourceSans'
  | 'openSans'
  | 'robotoFlex'
  | 'plexSans'
  | 'serif'
  | 'literata'
  | 'merriweather'
  | 'openDyslexic'
  | 'mono'
  | 'system'
  | 'custom';

export interface FontDef {
  label: string;
  /** CSS font-family stack (first entry = bundled family). */
  stack: string;
  group: 'Sans' | 'Serif' | 'Speziell' | 'System';
  /** Variable font: continuous `wght` range [min, max]. */
  range?: [number, number];
  /** Static font: available weights (ascending). */
  weights?: number[];
}

const SANS = 'system-ui, sans-serif';
const SERIF = "Georgia, 'Times New Roman', serif";

export const FONTS: Record<FontChoice, FontDef> = {
  atkinson: {
    label: 'Atkinson Hyperlegible Next',
    stack: `'Atkinson Hyperlegible Next Variable', 'Atkinson Hyperlegible Next', 'Inter Variable', ${SANS}`,
    group: 'Sans',
    range: [200, 800],
  },
  inter: { label: 'Inter', stack: `'Inter Variable', Inter, 'Segoe UI Variable', ${SANS}`, group: 'Sans', range: [100, 900] },
  lexend: { label: 'Lexend', stack: `'Lexend Variable', Lexend, ${SANS}`, group: 'Sans', range: [100, 900] },
  sourceSans: { label: 'Source Sans 3', stack: `'Source Sans 3 Variable', 'Source Sans 3', ${SANS}`, group: 'Sans', range: [200, 900] },
  openSans: { label: 'Open Sans', stack: `'Open Sans Variable', 'Open Sans', ${SANS}`, group: 'Sans', range: [300, 800] },
  robotoFlex: { label: 'Roboto Flex', stack: `'Roboto Flex Variable', 'Roboto Flex', Roboto, ${SANS}`, group: 'Sans', range: [100, 1000] },
  plexSans: { label: 'IBM Plex Sans', stack: `'IBM Plex Sans Variable', 'IBM Plex Sans', ${SANS}`, group: 'Sans', range: [100, 700] },
  serif: { label: 'Source Serif 4', stack: `'Source Serif 4 Variable', 'Source Serif 4', ${SERIF}`, group: 'Serif', range: [200, 900] },
  literata: { label: 'Literata', stack: `'Literata Variable', Literata, ${SERIF}`, group: 'Serif', range: [200, 900] },
  merriweather: { label: 'Merriweather', stack: `'Merriweather Variable', Merriweather, ${SERIF}`, group: 'Serif', range: [300, 900] },
  openDyslexic: { label: 'OpenDyslexic', stack: `OpenDyslexic, 'Comic Sans MS', ${SANS}`, group: 'Speziell', weights: [400, 700] },
  mono: { label: 'JetBrains Mono', stack: "'JetBrains Mono Variable', 'JetBrains Mono', ui-monospace, monospace", group: 'Speziell', range: [100, 800] },
  system: { label: 'Systemschrift (Standard)', stack: "system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif", group: 'System' },
  custom: { label: 'Systemschrift…', stack: "system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif", group: 'System' },
};

export const FONT_IDS = Object.keys(FONTS) as FontChoice[];

/** Font names may only contain letters, digits, spaces and a few separators. */
export function sanitizeFontName(name: string): string {
  return name
    .replace(/[^\p{L}\p{N} _.\-]/gu, '')
    .replace(/\s+/g, ' ')
    .trim()
    .slice(0, 64);
}

/** CSS font-family for a choice; `custom` puts the free font name first. */
export function fontStack(choice: FontChoice, customFont = ''): string {
  const def = FONTS[choice] ?? FONTS.atkinson;
  if (choice === 'custom') {
    const n = sanitizeFontName(customFont);
    return n ? `'${n}', ${def.stack}` : def.stack;
  }
  return def.stack;
}

/**
 * Weight actually rendered: clamped to the variable axis range, snapped to
 * the nearest static weight, or passed through for system/custom fonts.
 */
export function effectiveWeight(choice: FontChoice, weight: number): number {
  const w = Number.isFinite(weight) ? Math.round(Math.min(900, Math.max(100, weight))) : 400;
  const def = FONTS[choice] ?? FONTS.atkinson;
  if (def.range) return Math.min(def.range[1], Math.max(def.range[0], w));
  if (def.weights?.length) {
    return def.weights.reduce((best, x) => (Math.abs(x - w) < Math.abs(best - w) ? x : best), def.weights[0]);
  }
  return w;
}

/** Human-readable weight range for the settings hint. */
export function weightHint(choice: FontChoice): string {
  const def = FONTS[choice] ?? FONTS.atkinson;
  if (def.range) return `variabel ${def.range[0]}–${Math.min(900, def.range[1])}`;
  if (def.weights) return `nur ${def.weights.join(' / ')}`;
  return 'abhängig von der installierten Schrift';
}
