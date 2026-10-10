import { describe, expect, it } from 'vitest';
import { customWindowControls, formatBuild, LINKS, LICENSE, platformOf, REPO_URL } from './about';

describe('about', () => {
  it('formats version and short commit', () => {
    expect(formatBuild('0.1.0', 'a1b2c3d4e5f6')).toBe('Version 0.1.0 · Build a1b2c3d');
    expect(formatBuild('0.1.0', null)).toBe('Version 0.1.0');
    expect(formatBuild('0.1.0', 'not a sha')).toBe('Version 0.1.0');
    expect(formatBuild(null, undefined)).toBe('Version unbekannt');
  });

  it('links point into the repo and keys are unique', () => {
    expect(LICENSE).toBe('MIT OR Apache-2.0');
    expect(new Set(LINKS.map((l) => l.key)).size).toBe(LINKS.length);
    for (const l of LINKS) expect(l.url.startsWith(REPO_URL)).toBe(true);
  });

  it('detects platform and window chrome', () => {
    expect(platformOf('Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Edg/129')).toBe('windows');
    expect(platformOf('Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15')).toBe('macos');
    expect(platformOf('Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15')).toBe('linux');
    expect(customWindowControls('macos')).toBe(false);
    expect(customWindowControls('windows')).toBe(true);
    expect(customWindowControls('linux')).toBe(true);
  });
});
