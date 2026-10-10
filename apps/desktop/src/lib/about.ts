// Pure helpers for the "Über" settings tab and the custom window chrome.
// Kept free of Tauri imports so they are unit-testable (vitest, node env).

export const APP_NAME = 'zero-teleprompter';
export const LICENSE = 'MIT OR Apache-2.0';
export const REPO_URL = 'https://github.com/philppplik/zero-prompter';

/** Fixed link keys; the Rust `open_link` command maps them to URLs (no free-form URLs over IPC). */
export type LinkKey = 'repo' | 'notices' | 'models' | 'license' | 'changelog';

export const LINKS: { key: LinkKey; label: string; url: string }[] = [
  { key: 'repo', label: 'Quellcode auf GitHub', url: REPO_URL },
  { key: 'notices', label: 'Third-Party-Notices', url: `${REPO_URL}/blob/main/THIRD_PARTY_NOTICES.md` },
  { key: 'models', label: 'Modelllizenzen', url: `${REPO_URL}/blob/main/THIRD_PARTY_MODELS.md` },
  { key: 'license', label: 'Lizenztexte (MIT / Apache-2.0)', url: `${REPO_URL}#license` },
  { key: 'changelog', label: 'Changelog', url: `${REPO_URL}/blob/main/CHANGELOG.md` },
];

/** "0.1.0 · Build a1b2c3d" – commit is optional (dev builds without git). */
export function formatBuild(version: string | null, commit: string | null | undefined): string {
  const v = version ? `Version ${version}` : 'Version unbekannt';
  const c = commit && /^[0-9a-f]{7,40}$/i.test(commit) ? ` · Build ${commit.slice(0, 7)}` : '';
  return v + c;
}

export type Platform = 'macos' | 'windows' | 'linux';

/** Platform from the user agent (no plugin-os needed). */
export function platformOf(ua: string): Platform {
  if (/Mac OS X|Macintosh/i.test(ua)) return 'macos';
  if (/Windows/i.test(ua)) return 'windows';
  return 'linux';
}

/** macOS keeps native traffic lights (overlay title bar); others get custom controls. */
export function customWindowControls(p: Platform): boolean {
  return p !== 'macos';
}
