/// <reference types="vitest" />
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { execSync } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));

// Tauri expects a fixed dev port and no browser auto-open.
const host = process.env.TAURI_DEV_HOST;
const repoRoot = resolve(here, '../..');

// Build commit for the "Über" tab: CI sets GITHUB_SHA, locally ask git.
function commit(): string {
  if (process.env.GITHUB_SHA) return process.env.GITHUB_SHA.slice(0, 12);
  try {
    return execSync('git rev-parse --short=12 HEAD', { cwd: here, stdio: ['ignore', 'pipe', 'ignore'] }).toString().trim();
  } catch {
    return '';
  }
}

export default defineConfig({
  plugins: [svelte()],
  define: { __APP_COMMIT__: JSON.stringify(commit()) },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    // tokens.json lives in docs/design (single source of truth for Rust + UI).
    fs: { allow: [repoRoot] },
    watch: { ignored: ['**/src-tauri/**'] },
  },
  envPrefix: ['VITE_', 'TAURI_ENV_'],
  build: {
    // Two entry points = two windows (control + render-only talent route).
    rollupOptions: {
      input: {
        control: resolve(here, 'index.html'),
        talent: resolve(here, 'talent.html'),
      },
    },
    target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari15',
    minify: process.env.TAURI_ENV_DEBUG ? false : 'esbuild',
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
  test: {
    include: ['src/**/*.test.ts'],
    environment: 'node',
  },
});
