// Typed wrapper around the Tauri bridge (CONCEPT §8.3). The only file that
// knows command/event names; components import from here.
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  AppState, AudioDevice, Command, EngineInfo, ModelRow, ModelSelection, MonitorInfo, ResyncInfo, ScriptMeta,
  ScrollFrame, UiPrefs, VoiceModelPrefs,
} from './types';

export const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export const api = {
  dispatch: (cmd: Command) => invoke<void>('dispatch', { cmd }),
  getState: () => invoke<AppState>('get_state'),
  importFile: (path: string) => invoke<string>('import_file', { path }),
  saveScript: (id: string, md: string) => invoke<ScriptMeta>('save_script', { id, md }),
  loadScript: (id: string) => invoke<string>('load_script', { id }),
  listScripts: () => invoke<ScriptMeta[]>('list_scripts'),
  deleteScript: (id: string) => invoke<void>('delete_script', { id }),
  reportLayout: (scriptVersion: number, lines: number) => invoke<boolean>('report_layout', { scriptVersion, lines }),
  getUiPrefs: () => invoke<Partial<UiPrefs>>('get_ui_prefs'),
  setUiPrefs: (prefs: UiPrefs) => invoke<void>('set_ui_prefs', { prefs }),
  listMonitors: () => invoke<MonitorInfo[]>('list_monitors'),
  openTalent: (monitor: number | null, fullscreen: boolean) => invoke<void>('open_talent', { monitor, fullscreen }),
  closeTalent: () => invoke<void>('close_talent'),
  setTalentFullscreen: (fullscreen: boolean) => invoke<void>('set_talent_fullscreen', { fullscreen }),
  talentVisible: () => invoke<boolean>('talent_visible'),
  audioDevices: () => invoke<AudioDevice[]>('audio_devices'),
  setAudioDevice: (id: string | null) => invoke<void>('set_audio_device', { id }),
  micTest: (on: boolean) => invoke<void>('mic_test', { on }),
  modelsList: () => invoke<ModelRow[]>('models_list'),
  modelsInstall: (id: string) => invoke<void>('models_install', { id }),
  modelsRemove: (id: string) => invoke<void>('models_remove', { id }),
  voiceModelsGet: () => invoke<VoiceModelPrefs>('voice_models_get'),
  voiceModelsSet: (prefs: VoiceModelPrefs) => invoke<void>('voice_models_set', { prefs }),
  voiceModelsSelection: (lang: string | null) => invoke<ModelSelection>('voice_models_selection', { lang }),
  buildInfo: () =>
    invoke<{ version: string; cpal: boolean; sherpa: boolean; download: boolean; globalShortcuts: [string, string][] }>('build_info'),
};

export const events = {
  patch: (cb: (p: Partial<AppState>) => void) => listen<Partial<AppState>>('state://patch', (e) => cb(e.payload)),
  scroll: (cb: (f: ScrollFrame) => void) => listen<ScrollFrame>('scroll://frame', (e) => cb(e.payload)),
  word: (cb: (w: { token: number; confidence: number }) => void) => listen<{ token: number; confidence: number }>('voice://word', (e) => cb(e.payload)),
  level: (cb: (db: number) => void) => listen<{ db: number }>('voice://level', (e) => cb(e.payload.db)),
  transcript: (cb: (t: { text: string; isFinal: boolean }) => void) =>
    listen<{ text: string; isFinal: boolean }>('voice://transcript', (e) => cb(e.payload)),
  engine: (cb: (e: EngineInfo) => void) => listen<EngineInfo>('voice://engine', (e) => cb(e.payload)),
  models: (cb: (p: ModelProgress) => void) => listen<ModelProgress>('models://progress', (e) => cb(e.payload)),
  resync: (cb: (r: ResyncInfo) => void) => listen<ResyncInfo>('voice://resync', (e) => cb(e.payload)),
  prefs: (cb: (p: Partial<UiPrefs>) => void) => listen<Partial<UiPrefs>>('prefs://changed', (e) => cb(e.payload)),
  talent: (cb: (visible: boolean) => void) => listen<boolean>('talent://visibility', (e) => cb(e.payload)),
};

export interface ModelProgress {
  id: string;
  done: number;
  total: number;
  finished: boolean;
  error: string | null;
  /** [path, computed sha256] of files installed without manifest hash */
  unverified: [string, string][];
}

export type { UnlistenFn };

export function errorText(e: unknown): string {
  if (typeof e === 'string') return e;
  if (e instanceof Error) return e.message;
  return JSON.stringify(e);
}
