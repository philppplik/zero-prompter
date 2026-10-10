<!--
  Control window (CONCEPT §4.1, DESIGN §3.1/§4.1):
  Minimal layout: top bar (logo · action pill · close), one large rounded card
  (Edit: editor, Live: 1:1 talent preview), transport. Scripts drawer (Ctrl+B)
  and settings modal (Ctrl+,) are hidden by default.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import Toolbar from './Toolbar.svelte';
  import Sidebar from './Sidebar.svelte';
  import Editor from './Editor.svelte';
  import Settings from './Settings.svelte';
  import Dialog from './Dialog.svelte';
  import { fade, fly } from 'svelte/transition';
  import Transport from './Transport.svelte';
  import Prompter from './Prompter.svelte';
  import CommandPalette from './CommandPalette.svelte';
  import ShortcutOverlay from './ShortcutOverlay.svelte';
  import Onboarding from './Onboarding.svelte';
  import Toasts from './Toasts.svelte';
  import { store } from '../lib/store.svelte';
  import { api, errorText, isTauri } from '../lib/bridge';
  import { importPath, newScriptId, runHotkeyAction, setMode, toggleMode, toggleTalent } from '../lib/actions';
  import { bindingMap, chordOf, displayChord, EXTRA_BINDINGS, isTypingTarget } from '../lib/hotkeys';
  import { dur, EASE, setReducedMotion } from '../lib/motion';
  import { setUiTheme } from '../lib/tokens';
  import { EXAMPLE_SCRIPT } from '../lib/example';
  import type { ScriptMeta, ScrollMode } from '../lib/types';

  let scripts = $state<ScriptMeta[]>([]);
  let width = $state(1280);
  let previewW = $state(800);
  let dropping = $state(false);
  let lostSince = $state<number | null>(null);
  let now = $state(Date.now());
  let ready = $state(false);
  let initError = $state<string | null>(null);

  const st = $derived(store.state);
  const bindings = $derived(bindingMap(st?.settings.hotkeys ?? {}, EXTRA_BINDINGS));
  const title = $derived(scripts.find((s) => s.id === store.scriptId)?.title ?? (st?.script.tokens.length ? 'Unbenannt' : 'Kein Skript'));
  // Preview = talent geometry scaled down (reference: 1920 px wide output).
  const previewScale = $derived(Math.max(0.2, Math.min(1, previewW / 1920)));
  const lostLong = $derived(lostSince !== null && now - lostSince > 4000);

  $effect(() => setUiTheme(store.prefs.uiTheme));
  $effect(() => setReducedMotion(!!st?.settings.reducedMotion));

  const scrollMode = $derived<ScrollMode>(st?.scroll.mode ?? 'constant');
  const modes: { id: ScrollMode; label: string }[] = [
    { id: 'manual', label: 'Manuell' },
    { id: 'constant', label: 'Konstant' },
    { id: 'voiceTracking', label: 'Voice' },
  ];
  $effect(() => {
    const t = `${title} – zero-teleprompter`;
    document.title = t;
    if (isTauri) getCurrentWindow().setTitle(t).catch(() => {});
  });
  $effect(() => {
    const lost = st?.voice.enabled && st.voice.phase === 'lost';
    if (lost && lostSince === null) lostSince = Date.now();
    if (!lost && lostSince !== null) lostSince = null;
  });

  function setFocusLine(f: number) {
    if (!st) return;
    store.send({ type: 'replaceSettings', value: { ...$state.snapshot(st.settings), focusLine: f } });
  }

  async function refreshScripts() {
    scripts = await api.listScripts().catch(() => []);
  }

  async function loadInitial() {
    await refreshScripts();
    const id = store.prefs.lastScriptId;
    if (id && scripts.some((s) => s.id === id)) {
      try {
        await api.loadScript(id);
        store.scriptId = id;
        return;
      } catch {
        /* fall through to example */
      }
    }
    const exampleId = newScriptId('willkommen');
    await store.send({ type: 'loadScript', value: EXAMPLE_SCRIPT });
    await api.saveScript(exampleId, EXAMPLE_SCRIPT).catch(() => {});
    store.scriptId = exampleId;
    store.setPrefs({ lastScriptId: exampleId });
    await refreshScripts();
  }

  function onKey(e: KeyboardEvent) {
    if (e.defaultPrevented) return;
    const chord = chordOf(e);
    const typing = isTypingTarget(e.target);
    // In text fields only modifier chords (and Escape) act globally.
    if (typing && !(e.ctrlKey || e.metaKey) && e.key !== 'Escape') return;
    if (typing && /^Ctrl\+(A|C|V|X|Z|Y|Shift\+Z|ArrowUp|ArrowDown)$/.test(chord)) return;
    const action = bindings.get(chord);
    if (action && runHotkeyAction(action)) e.preventDefault();
  }

  async function onPaste(e: ClipboardEvent) {
    if (isTypingTarget(e.target) || !st || st.script.tokens.length > 0 || store.running) return;
    const text = e.clipboardData?.getData('text/plain');
    if (text?.trim()) {
      e.preventDefault();
      await store.send({ type: 'loadScript', value: text });
    }
  }

  onMount(() => {
    let unlisten: (() => void)[] = [];
    let alive = true;
    const clock = setInterval(() => (now = Date.now()), 500);
    (async () => {
      try {
        unlisten = await store.init();
        if (!alive) return unlisten.forEach((u) => u());
        await loadInitial();
        if (!store.prefs.onboarded) store.onboardingOpen = true;
        const un = await getCurrentWebview().onDragDropEvent((ev) => {
          const p = ev.payload;
          if (p.type === 'enter' || p.type === 'over') dropping = true;
          else if (p.type === 'leave') dropping = false;
          else if (p.type === 'drop') {
            dropping = false;
            if (p.paths[0]) importPath(p.paths[0]).then(refreshScripts);
          }
        });
        unlisten.push(un);
        ready = true;
      } catch (e) {
        initError = errorText(e);
      }
    })();
    return () => {
      alive = false;
      clearInterval(clock);
      unlisten.forEach((u) => u());
    };
  });
</script>

<svelte:window bind:innerWidth={width} onkeydown={onKey} onpaste={onPaste} />

<a class="skip" href="#transport">Zur Wiedergabe</a>

{#if initError}
  <div class="fatal">
    <h1>Start fehlgeschlagen</h1>
    <p>{initError}</p>
    {#if !isTauri}<p class="footnote">Diese Oberfläche läuft nur innerhalb der Tauri-App (<code>npm run tauri dev</code>).</p>{/if}
  </div>
{:else if st && ready}
  <div class="shell" in:fade={{ duration: dur('slow'), easing: EASE.enter }}>
    <Toolbar onlost={() => store.toast('Klicke in der Vorschau auf das Wort, an dem du gerade bist (oder R)')} />

    <main class="stage" aria-label={store.live ? 'Live-Vorschau' : 'Editor'}>
      <div class="card" class:live={store.live}>
        <div class="card-head">
          <span class="title" title={title}>{title}</span>
          <div class="controls">
            <div class="seg" role="group" aria-label="Scroll-Modus">
              {#each modes as m (m.id)}
                <button aria-pressed={scrollMode === m.id || (m.id === 'constant' && scrollMode === 'voiceActivated')} onclick={() => setMode(m.id)}>{m.label}</button>
              {/each}
            </div>
            <div class="seg" role="group" aria-label="Modus">
              <button aria-pressed={!store.live} disabled={store.running} onclick={() => store.live && toggleMode()} title="Edit ({displayChord('Ctrl+E')})">Edit</button>
              <button aria-pressed={store.live} onclick={() => !store.live && toggleMode()} title="Live ({displayChord('Ctrl+E')})">Live</button>
            </div>
          </div>
        </div>

        {#if store.live && !store.talentVisible}
          <div class="banner" transition:fly={{ y: -6, duration: dur('base'), easing: EASE.enter }}>
            <span>Ausgabe nicht verbunden – Monitor wählen</span>
            <button class="btn outline" onclick={toggleTalent}>Talent-Fenster öffnen</button>
          </div>
        {/if}
        {#if store.live && lostLong}
          <div class="banner lost" role="alert" transition:fly={{ y: -6, duration: dur('base'), easing: EASE.enter }}>
            <span>Voice hat die Stelle verloren – klicke auf das Wort, an dem du bist, oder drücke R.</span>
          </div>
        {/if}

        <div class="card-body">
          {#key store.live}
            <div class="view" in:fade={{ duration: dur('base'), easing: EASE.enter }}>
              {#if store.live}
                <div class="preview" bind:clientWidth={previewW}>
                  <Prompter
                    app={st}
                    prefs={store.prefs}
                    scale={previewScale}
                    interactive
                    onword={(t) => store.send({ type: 'resyncHere', value: t })}
                    onfocusdrag={setFocusLine}
                  />
                </div>
              {:else}
                <Editor onsaved={refreshScripts} />
              {/if}
            </div>
          {/key}
        </div>
      </div>
    </main>

    <Transport />
  </div>

  {#if store.sidebarOpen}
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="drawer-scrim" onclick={() => (store.sidebarOpen = false)} transition:fade={{ duration: dur('base'), easing: EASE.standard }}></div>
    <div
      class="drawer"
      in:fly={{ x: -24, opacity: 0, duration: dur('panel'), easing: EASE.emphasized }}
      out:fly={{ x: -16, opacity: 0, duration: dur('fast'), easing: EASE.exit }}
    >
      <Sidebar {scripts} onrefresh={refreshScripts} onclose={() => (store.sidebarOpen = false)} />
    </div>
  {/if}

  <Dialog open={store.settingsOpen} label="Einstellungen" width="840px" height="640px" onclose={() => (store.settingsOpen = false)}>
    <Settings onclose={() => (store.settingsOpen = false)} />
  </Dialog>
{:else}
  <div class="loading" aria-busy="true">zero-teleprompter</div>
{/if}

{#if dropping}
  <div class="dropzone" aria-hidden="true" transition:fade={{ duration: dur('fast') }}><div>Skript hier ablegen (.txt, .md)</div></div>
{/if}

<CommandPalette />
<ShortcutOverlay />
<Onboarding />
<Toasts />

<style>
  .shell { display: grid; grid-template-rows: auto minmax(0, 1fr) auto; height: 100vh; background: var(--ui-bg); }
  .stage { min-height: 0; display: flex; padding: 0 24px; }
  .card {
    flex: 1; min-width: 0; min-height: 0;
    display: flex; flex-direction: column;
    width: 100%; max-width: 1200px; margin: 0 auto;
    border-radius: var(--radius-card);
    background: linear-gradient(180deg, var(--ui-card-from) 0%, var(--ui-card-to) 100%);
    border: 1px solid var(--ui-border);
    box-shadow: var(--elev-2);
    overflow: hidden;
    transition: box-shadow var(--dur-slow) var(--ease-standard);
  }
  .card.live { box-shadow: var(--elev-3); }
  .card-head {
    display: flex; align-items: center; justify-content: space-between; gap: 12px;
    padding: 14px 16px 10px 24px;
  }
  .title { font: 600 15px/22px var(--font-ui); letter-spacing: -0.01em; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; min-width: 0; }
  .controls { display: flex; align-items: center; gap: 8px; flex: none; }
  .seg { border-radius: var(--radius-pill); }
  .seg button { border-radius: var(--radius-pill); }
  .seg button:disabled { opacity: .4; cursor: not-allowed; }
  .card-body { flex: 1; min-height: 0; display: flex; position: relative; }
  .view { flex: 1; min-width: 0; min-height: 0; display: flex; flex-direction: column; }
  .preview { flex: 1; min-height: 0; margin: 0 16px 16px; border-radius: var(--radius-lg); overflow: hidden; box-shadow: var(--elev-1); }
  .banner {
    display: flex; align-items: center; justify-content: space-between; gap: 12px;
    margin: 0 16px 12px; padding: 8px 12px; border-radius: var(--radius-md);
    background: var(--ui-surface-raised); border: 1px solid var(--ui-border); font-weight: 500;
  }
  .banner.lost { border-color: var(--state-lost); color: var(--ui-text); }
  .drawer-scrim { position: fixed; inset: 0; z-index: 40; background: var(--ui-overlay); }
  .drawer {
    position: fixed; z-index: 41; left: 12px; top: 12px; bottom: 12px;
    display: flex; min-height: 0;
    border-radius: var(--radius-xl); overflow: hidden;
    box-shadow: var(--elev-4); border: 1px solid var(--ui-border);
  }
  .skip { position: absolute; left: -9999px; top: 8px; z-index: 100; }
  .skip:focus { left: 8px; background: var(--ui-surface-raised); padding: 6px 10px; border-radius: var(--radius-sm); }
  .dropzone {
    position: fixed; inset: 0; z-index: 80; display: grid; place-items: center;
    background: color-mix(in srgb, var(--ui-accent) 10%, transparent); pointer-events: none;
  }
  .dropzone div {
    padding: 24px 32px; border: 2px dashed var(--ui-border-strong); border-radius: var(--radius-card);
    background: var(--ui-surface-raised); font: 600 17px/24px var(--font-ui); box-shadow: var(--elev-3);
  }
  .loading { height: 100vh; display: grid; place-items: center; color: var(--ui-text-muted); font: 600 17px/24px var(--font-ui); }
  .fatal { padding: 48px; max-width: 640px; }
  @media (max-width: 860px) {
    .stage { padding: 0 12px; }
    .card-head { flex-wrap: wrap; padding-left: 16px; }
  }
</style>
