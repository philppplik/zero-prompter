# ADR-001: UI Framework – Tauri 2 + Svelte 5

> German original: [de/ADR-001-ui-framework.md](de/ADR-001-ui-framework.md)

- **Status:** accepted
- **Source:** taken over from `docs/research/RESEARCH.md` §4 (2026-10-10)

## Context

The requirements are a native .exe plus macOS and Linux builds, Apple-level polish, Rust performance, an installer, and updates.

## Options

- Tauri 2 (111.7k stars, v2.12.2 of 2026-10-09, verified according to the research)
- Slint (24.1k stars)
- egui, iced, Dioxus, GPUI (gpui-component, licence NOASSERTION), Makepad, Floem/Xilem

## Decision

Tauri 2.12.x + Svelte 5 + TypeScript + Vite. Components: Bits UI or shadcn-svelte.

## Rationale

- A web stack is the fastest route to premium polish; text rendering and fonts are very good.
- Tauri ships a bundler for all three platforms, an updater and signing support; permissive licence.
- Svelte rather than Solid: Solid has not published a GitHub release since v1.9.0 (09/2024). Svelte has the larger ecosystem.

## Consequences

- Linux/WebKitGTK is a risk. Fallback plan: `WEBKIT_DISABLE_DMABUF_RENDERER=1` on NVIDIA (`ZT_SAFE_RENDER=1`), a performance mode without `backdrop-filter`, later a CEF runtime, and as a last resort a native talent-window renderer (wgpu/egui or Slint).
- The scroll engine lives in Rust; the UI only interpolates. A native output renderer can still be added later.
- The exact Tauri and plugin versions and several plugin APIs (global-shortcut handler, permission names) are not yet verified in code, see `docs/book/src/status.md`.
