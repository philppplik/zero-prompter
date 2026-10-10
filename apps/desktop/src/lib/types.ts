// Mirrors the serde JSON of zt-core (crates/zt-core/src/*.rs). Keep in sync.

export type TokenId = number;

export interface Token {
  id: TokenId;
  text: string;
  range: { start: number; end: number };
  paragraph: number;
  is_cue: boolean; // Script/Token have no serde rename -> snake_case
}

export interface Paragraph {
  index: number;
  tokens: { start: number; end: number };
}

export interface Script {
  source: string;
  tokens: Token[];
  paragraphs: Paragraph[];
  version: number;
}

export type UiMode = 'Edit' | 'Live';

export type PlaybackState =
  | { kind: 'stopped' }
  | { kind: 'countdown'; remaining: number; ms_left: number }
  | { kind: 'playing' }
  | { kind: 'paused' }
  | { kind: 'ended' };

export interface Playback {
  mode: UiMode;
  state: PlaybackState;
  countdown_secs: number;
}

export type ScrollMode = 'manual' | 'constant' | 'voiceActivated' | 'voiceTracking';
export type VoicePhase = 'idle' | 'listening' | 'tracking' | 'hold' | 'lost' | 'resync';
export type LostPolicy = 'stop' | 'constantSpeed';

export interface ScrollState {
  mode: ScrollMode;
  position: number;
  target: number;
  wpm: number;
}

export interface VoiceView {
  enabled: boolean;
  phase: VoicePhase;
  confidence: number;
  cursorToken: TokenId;
  rateWps: number;
  overrideMs: number;
}

export interface OutputState {
  monitorId: number | null;
  fullscreen: boolean;
  mirrorH: boolean;
  mirrorV: boolean;
  overlay: boolean;
}

export interface Settings {
  theme: string;
  fontSizePx: number;
  lineHeight: number;
  focusLine: number;
  wpm: number;
  countdownSecs: number;
  lostPolicy: LostPolicy;
  lostTimeoutMs: number;
  manualOverrideMs: number;
  language: string;
  reducedMotion: boolean;
  hotkeys: Record<string, string>;
}

export interface AppState {
  script: Script;
  playback: Playback;
  scroll: ScrollState;
  voice: VoiceView;
  output: OutputState;
  settings: Settings;
}

/** zt_core::Command, serde(tag = "type", content = "value", camelCase). */
export type Command =
  | { type: 'loadScript'; value: string }
  | { type: 'editScript'; value: string }
  | { type: 'enterLive' }
  | { type: 'enterEdit' }
  | { type: 'togglePlay' }
  | { type: 'play' }
  | { type: 'pause' }
  | { type: 'stop' }
  | { type: 'setWpm'; value: number }
  | { type: 'nudgeWpm'; value: number }
  | { type: 'jumpParagraph'; value: number }
  | { type: 'seekToken'; value: TokenId }
  | { type: 'resyncHere'; value: TokenId }
  | { type: 'setMirror'; value: { h: boolean; v: boolean } }
  | { type: 'toggleMirrorH' }
  | { type: 'setFullscreen'; value: boolean }
  | { type: 'setScrollMode'; value: ScrollMode }
  | { type: 'toggleVoice' }
  | { type: 'setTheme'; value: string }
  | { type: 'setFontSize'; value: number }
  | { type: 'replaceSettings'; value: Settings };

export interface ScrollFrame {
  position: number;
  target: number;
  velocity: number;
  playing: boolean;
  scriptVersion: number;
}

export interface MonitorInfo {
  index: number;
  name: string;
  x: number;
  y: number;
  width: number;
  height: number;
  scale: number;
  primary: boolean;
}

export interface AudioDevice {
  id: string;
  name: string;
  isDefault: boolean;
}

export interface ScriptMeta {
  id: string;
  title: string;
  modifiedMs: number;
  words: number;
}

export type ModelType = 'online_transducer' | 'offline_nemo_transducer';
export type ModelRole = 'streaming' | 'resync';

export interface ModelRow {
  entry: {
    id: string;
    name: string;
    languages: string[];
    tier: 's' | 'm' | 'l';
    engine: 'sherpaOnlineTransducer' | 'sherpaOfflineTransducer' | 'sherpaVad';
    license: string;
    attribution?: string | null;
    defaultRank?: number | null;
  };
  status: 'installed' | 'notInstalled' | 'incomplete';
  missing: string[];
  totalSize: number;
  requiresAttribution: boolean;
  modelType: ModelType | null;
  roles: ModelRole[];
  /** Files without manifest hash: [path, computed sha256 after download | null]. */
  unverified: [string, string | null][];
}

/** Per-language model choice + resync switch (backend `voice-models.json`, ADR-006). */
export interface VoiceModelPrefs {
  /** primary language -> streaming model id; missing = automatic */
  streaming: Record<string, string>;
  resync: boolean;
  resyncModel: string | null;
}

export interface ModelSelection {
  language: string;
  /** installed streaming models in fallback order; [0] is used */
  streamingChain: string[];
  streamingOptions: string[];
  resync: string | null;
  resyncOptions: string[];
  sileroVad: boolean;
}

export interface EngineInfo {
  name: string;
  /** Demo engine (auto-reads the script); only when no model / no STT in the build. */
  demo: boolean;
  /** Why no real engine runs: 'noModel' | 'noSherpa' | 'engineError'. */
  reason: 'noModel' | 'noSherpa' | 'engineError' | null;
  microphone: boolean;
  error: string | null;
  resync: boolean;
  notes: string[];
  language: string;
  audio: string | null;
}

export interface LatencySummary {
  count: number;
  total: number;
  p50Ms: number | null;
  p95Ms: number | null;
  maxMs: number | null;
  lastMs: number | null;
}

export interface DiagErrorEntry {
  atMs: number;
  source: string;
  message: string;
}

/** Backend `diag_snapshot` (diagnostics settings tab). */
export interface DiagReport {
  version: string;
  os: string;
  arch: string;
  sherpa: boolean;
  cpal: boolean;
  running: boolean;
  tracking: boolean;
  engine: EngineInfo | null;
  audio: string | null;
  levelDb: number;
  framesPerS: number;
  tickHz: number;
  voiceState: string;
  confidence: number;
  lastPartial: string | null;
  latency: {
    audioToPartial: LatencySummary;
    onsetToPartial: LatencySummary;
    hypToCursor: LatencySummary;
    endToEnd: LatencySummary;
  };
  errors: DiagErrorEntry[];
  logDir: string | null;
  selection: ModelSelection | null;
}

export interface ResyncInfo {
  text: string;
  ageMs: number;
  outcome: 'ignored' | 'noMatch' | 'confirmed' | 'reanchored';
}

export type MarkerStyle = 'line' | 'band';
export type FontChoice = 'atkinson' | 'inter' | 'system' | 'serif' | 'mono';

/** Frontend-only preferences (persisted via `set_ui_prefs`, broadcast to all windows). */
export interface UiPrefs {
  fontFamily: FontChoice;
  measureCh: number;
  markerStyle: MarkerStyle;
  dimRead: boolean;
  readOpacity: number | null; // null = theme default
  textColor: string | null; // null = theme default
  bgColor: string | null;
  markerColor: string | null;
  uiTheme: 'system' | 'light' | 'dark';
  onboarded: boolean;
  lastScriptId: string | null;
  inspectorOpen: Record<string, boolean>;
  showFrameStats: boolean;
}

export const DEFAULT_PREFS: UiPrefs = {
  fontFamily: 'atkinson',
  measureCh: 32,
  markerStyle: 'line',
  dimRead: true,
  readOpacity: null,
  textColor: null,
  bgColor: null,
  markerColor: null,
  uiTheme: 'system',
  onboarded: false,
  lastScriptId: null,
  inspectorOpen: { appearance: true, scroll: true, marker: false, voice: false, output: false },
  showFrameStats: false,
};
