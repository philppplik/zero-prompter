// Pure helpers for the diagnostics panel and the voice indicator (testable without Tauri).
import type { DiagReport, EngineInfo } from './types';

export type LatencyKey = keyof DiagReport['latency'];

export const LATENCY_ROWS: { key: LatencyKey; label: string; hint: string }[] = [
  { key: 'onsetToPartial', label: 'Sprechbeginn → erste Hypothese', hint: 'Erster lauter Audio-Chunk bis zur ersten Partial-Hypothese (inkl. Modell-Lookahead)' },
  { key: 'audioToPartial', label: 'Audio-Chunk → Hypothese', hint: 'Aufnahmezeitpunkt des Chunks bis Partial (Queue + Decode)' },
  { key: 'hypToCursor', label: 'Hypothese → Cursor', hint: 'Tracker-Alignment bis Cursor-Event' },
  { key: 'endToEnd', label: 'Audio → Cursor (gesamt)', hint: 'Aufnahme des Chunks bis voice://word (ohne Scroll-Animation)' },
];

export function fmtMs(v: number | null | undefined): string {
  if (v === null || v === undefined || !Number.isFinite(v)) return '–';
  return v < 10 ? `${v.toFixed(1)} ms` : `${Math.round(v)} ms`;
}

/** Whether the "Demo" badge (with "Modell laden" CTA) must be shown. */
export function showDemoBadge(engine: EngineInfo | null | undefined): boolean {
  return !!engine && engine.demo && (engine.reason === 'noModel' || engine.reason === 'noSherpa');
}

export function engineHeadline(r: Pick<DiagReport, 'engine' | 'running' | 'sherpa'>): string {
  const e = r.engine;
  if (!e) return r.running ? 'Engine startet…' : 'Voice noch nicht gestartet (Mikrofontest oder Voice-Tracking starten)';
  if (e.reason === 'noSherpa') return 'Demo: dieser Build enthält keine Spracherkennung';
  if (e.reason === 'noModel') return 'Demo: kein Sprachmodell installiert – Modell laden';
  if (e.reason === 'engineError') return `Fehler: ${e.error ?? 'Modell konnte nicht geladen werden'}`;
  const extras = [e.resync ? 'Resync an' : null, e.microphone ? null : 'kein Mikrofon'].filter(Boolean);
  return `${e.name}${extras.length ? ` · ${extras.join(' · ')}` : ''}`;
}
