// Reactive mirror of the Rust state (Svelte 5 runes). Rust is the single
// source of truth: the UI never mutates `state` locally, it dispatches commands
// and applies `state://patch` events.
import { api, errorText, events, type UnlistenFn } from './bridge';
import { migrateLegacyStyle, sanitizeMarker } from './marker';
import { DEFAULT_PREFS, type AppState, type Command, type EngineInfo, type ResyncInfo, type UiPrefs } from './types';

export type SettingsTab = 'appearance' | 'marker' | 'scroll' | 'voice' | 'hotkeys' | 'output' | 'diagnostics' | 'about';

export interface Toast {
  id: number;
  text: string;
  kind: 'info' | 'error';
}

/** Focus line bounds (viewport fraction). Must match `Settings::FOCUS_LINE_MIN/MAX` in zt-core. */
export const FOCUS_LINE_MIN = 0.05;
export const FOCUS_LINE_MAX = 0.7;
export const clampFocusLine = (f: number): number => Math.min(FOCUS_LINE_MAX, Math.max(FOCUS_LINE_MIN, f));

/** Latest scroll frame (plain object, read by rAF loops without reactivity overhead). */
export const frame = { position: 0, target: 0, velocity: 0, playing: false, receivedAt: 0, version: 0 };

class Store {
  state = $state<AppState | null>(null);
  prefs = $state<UiPrefs>({ ...DEFAULT_PREFS });
  /** Throttled (~10 Hz) copy of the scroll position for text UI (transport, outline). */
  position = $state(0);
  level = $state(-100);
  transcript = $state('');
  engine = $state<EngineInfo | null>(null);
  lastResync = $state<ResyncInfo | null>(null);
  word = $state<{ token: number; confidence: number } | null>(null);
  toasts = $state<Toast[]>([]);
  talentVisible = $state(false);
  paletteOpen = $state(false);
  shortcutsOpen = $state(false);
  onboardingOpen = $state(false);
  /** Script library drawer (hidden by default, Ctrl+B / pill). */
  sidebarOpen = $state(false);
  /** Settings modal (Ctrl+, / pill) and its active tab. */
  settingsOpen = $state(false);
  settingsTab = $state<SettingsTab>('appearance');
  scriptId = $state<string | null>(null);
  modelProgress = $state<Record<string, { done: number; total: number; error: string | null; finished: boolean }>>({});

  private nextToast = 1;
  private lastPosPush = 0;
  private prefsTimer: ReturnType<typeof setTimeout> | null = null;

  async init(): Promise<UnlistenFn[]> {
    const un: UnlistenFn[] = [];
    un.push(await events.patch((p) => this.applyPatch(p)));
    un.push(
      await events.scroll((f) => {
        frame.position = f.position;
        frame.target = f.target;
        frame.velocity = f.velocity;
        frame.playing = f.playing;
        frame.version = f.scriptVersion;
        frame.receivedAt = performance.now();
        if (frame.receivedAt - this.lastPosPush > 100 || !f.playing) {
          this.lastPosPush = frame.receivedAt;
          this.position = f.position;
        }
      }),
    );
    un.push(await events.word((w) => (this.word = w)));
    un.push(await events.level((db) => (this.level = db)));
    un.push(await events.transcript((t) => (this.transcript = t.text)));
    un.push(await events.engine((e) => {
      // Errors arrive separately via diag://error (one toast per error).
      this.engine = e;
      for (const n of e.notes ?? []) this.toast(`Voice: ${n}`, 'info');
    }));
    un.push(await events.diagError((e) => this.toast(`${e.source}: ${e.message}`, 'error', 7000)));
    this.forwardFrontendErrors();
    un.push(await events.resync((r) => (this.lastResync = r)));
    un.push(await events.prefs((p) => Object.assign(this.prefs, p)));
    un.push(await events.talent((v) => (this.talentVisible = v)));
    un.push(
      await events.models((p) => {
        this.modelProgress[p.id] = { done: p.done, total: p.total, error: p.error, finished: p.finished };
        if (!p.finished) return;
        if (p.error) this.toast(`Download fehlgeschlagen: ${p.error}`, 'error');
        else if (p.unverified?.length)
          this.toast(`Modell installiert · ${p.unverified.length} Datei(en) unverifiziert (kein Hash im Manifest)`, 'info');
        else this.toast('Modell installiert', 'info');
      }),
    );
    const [st, prefs, visible] = await Promise.all([api.getState(), api.getUiPrefs(), api.talentVisible()]);
    this.state = st;
    Object.assign(this.prefs, { ...DEFAULT_PREFS, ...prefs });
    this.talentVisible = visible;
    // One-time migration: legacy UI pref markerStyle 'band' -> Settings.marker.style.
    const migrated = migrateLegacyStyle(this.prefs.markerStyle, sanitizeMarker(st.settings.marker));
    if (migrated) {
      this.send({ type: 'replaceSettings', value: { ...st.settings, marker: migrated } });
      this.setPrefs({ markerStyle: 'line' });
    }
    frame.position = frame.target = this.position = st.scroll.position;
    frame.version = st.script.version;
    return un;
  }

  /** Uncaught frontend errors go to the backend log too (no silent failures). */
  private forwardFrontendErrors(): void {
    if (typeof window === 'undefined') return;
    const send = (msg: string) => api.diagLogFrontend('error', msg).catch(() => {});
    window.addEventListener('error', (e) => send(`${e.message} @ ${e.filename}:${e.lineno}`));
    window.addEventListener('unhandledrejection', (e) => send(`unhandled rejection: ${errorText(e.reason)}`));
  }

  /** Opens the settings dialog on a tab (e.g. "Modell laden" CTA -> voice). */
  openSettings(tab: SettingsTab): void {
    this.settingsTab = tab;
    this.settingsOpen = true;
  }

  applyPatch(p: Partial<AppState>): void {
    if (!this.state) return;
    Object.assign(this.state, p);
    if (p.scroll && !frame.playing) {
      frame.position = p.scroll.position;
      frame.target = p.scroll.target;
      this.position = p.scroll.position;
    }
  }

  /** Dispatch a command; errors become toasts (e.g. "script is locked while playing"). */
  async send(cmd: Command): Promise<boolean> {
    try {
      await api.dispatch(cmd);
      return true;
    } catch (e) {
      this.toast(errorText(e), 'error');
      return false;
    }
  }

  toast(text: string, kind: Toast['kind'] = 'info', ms = 2600): void {
    const id = this.nextToast++;
    this.toasts = [...this.toasts.slice(-3), { id, text, kind }];
    setTimeout(() => (this.toasts = this.toasts.filter((t) => t.id !== id)), ms);
  }

  /** Update prefs locally, persist (debounced) and broadcast to the talent window. */
  setPrefs(patch: Partial<UiPrefs>): void {
    Object.assign(this.prefs, patch);
    if (this.prefsTimer) clearTimeout(this.prefsTimer);
    this.prefsTimer = setTimeout(() => {
      api.setUiPrefs($state.snapshot(this.prefs) as UiPrefs).catch((e) => this.toast(errorText(e), 'error'));
    }, 150);
  }

  get live(): boolean {
    return this.state?.playback.mode === 'Live';
  }

  get running(): boolean {
    const k = this.state?.playback.state.kind;
    return k === 'playing' || k === 'countdown';
  }

  get currentToken(): number {
    const n = this.state?.script.tokens.length ?? 0;
    return Math.max(0, Math.min(n - 1, Math.floor(this.position)));
  }
}

export const store = new Store();
