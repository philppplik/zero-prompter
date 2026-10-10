// Pure command-palette search (DESIGN §3.4), extracted from
// CommandPalette.svelte so it can be unit-tested without Tauri/DOM.
import { fuzzy } from './fuzzy';

export const PALETTE_GROUPS = ['Wiedergabe', 'Darstellung', 'Themes', 'Ausgabe', 'Voice', 'Skripte', 'Hilfe'] as const;
export type PaletteGroup = (typeof PALETTE_GROUPS)[number];

export interface Searchable {
  label: string;
  group: PaletteGroup;
  keywords?: string;
}

export interface PaletteHit<T extends Searchable> {
  a: T;
  score: number;
  /** Matched indices inside `label` only (keyword hits are not bolded). */
  idx: number[];
}

export const PALETTE_LIMIT = 40;

/**
 * Empty query: every action, ordered by group (stable within a group).
 * Non-empty query: fuzzy hits on `label + keywords`, best score first.
 */
export function searchActions<T extends Searchable>(all: readonly T[], query: string, limit = PALETTE_LIMIT): PaletteHit<T>[] {
  const hits: PaletteHit<T>[] = [];
  for (const a of all) {
    const h = fuzzy(query, `${a.label} ${a.keywords ?? ''}`);
    if (h) hits.push({ a, score: h.score, idx: h.indices.filter((i) => i < a.label.length) });
  }
  if (query.trim()) hits.sort((x, y) => y.score - x.score);
  else hits.sort((x, y) => PALETTE_GROUPS.indexOf(x.a.group) - PALETTE_GROUPS.indexOf(y.a.group));
  return hits.slice(0, limit);
}
