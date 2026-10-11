// Every user action lives here once: used by hotkeys, command palette,
// toolbar/transport buttons (DESIGN §1.4 "Tastatur zuerst").
import { open } from '@tauri-apps/plugin-dialog';
import { api, errorText } from './bridge';
import { store } from './store.svelte';
import { THEME_IDS, THEME_LABELS, THEMES } from './tokens';
import type { ScrollMode } from './types';
import { MARKER_STYLES, nextStyle, sanitizeMarker, type MarkerSettings, type MarkerStyle } from './marker';

export interface Action {
  id: string;
  label: string;
  group: 'Wiedergabe' | 'Darstellung' | 'Themes' | 'Ausgabe' | 'Voice' | 'Skripte' | 'Hilfe';
  hint?: string; // right-aligned meta, e.g. contrast "17,0:1"
  keywords?: string;
  run: () => void | Promise<void>;
}

const s = () => store.state;

const curMarker = (): MarkerSettings => sanitizeMarker(store.state?.settings.marker);
const styleLabel = (id: MarkerStyle) => MARKER_STYLES.find((x) => x.id === id)?.label ?? id;

/** Patch the reading marker (core settings, sanitized again in Rust). */
export function updateMarker(patch: Partial<MarkerSettings>): void {
  const st = store.state;
  if (!st) return;
  const settings = JSON.parse(JSON.stringify(st.settings)) as typeof st.settings;
  store.send({ type: 'replaceSettings', value: { ...settings, marker: sanitizeMarker({ ...curMarker(), ...patch }) } });
}

export async function openFileDialog(): Promise<void> {
  try {
    const path = await open({
      multiple: false,
      directory: false,
      filters: [{ name: 'Skript', extensions: ['txt', 'md', 'markdown'] }],
    });
    if (typeof path === 'string') await importPath(path);
  } catch (e) {
    store.toast(errorText(e), 'error');
  }
}

export async function importPath(path: string): Promise<void> {
  try {
    const text = await api.importFile(path);
    const id = newScriptId(path.split(/[\\/]/).pop() ?? 'skript');
    await api.saveScript(id, text);
    store.scriptId = id;
    store.setPrefs({ lastScriptId: id });
    store.toast('Skript importiert');
  } catch (e) {
    store.toast(`Import fehlgeschlagen: ${errorText(e)}`, 'error');
  }
}

export function newScriptId(name: string): string {
  const base = name.replace(/\.[^.]+$/, '').toLowerCase().replace(/[^a-z0-9_-]+/g, '-').replace(/^-+|-+$/g, '').slice(0, 40) || 'skript';
  return `${base}-${Date.now().toString(36)}`;
}

export function toggleMode(): void {
  if (!s()) return;
  store.send({ type: store.live ? 'enterEdit' : 'enterLive' });
}

export function home(): void {
  if (!s()) return;
  if (store.live) store.send({ type: 'stop' });
  else store.send({ type: 'seekToken', value: 0 });
}

export async function setMode(mode: ScrollMode): Promise<void> {
  const st = s();
  if (!st) return;
  const voiceOn = st.voice.enabled;
  // 'voiceActivated' also needs the voice pipeline (it moves only while voice
  // reports speech), otherwise the text would never move.
  const wantsVoice = mode === 'voiceTracking' || mode === 'voiceActivated';
  // Sequential: ToggleVoice itself resets the scroll mode (on -> voiceTracking,
  // off -> constant), so an explicit SetScrollMode must arrive after it, not race it.
  const toggled = wantsVoice !== voiceOn;
  if (toggled && !(await store.send({ type: 'toggleVoice' }))) return;
  const implied: ScrollMode = wantsVoice ? 'voiceTracking' : 'constant';
  if (!toggled || mode !== implied) await store.send({ type: 'setScrollMode', value: mode });
}

export function setTheme(id: string): void {
  store.send({ type: 'setTheme', value: id });
}

export function nudgeFont(delta: number): void {
  const st = s();
  if (st) store.send({ type: 'setFontSize', value: st.settings.fontSizePx + delta });
}

export async function toggleTalent(): Promise<void> {
  try {
    if (store.talentVisible) await api.closeTalent();
    else await api.openTalent(null, false);
  } catch (e) {
    store.toast(errorText(e), 'error');
  }
}

export async function toggleFullscreen(): Promise<void> {
  const st = s();
  if (!st) return;
  try {
    if (!store.talentVisible) await api.openTalent(null, true);
    else await api.setTalentFullscreen(!st.output.fullscreen);
  } catch (e) {
    store.toast(errorText(e), 'error');
  }
}

/** Hotkey action ids (Settings.hotkeys keys + EXTRA_BINDINGS) -> behaviour. */
export function runHotkeyAction(id: string): boolean {
  const st = s();
  if (!st) return false;
  const counting = st.playback.state.kind === 'countdown';
  switch (id) {
    case 'togglePlay': store.send({ type: 'togglePlay' }); break;
    case 'stop': case 'homeMac': home(); break;
    case 'faster': store.send({ type: 'nudgeWpm', value: 5 }).then((ok) => ok && store.toast(`${Math.min(400, st.scroll.wpm + 5)} WPM`)); break;
    case 'slower': store.send({ type: 'nudgeWpm', value: -5 }).then((ok) => ok && store.toast(`${Math.max(40, st.scroll.wpm - 5)} WPM`)); break;
    case 'nextParagraph': store.send({ type: 'jumpParagraph', value: 1 }); break;
    case 'prevParagraph': store.send({ type: 'jumpParagraph', value: -1 }); break;
    case 'toggleMirror': store.send({ type: 'toggleMirrorH' }); break;
    case 'toggleMirrorV': store.send({ type: 'setMirror', value: { h: st.output.mirrorH, v: !st.output.mirrorV } }); break;
    case 'toggleFullscreen': toggleFullscreen(); break;
    case 'toggleVoice': store.send({ type: 'toggleVoice' }); break;
    case 'resyncHere': store.send({ type: 'resyncHere', value: store.currentToken }); break;
    case 'commandPalette': store.paletteOpen = !store.paletteOpen; break;
    case 'shortcutOverlay': store.shortcutsOpen = !store.shortcutsOpen; break;
    case 'toggleMode': toggleMode(); break;
    case 'openFile': openFileDialog(); break;
    case 'toggleSidebar': store.sidebarOpen = !store.sidebarOpen; break;
    case 'openSettings': store.settingsOpen = !store.settingsOpen; break;
    case 'fontBigger': nudgeFont(4); break;
    case 'fontSmaller': nudgeFont(-4); break;
    case 'escape':
      if (store.paletteOpen) store.paletteOpen = false;
      else if (store.shortcutsOpen) store.shortcutsOpen = false;
      else if (store.settingsOpen) store.settingsOpen = false;
      else if (store.sidebarOpen) store.sidebarOpen = false;
      else if (counting) store.send({ type: 'stop' });
      else if (st.output.fullscreen) api.setTalentFullscreen(false).catch(() => {});
      else return false;
      break;
    default: {
      const m = /^theme([1-6])$/.exec(id);
      if (m) {
        setTheme(THEME_IDS[Number(m[1]) - 1]);
        break;
      }
      return false;
    }
  }
  return true;
}

const fmtRatio = (r: number) => r.toFixed(1).replace('.', ',') + ':1';

/** Palette catalogue (recomputed when opened, so labels reflect state). */
export function paletteActions(): Action[] {
  const st = s();
  const playing = st?.playback.state.kind === 'playing';
  const list: Action[] = [
    { id: 'togglePlay', group: 'Wiedergabe', label: playing ? 'Pause' : 'Abspielen', keywords: 'play start', run: () => { runHotkeyAction('togglePlay'); } },
    { id: 'stop', group: 'Wiedergabe', label: 'Zum Anfang', keywords: 'home stop reset', run: home },
    { id: 'toggleMode', group: 'Wiedergabe', label: store.live ? 'Zum Edit-Modus' : 'Live gehen', keywords: 'edit live modus', run: toggleMode },
    { id: 'faster', group: 'Wiedergabe', label: 'Schneller (+5 WPM)', keywords: 'tempo speed', run: () => { runHotkeyAction('faster'); } },
    { id: 'slower', group: 'Wiedergabe', label: 'Langsamer (−5 WPM)', keywords: 'tempo speed', run: () => { runHotkeyAction('slower'); } },
    { id: 'nextParagraph', group: 'Wiedergabe', label: 'Nächster Absatz', run: () => { runHotkeyAction('nextParagraph'); } },
    { id: 'prevParagraph', group: 'Wiedergabe', label: 'Vorheriger Absatz', run: () => { runHotkeyAction('prevParagraph'); } },
    { id: 'mode-manual', group: 'Wiedergabe', label: 'Scroll-Modus: Manuell', run: () => setMode('manual') },
    { id: 'mode-constant', group: 'Wiedergabe', label: 'Scroll-Modus: Konstant', run: () => setMode('constant') },
    { id: 'mode-voice', group: 'Voice', label: 'Scroll-Modus: Voice-Tracking', keywords: 'stimme sprache', run: () => setMode('voiceTracking') },
    { id: 'toggleVoice', group: 'Voice', label: st?.voice.enabled ? 'Voice aus' : 'Voice an', run: () => { runHotkeyAction('toggleVoice'); } },
    { id: 'resyncHere', group: 'Voice', label: 'Resync am Marker', run: () => { runHotkeyAction('resyncHere'); } },
    { id: 'fontBigger', group: 'Darstellung', label: 'Schrift größer', run: () => nudgeFont(4) },
    { id: 'fontSmaller', group: 'Darstellung', label: 'Schrift kleiner', run: () => nudgeFont(-4) },
    { id: 'dim', group: 'Darstellung', label: store.prefs.dimRead ? 'Gelesenen Text nicht dimmen' : 'Gelesenen Text dimmen', run: () => store.setPrefs({ dimRead: !store.prefs.dimRead }) },
    { id: 'marker', group: 'Darstellung', label: `Lesemarker: ${styleLabel(nextStyle(curMarker().style))}`, keywords: 'marker stil linie band', run: () => updateMarker({ style: nextStyle(curMarker().style) }) },
    { id: 'fps', group: 'Darstellung', label: store.prefs.showFrameStats ? 'Frametime-Anzeige aus' : 'Frametime-Anzeige an', keywords: 'fps performance', run: () => store.setPrefs({ showFrameStats: !store.prefs.showFrameStats }) },
    { id: 'toggleMirror', group: 'Ausgabe', label: 'Horizontal spiegeln', keywords: 'mirror', run: () => { runHotkeyAction('toggleMirror'); } },
    { id: 'toggleMirrorV', group: 'Ausgabe', label: 'Vertikal spiegeln', keywords: 'mirror', run: () => { runHotkeyAction('toggleMirrorV'); } },
    { id: 'talent', group: 'Ausgabe', label: store.talentVisible ? 'Talent-Fenster schließen' : 'Talent-Fenster öffnen', keywords: 'output monitor', run: toggleTalent },
    { id: 'toggleFullscreen', group: 'Ausgabe', label: 'Talent-Vollbild umschalten', run: toggleFullscreen },
    { id: 'toggleSidebar', group: 'Skripte', label: store.sidebarOpen ? 'Skripte ausblenden' : 'Skripte & Gliederung anzeigen', keywords: 'sidebar bibliothek outline', run: () => { store.sidebarOpen = !store.sidebarOpen; } },
    { id: 'openSettings', group: 'Hilfe', label: 'Einstellungen…', keywords: 'settings optionen inspector voice modelle', run: () => { store.settingsOpen = true; } },
    { id: 'openFile', group: 'Skripte', label: 'Skript öffnen…', keywords: 'import datei', run: openFileDialog },
    { id: 'shortcuts', group: 'Hilfe', label: 'Tastenkürzel anzeigen', run: () => { store.shortcutsOpen = true; } },
    { id: 'onboarding', group: 'Hilfe', label: 'Einführung erneut starten', keywords: 'onboarding hilfe', run: () => { store.onboardingOpen = true; } },
  ];
  THEME_IDS.forEach((id, i) => {
    const t = THEMES[id];
    list.push({
      id: `theme${i + 1}`,
      group: 'Themes',
      label: `Theme: ${THEME_LABELS[id] ?? id}`,
      hint: fmtRatio(t.contrast),
      run: () => setTheme(id),
    });
  });
  return list;
}

export const ACTION_SHORTCUT: Record<string, string> = {
  toggleMode: 'Ctrl+E',
  openFile: 'Ctrl+O',
  toggleSidebar: 'Ctrl+B',
  openSettings: 'Ctrl+,',
  fontBigger: 'Ctrl+=',
  fontSmaller: 'Ctrl+-',
  toggleMirrorV: 'Shift+M',
  theme1: 'Ctrl+1', theme2: 'Ctrl+2', theme3: 'Ctrl+3', theme4: 'Ctrl+4', theme5: 'Ctrl+5', theme6: 'Ctrl+6',
};
