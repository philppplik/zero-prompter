import { describe, expect, it } from 'vitest';
import { fuzzy, highlight } from './fuzzy';
import { PALETTE_GROUPS, PALETTE_LIMIT, searchActions } from './palette';
import { action } from '../test/helpers';

describe('fuzzy', () => {
  it.each([
    ['thm', 'Theme: Daylight', true],
    ['xyz', 'Theme', false],
    ['grö', 'Schrift größer', true], // umlaut in query
    ['gro', 'Schrift größer', true], // umlaut folded in text
    ['SPIEGELN', 'spiegeln', true], // case-insensitive
    ['sp ie', 'Spiegeln', true], // spaces in the query are ignored
    ['nlg', 'Spiegeln', false], // order matters (subsequence)
  ])('%s ~ %s -> %s', (q, t, ok) => {
    expect(fuzzy(q, t) !== null).toBe(ok);
  });

  it('empty / whitespace query matches everything with score 0', () => {
    expect(fuzzy('', 'abc')).toEqual({ score: 0, indices: [] });
    expect(fuzzy('   ', 'abc')).toEqual({ score: 0, indices: [] });
  });

  it('prefers prefix > word start > scattered', () => {
    const s = (t: string) => fuzzy('spi', t)!.score;
    expect(s('Spiegeln')).toBeGreaterThan(s('Absatz spielen'));
    expect(s('Absatz spielen')).toBeGreaterThan(s('Xsxpxi'));
  });

  it('returns indices that point at the matched characters', () => {
    const t = 'Schrift größer';
    const h = fuzzy('grö', t)!;
    expect(h.indices.map((i) => t[i].toLowerCase()).join('')).toBe('grö');
  });
});

describe('highlight', () => {
  it.each([
    ['abc', [1], [{ s: 'a', hit: false }, { s: 'b', hit: true }, { s: 'c', hit: false }]],
    ['abc', [], [{ s: 'abc', hit: false }]],
    ['abc', [0, 1, 2], [{ s: 'abc', hit: true }]],
    ['', [], []],
  ] as const)('%s %j', (text, idx, expected) => {
    expect(highlight(text, [...idx])).toEqual(expected);
  });

  it('round-trips the text', () => {
    const t = 'Theme: High Contrast';
    expect(highlight(t, fuzzy('hc', t)!.indices).map((x) => x.s).join('')).toBe(t);
  });
});

describe('searchActions (command palette)', () => {
  const all = [
    action('Hilfe anzeigen', 'Hilfe'),
    action('Spiegeln horizontal', 'Ausgabe', 'mirror flip'),
    action('Wiedergabe starten', 'Wiedergabe', 'play'),
    action('Theme: Daylight', 'Themes'),
    action('Schrift größer', 'Darstellung', 'font bigger zoom'),
    action('Voice Tracking an/aus', 'Voice'),
  ];

  it('empty query lists everything grouped in palette order', () => {
    const groups = searchActions(all, '').map((h) => h.a.group);
    const order = groups.map((g) => PALETTE_GROUPS.indexOf(g));
    expect(order).toEqual([...order].sort((a, b) => a - b));
    expect(groups).toHaveLength(all.length);
  });

  it('ranks the best match first', () => {
    expect(searchActions(all, 'spieg')[0].a.label).toBe('Spiegeln horizontal');
    expect(searchActions(all, 'wied')[0].a.label).toBe('Wiedergabe starten');
  });

  it('finds actions by keywords but only bolds label characters', () => {
    const hits = searchActions(all, 'mirror');
    expect(hits[0].a.label).toBe('Spiegeln horizontal');
    for (const i of hits[0].idx) expect(i).toBeLessThan(hits[0].a.label.length);
  });

  it('returns nothing for a query without matches', () => {
    expect(searchActions(all, 'qqqqzz')).toEqual([]);
  });

  it('caps the result list', () => {
    const many = Array.from({ length: 100 }, (_, i) => action(`Aktion ${i}`));
    expect(searchActions(many, '')).toHaveLength(PALETTE_LIMIT);
    expect(searchActions(many, 'akt', 5)).toHaveLength(5);
  });

  it('does not mutate the input list', () => {
    const copy = [...all];
    searchActions(all, 'e');
    expect(all).toEqual(copy);
  });
});
