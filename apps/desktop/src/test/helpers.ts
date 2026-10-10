// Shared builders for Vitest suites. Pure TS, no DOM library required.
import { buildLayout, type LayoutMap } from '../lib/scroller';
import type { Searchable, PaletteGroup } from '../lib/palette';
import { DEFAULT_PREFS, type UiPrefs } from '../lib/types';

/** Token boxes from per-token `[top, height]` rows (left = 0, width = 10). */
export function boxes(rows: ReadonlyArray<readonly [number, number]>): Float32Array {
  const b = new Float32Array(rows.length * 4);
  rows.forEach(([top, h], i) => {
    b[i * 4 + 1] = top;
    b[i * 4 + 2] = 10;
    b[i * 4 + 3] = h;
  });
  return b;
}

/**
 * Layout from a line spec: `[2, 3, 1]` = three lines of `lineHeight` px with
 * 2, 3 and 1 tokens. Hidden tokens can be added via `hidden` (token ids).
 */
export function layoutOf(tokensPerLine: number[], lineHeight = 100, hidden: number[] = []): LayoutMap {
  const rows: [number, number][] = [];
  tokensPerLine.forEach((n, li) => {
    for (let i = 0; i < n; i++) rows.push([li * lineHeight, lineHeight]);
  });
  for (const h of hidden) if (rows[h]) rows[h] = [rows[h][0], 0];
  return buildLayout(boxes(rows), rows.length);
}

/** Evenly spaced samples in `[from, to)`. */
export function range(from: number, to: number, step: number): number[] {
  const out: number[] = [];
  for (let x = from; x < to - 1e-9; x += step) out.push(x);
  return out;
}

export function prefs(over: Partial<UiPrefs> = {}): UiPrefs {
  return { ...DEFAULT_PREFS, ...over };
}

/** `"--a:1;--b:2"` -> `{ '--a': '1', '--b': '2' }` (values may contain ':' e.g. fonts). */
export function parseVars(css: string): Record<string, string> {
  const out: Record<string, string> = {};
  for (const part of css.split(';')) {
    const i = part.indexOf(':');
    if (i > 0) out[part.slice(0, i)] = part.slice(i + 1);
  }
  return out;
}

export function action(label: string, group: PaletteGroup = 'Wiedergabe', keywords?: string): Searchable & { id: string } {
  return { id: label.toLowerCase().replace(/\s+/g, '-'), label, group, keywords };
}

/**
 * Minimal `document` stand-in for `applyUiTokens` / `setUiTheme` in the node
 * environment (avoids a jsdom dependency). Install with `installFakeDocument()`
 * and restore with the returned function.
 */
export interface FakeStyle {
  id: string;
  textContent: string;
}
export interface FakeDocument {
  styles: FakeStyle[];
  attrs: Map<string, string>;
}

export function installFakeDocument(): { doc: FakeDocument; restore: () => void } {
  const doc: FakeDocument = { styles: [], attrs: new Map() };
  const documentElement = {
    setAttribute: (k: string, v: string) => doc.attrs.set(k, v),
    removeAttribute: (k: string) => doc.attrs.delete(k),
    getAttribute: (k: string) => doc.attrs.get(k) ?? null,
  };
  const fake = {
    documentElement,
    getElementById: (id: string) => doc.styles.find((s) => s.id === id) ?? null,
    createElement: (_tag: string) => ({ id: '', textContent: '' }) as FakeStyle,
    head: { prepend: (el: FakeStyle) => doc.styles.unshift(el) },
  };
  const g = globalThis as unknown as { document?: unknown };
  const prev = g.document;
  g.document = fake;
  return {
    doc,
    restore: () => {
      if (prev === undefined) delete g.document;
      else g.document = prev;
    },
  };
}
