// In-app keyboard handling (DESIGN §5). Chords use the format stored in
// `Settings.hotkeys`, e.g. "Space", "Ctrl+K", "Shift+M", "ArrowUp", "?".
// On macOS ⌘ (Meta) is treated as Ctrl so one binding works everywhere.

export const isMac = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);

export function chordOf(e: Pick<KeyboardEvent, 'key' | 'ctrlKey' | 'metaKey' | 'altKey' | 'shiftKey' | 'code'>): string {
  let key = e.key;
  if (key === ' ' || e.code === 'Space') key = 'Space';
  else if (key.length === 1) key = key.toUpperCase();
  if (key === '+') key = '=';
  const mods: string[] = [];
  if (e.ctrlKey || e.metaKey) mods.push('Ctrl');
  if (e.altKey) mods.push('Alt');
  // Shift is implied by printable symbols like "?" — only keep it for letters/named keys.
  const symbol = key.length === 1 && !/[A-Z0-9]/.test(key);
  if (e.shiftKey && !symbol) mods.push('Shift');
  return [...mods, key].join('+');
}

export function normalizeChord(chord: string): string {
  const parts = chord.split('+').map((p) => p.trim()).filter(Boolean);
  const key = parts.pop() ?? '';
  const mods = parts.map((m) => (/^(cmd|meta|⌘|command|control|ctrl)$/i.test(m) ? 'Ctrl' : m[0].toUpperCase() + m.slice(1).toLowerCase()));
  const order = ['Ctrl', 'Alt', 'Shift'];
  mods.sort((a, b) => order.indexOf(a) - order.indexOf(b));
  const k = key.length === 1 ? key.toUpperCase() : key;
  return [...new Set(mods), k].join('+');
}

/** Pretty label for UI (⌘ on macOS). */
export function displayChord(chord: string): string {
  const n = normalizeChord(chord);
  const map: Record<string, string> = {
    ArrowUp: '↑', ArrowDown: '↓', ArrowLeft: '←', ArrowRight: '→', PageUp: 'PgUp', PageDown: 'PgDn', Escape: 'Esc', Enter: '↵',
  };
  return n
    .split('+')
    .map((p) => (p === 'Ctrl' ? (isMac ? '⌘' : 'Ctrl') : p === 'Shift' && isMac ? '⇧' : p === 'Alt' && isMac ? '⌥' : map[p] ?? p))
    .join(isMac ? '' : '+');
}

/** true if the event target is a text field where plain keys must type text. */
export function isTypingTarget(t: EventTarget | null): boolean {
  if (!(t instanceof HTMLElement)) return false;
  if (t.isContentEditable) return true;
  const tag = t.tagName;
  if (tag === 'TEXTAREA') return true;
  if (tag === 'INPUT') {
    const type = (t as HTMLInputElement).type;
    return !['checkbox', 'radio', 'range', 'button', 'color'].includes(type);
  }
  return tag === 'SELECT';
}

/** Builds chord -> action map from settings (later bindings win). */
export function bindingMap(hotkeys: Record<string, string>, extra: Record<string, string> = {}): Map<string, string> {
  const m = new Map<string, string>();
  for (const [action, chord] of Object.entries({ ...extra, ...hotkeys })) m.set(normalizeChord(chord), action);
  return m;
}

/** Fixed app bindings not (yet) remappable in Settings (DESIGN §5). */
export const EXTRA_BINDINGS: Record<string, string> = {
  toggleMirrorV: 'Shift+M',
  toggleMode: 'Ctrl+E',
  openFile: 'Ctrl+O',
  fontBigger: 'Ctrl+=',
  fontSmaller: 'Ctrl+-',
  theme1: 'Ctrl+1',
  theme2: 'Ctrl+2',
  theme3: 'Ctrl+3',
  theme4: 'Ctrl+4',
  theme5: 'Ctrl+5',
  theme6: 'Ctrl+6',
  escape: 'Escape',
  homeMac: 'Ctrl+ArrowUp',
};

/** System-wide hotkeys registered in src-tauri/src/hotkeys.rs (display only; keep in sync).
 *  Ctrl+Shift+F-keys avoid Ctrl+Shift+V (paste as plain text) and Ctrl+Shift+Arrows (text selection). */
export const GLOBAL_HOTKEYS: Record<string, string> = {
  toggleVoice: 'Ctrl+Shift+F6',
  togglePlay: 'Ctrl+Shift+F7',
  slower: 'Ctrl+Shift+F8',
  faster: 'Ctrl+Shift+F9',
  prevParagraph: 'Ctrl+Shift+F10',
  nextParagraph: 'Ctrl+Shift+F11',
};
