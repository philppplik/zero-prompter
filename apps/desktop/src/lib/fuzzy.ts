// Tiny subsequence fuzzy matcher for the command palette (DESIGN §3.4).
// Returns a score (higher = better) and matched character indices for bolding.

export interface FuzzyHit {
  score: number;
  indices: number[];
}

const fold = (s: string) =>
  s.toLowerCase().replace(/ä/g, 'a').replace(/ö/g, 'o').replace(/ü/g, 'u').replace(/ß/g, 's');

export function fuzzy(query: string, text: string): FuzzyHit | null {
  const q = fold(query.trim());
  if (!q) return { score: 0, indices: [] };
  const t = fold(text);
  // Folding keeps lengths for umlauts (1:1 replacements), so indices map back.
  const indices: number[] = [];
  let score = 0;
  let ti = 0;
  let prev = -2;
  for (const ch of q) {
    if (ch === ' ') continue;
    const found = t.indexOf(ch, ti);
    if (found < 0) return null;
    indices.push(found);
    const wordStart = found === 0 || /[\s\-_/·:]/.test(t[found - 1]);
    score += 1 + (found === prev + 1 ? 3 : 0) + (wordStart ? 2 : 0);
    prev = found;
    ti = found + 1;
  }
  score -= (t.length - q.length) * 0.01; // prefer shorter labels
  if (t.startsWith(q)) score += 5;
  return { score, indices };
}

/** Splits `text` into segments for rendering with highlighted matches. */
export function highlight(text: string, indices: number[]): { s: string; hit: boolean }[] {
  const set = new Set(indices);
  const out: { s: string; hit: boolean }[] = [];
  for (let i = 0; i < text.length; i++) {
    const hit = set.has(i);
    const last = out[out.length - 1];
    if (last && last.hit === hit) last.s += text[i];
    else out.push({ s: text[i], hit });
  }
  return out;
}
