# ADR-001: UI-Framework – Tauri 2 + Svelte 5

- **Status:** angenommen
- **Quelle:** übernommen aus `docs/research/RESEARCH.md` §4 (2026-10-10)

## Kontext

Gefordert sind eine native .exe sowie macOS und Linux, Polish auf Apple-Niveau, Rust-Geschwindigkeit, Installer und Updates.

## Optionen

- Tauri 2 (111,7k Stars, v2.12.2 vom 09.10.2026, laut Research verifiziert)
- Slint (24,1k Stars)
- egui, iced, Dioxus, GPUI (gpui-component, Lizenz NOASSERTION), Makepad, Floem/Xilem

## Entscheidung

Tauri 2.12.x + Svelte 5 + TypeScript + Vite. Komponenten: Bits UI bzw. shadcn-svelte.

## Begründung

- Ein Web-Stack ist der schnellste Weg zu Premium-Polish; Text-Rendering und Fonts sind sehr gut.
- Tauri bringt Bundler für alle drei Plattformen, Updater und Signing mit; permissive Lizenz.
- Svelte statt Solid: Solid hat seit v1.9.0 (09/2024) kein GitHub-Release mehr veröffentlicht. Svelte hat das größere Ökosystem.

## Konsequenzen

- Linux/WebKitGTK ist ein Risiko. Fallback-Plan: `WEBKIT_DISABLE_DMABUF_RENDERER=1` bei NVIDIA (`ZT_SAFE_RENDER=1`), Performance-Modus ohne `backdrop-filter`, später CEF-Runtime, im Notfall ein nativer Talent-Renderer (wgpu/egui oder Slint).
- Die Scroll-Engine liegt in Rust, die UI interpoliert nur. Ein nativer Ausgabe-Renderer bleibt nachrüstbar.
- Die exakten Tauri-/Plugin-Versionen und mehrere Plugin-APIs (global-shortcut-Handler, Permission-Namen) sind im Code noch nicht verifiziert, siehe `docs/book/src/status.md`.
