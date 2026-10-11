import { describe, expect, it } from 'vitest';
import { engineHeadline, fmtMs, showDemoBadge } from './diagnostics';
import type { EngineInfo } from './types';

const base: EngineInfo = {
  name: 'Streaming Zipformer EN 20M int8 (2023-02-17)',
  demo: false,
  reason: null,
  microphone: true,
  error: null,
  resync: false,
  notes: [],
  language: 'de',
  audio: 'Mic · 48000 Hz · 1 ch · F32 -> 16 kHz mono',
};

describe('diagnostics helpers', () => {
  it('formats milliseconds', () => {
    expect(fmtMs(null)).toBe('–');
    expect(fmtMs(undefined)).toBe('–');
    expect(fmtMs(3.14159)).toBe('3.1 ms');
    expect(fmtMs(212.6)).toBe('213 ms');
  });

  it('shows the Demo badge only without model / without STT', () => {
    expect(showDemoBadge(null)).toBe(false);
    expect(showDemoBadge(base)).toBe(false);
    expect(showDemoBadge({ ...base, demo: true, reason: 'noModel' })).toBe(true);
    expect(showDemoBadge({ ...base, demo: true, reason: 'noSherpa' })).toBe(true);
    // A broken model is an error, never "Demo".
    expect(showDemoBadge({ ...base, demo: false, reason: 'engineError', error: 'x' })).toBe(false);
  });

  it('builds the engine headline', () => {
    expect(engineHeadline({ engine: base, running: true, sherpa: true })).toBe(base.name);
    expect(engineHeadline({ engine: { ...base, microphone: false, resync: true }, running: true, sherpa: true })).toBe(
      `${base.name} · Resync an · kein Mikrofon`,
    );
    expect(engineHeadline({ engine: { ...base, demo: true, reason: 'noModel' }, running: true, sherpa: true })).toMatch(/Modell laden/);
    expect(engineHeadline({ engine: { ...base, reason: 'engineError', error: 'missing x.onnx' }, running: true, sherpa: true })).toBe(
      'Fehler: missing x.onnx',
    );
    expect(engineHeadline({ engine: null, running: false, sherpa: true })).toMatch(/nicht gestartet/);
  });
});
