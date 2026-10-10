import { describe, expect, it } from 'vitest';
import { bindingMap, chordOf, EXTRA_BINDINGS, normalizeChord } from './hotkeys';

const ev = (key: string, o: Partial<KeyboardEvent> = {}) =>
  ({ key, code: '', ctrlKey: false, metaKey: false, altKey: false, shiftKey: false, ...o }) as KeyboardEvent;

describe('chordOf', () => {
  it.each([
    [ev(' '), 'Space'],
    [ev('x', { code: 'Space' }), 'Space'],
    [ev('k', { ctrlKey: true }), 'Ctrl+K'],
    [ev('k', { metaKey: true }), 'Ctrl+K'], // ⌘ == Ctrl
    [ev('M', { shiftKey: true }), 'Shift+M'],
    [ev('?', { shiftKey: true }), '?'], // shift implied by the symbol
    [ev('ArrowUp'), 'ArrowUp'],
    [ev('+', { ctrlKey: true }), 'Ctrl+='],
    [ev('1', { ctrlKey: true, altKey: true }), 'Ctrl+Alt+1'],
    [ev(',', { ctrlKey: true }), 'Ctrl+,'], // settings
    [ev('b', { ctrlKey: true }), 'Ctrl+B'], // script drawer
  ])('%#: -> %s', (e, chord) => {
    expect(chordOf(e)).toBe(chord);
  });
});

describe('normalizeChord', () => {
  it.each([
    ['ctrl+k', 'Ctrl+K'],
    ['Shift+Ctrl+m', 'Ctrl+Shift+M'],
    ['Cmd+1', 'Ctrl+1'],
    ['meta+ctrl+k', 'Ctrl+K'], // duplicate modifier collapses
    [' alt + shift + x ', 'Alt+Shift+X'],
    ['PageDown', 'PageDown'],
  ])('%s -> %s', (raw, n) => {
    expect(normalizeChord(raw)).toBe(n);
  });

  it('is idempotent', () => {
    for (const c of ['Ctrl+K', 'Shift+M', 'Space', '?', 'Ctrl+Alt+Shift+Z']) {
      expect(normalizeChord(normalizeChord(c))).toBe(normalizeChord(c));
    }
  });

  it('agrees with chordOf for recorded events', () => {
    expect(normalizeChord('shift+ctrl+k')).toBe(chordOf(ev('k', { ctrlKey: true, shiftKey: true })));
  });
});

describe('EXTRA_BINDINGS', () => {
  it('binds the settings modal and the script drawer', () => {
    const m = bindingMap({}, EXTRA_BINDINGS);
    expect(m.get(chordOf(ev(',', { ctrlKey: true })))).toBe('openSettings');
    expect(m.get(chordOf(ev('b', { metaKey: true })))).toBe('toggleSidebar');
  });
});

describe('bindingMap', () => {
  it('settings override extra bindings for the same chord', () => {
    const m = bindingMap({ custom: 'ctrl+e' }, { toggleMode: 'Ctrl+E', openFile: 'Ctrl+O' });
    expect(m.get('Ctrl+E')).toBe('custom');
    expect(m.get('Ctrl+O')).toBe('openFile');
  });
});
