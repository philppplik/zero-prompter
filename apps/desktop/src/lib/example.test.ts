import { describe, expect, it } from 'vitest';
import { EXAMPLE_SCRIPT } from './example';

describe('example script (voice-tracking test material)', () => {
  const words = EXAMPLE_SCRIPT.replace(/\[[^\]]*\]/g, ' ').split(/\s+/).filter((w) => /[\p{L}\p{N}]/u.test(w));

  it('is long enough for extended voice tests', () => {
    expect(words.length).toBeGreaterThanOrEqual(1100);
  });

  it('contains cues, numbers, headings and many paragraphs', () => {
    expect((EXAMPLE_SCRIPT.match(/\[[^\]\n]+\]/g) ?? []).length).toBeGreaterThanOrEqual(10);
    expect(EXAMPLE_SCRIPT).toMatch(/\d{1,3}\.\d{3}/); // 1.250
    expect(EXAMPLE_SCRIPT).toMatch(/\d+,\d+/); // 3,8
    expect((EXAMPLE_SCRIPT.match(/^## /gm) ?? []).length).toBeGreaterThanOrEqual(10);
    expect(EXAMPLE_SCRIPT.split(/\n\s*\n/).length).toBeGreaterThanOrEqual(50);
  });

  it('has balanced brackets (cues never swallow script text)', () => {
    for (const line of EXAMPLE_SCRIPT.split('\n')) {
      expect((line.match(/\[/g) ?? []).length, line).toBe((line.match(/\]/g) ?? []).length);
    }
  });
});
