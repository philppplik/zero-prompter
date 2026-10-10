<!--
  Control window (CONCEPT §4.1, DESIGN §3.1/§4.1):
  Toolbar · Sidebar | Main (Edit: editor, Live: 1:1 talent preview) | Inspector · Transport.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import Toolbar from './Toolbar.svelte';
  import Sidebar from './Sidebar.svelte';
  import Editor from './Editor.svelte';
  import Inspector from './Inspector.svelte';
  import Transport from './Transport.svelte';
  import Prompter from './Prompter.svelte';
  import CommandPalette from './CommandPalette.svelte';
  import ShortcutOverlay from './ShortcutOverlay.svelte';
  import Onboarding from './Onboarding.svelte';
  import Toasts from './Toasts.svelte';
  import { store } from '../lib/store.svelte';
  import { api, errorText, isTauri } from '../lib/bridge';
  import { importPath, newScriptId, runHotkeyAction, toggleTalent } from '../lib/actions';
  import { bindingMap, chordOf, EXTRA_BINDINGS, isTypingTarget } from '../lib/hotkeys';
  import { setUiTheme } from '../lib/tokens';
  import { EXAMPLE_SCRIPT } from '../lib/example';
  import type { ScriptMeta } from '../lib/types';

  let scripts = $state<ScriptMeta[]>([]);
  let width = $state(1280);
  let previewW = $state(800);
  let inspectorOpen = $state(true);
  let dropping = $state(false);
  let lostSince = $state<number | null>(null);
  let now = $state(Date.now());
  let ready = $state(false);
  let initError = $state<string | null>(null);

  const st = $derived(store.state);
  const bindings = $derived(bindingMap(st?.settings.hotkeys ?? {}, EXTRA_BINDINGS));
  const title = $derived(scripts.find((s) => s.id === store.scriptId)?.title ?? (st?.script.tokens.length ? 'Unbenannt' : 'Kein Skript'));
  const narrow = $derived(width < 1180);
  const inspectorSheet = $derived(width < 1000);
  // Preview = talent geometry scaled down (reference: 1920 px wide output).
  const previewScale = $derived(Math.max(0.2, Math.min(1, previewW / 1920)));
  const lostLong = $derived(lostSince !== null && now - lostSince > 4000);

  $effect(() => setUiTheme(store.prefs.uiTheme));
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
  <div class="shell">
    <Toolbar {title} onlost={() => store.toast('Klicke in der Vorschau auf das Wort, an dem du gerade bist (oder R)')} ontoggleinspector={() => (inspectorOpen = !inspectorOpen)} />
    <div class="body">
      {#if !narrow}<Sidebar {scripts} onrefresh={refreshScripts} />{/if}
      <main class="main" aria-label={store.live ? 'Live-Vorschau' : 'Editor'}>
        {#if store.live}
          {#if !store.talentVisible}
            <div class="banner">
              <span>Ausgabe nicht verbunden – Monitor wählen</span>
              <button class="btn outline" onclick={toggleTalent}>Talent-Fenster öffnen</button>
            </div>
          {/if}
          {#if lostLong}
            <div class="banner lost" role="alert">
              <span>Voice hat die Stelle verloren – klicke auf das Wort, an dem du bist, oder drücke R.</span>
            </div>
          {/if}
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
      </main>
      {#if inspectorOpen}
        <div class:sheet={inspectorSheet}><Inspector /></div>
      {/if}
    </div>
    <Transport />
  </div>
{:else}
  <div class="loading" aria-busy="true">zero-teleprompter</div>
{/if}

{#if dropping}
  <div class="dropzone" aria-hidden="true"><div>Skript hier ablegen (.txt, .md)</div></div>
{/if}

<CommandPalette />
<ShortcutOverlay />
<Onboarding />
<Toasts />

<style>
  .shell { display: grid; grid-template-rows: auto 1fr auto; height: 100vh; }
  .body { display: flex; min-height: 0; position: relative; }
  .main { flex: 1; min-width: 0; display: flex; flex-direction: column; position: relative; background: var(--ui-bg); }
  .preview { flex: 1; min-height: 0; margin: 16px; border-radius: var(--radius-lg); overflow: hidden; box-shadow: var(--elev-2); }
  .banner {
    display: flex; align-items: center; justify-content: space-between; gap: 12px;
    margin: 12px 16px 0; padding: 8px 12px; border-radius: var(--radius-md);
    background: var(--ui-surface-raised); border: 1px solid var(--ui-border); font-weight: 500;
  }
  .banner.lost { border-color: var(--state-lost); color: var(--ui-text); }
  .sheet { position: absolute; right: 0; top: 0; bottom: 0; z-index: 20; box-shadow: var(--elev-3); display: flex; }
  .skip { position: absolute; left: -9999px; top: 8px; z-index: 100; }
  .skip:focus { left: 8px; background: var(--ui-surface-raised); padding: 6px 10px; border-radius: var(--radius-sm); }
  .dropzone {
    position: fixed; inset: 0; z-index: 80; display: grid; place-items: center;
    background: color-mix(in srgb, var(--ui-accent) 12%, transparent); pointer-events: none;
  }
  .dropzone div {
    padding: 24px 32px; border: 2px dashed var(--ui-accent); border-radius: var(--radius-xl);
    background: var(--ui-surface-raised); font: 600 17px/24px var(--font-ui);
  }
  .loading { height: 100vh; display: grid; place-items: center; color: var(--ui-text-muted); font: 600 17px/24px var(--font-ui); }
  .fatal { padding: 48px; max-width: 640px; }
</style>
