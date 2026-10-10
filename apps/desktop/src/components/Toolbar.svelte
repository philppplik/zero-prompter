<!--
  Top bar (DESIGN §3.2, minimal layout): app logo left; right a pill-shaped
  action control (command palette · output · settings), the voice status, a
  "Live beenden" button in Live and the custom window controls. The whole bar
  is a drag region (double-click = maximise). The scripts drawer toggle lives
  in the left rail (Ctrl+B).
-->
<script lang="ts">
  import Icon from './Icon.svelte';
  import VoiceIndicator from './VoiceIndicator.svelte';
  import logo from '../../app-icon.png';
  import { store } from '../lib/store.svelte';
  import { toggleTalent } from '../lib/actions';
  import WindowControls from './WindowControls.svelte';
  import { platformOf } from '../lib/about';
  import { displayChord } from '../lib/hotkeys';

  interface Props {
    onlost?: () => void;
  }
  let { onlost }: Props = $props();

  const st = $derived(store.state);

  // macOS keeps native traffic lights over the (overlay) title bar → leave room.
  const mac = platformOf(typeof navigator === 'undefined' ? '' : navigator.userAgent) === 'macos';

  async function exitLive() {
    if (store.running) await store.send({ type: 'stop' });
    store.send({ type: 'enterEdit' });
  }
</script>

<header class="top" class:mac data-tauri-drag-region>
  <div class="brand" title="zero-teleprompter" data-tauri-drag-region>
    <img src={logo} alt="zero-teleprompter" width="28" height="28" draggable="false" data-tauri-drag-region />
  </div>

  <div class="right" data-tauri-drag-region>
    <VoiceIndicator {onlost} />
    <div class="pill" role="toolbar" aria-label="Aktionen">
      <button class="seg-btn cmd" onclick={() => (store.paletteOpen = true)} aria-label="Befehle suchen ({displayChord('Ctrl+K')})">
        <Icon name="search" /><span class="ph">Befehle</span><span class="kbd">{displayChord('Ctrl+K')}</span>
      </button>
      <button
        class="seg-btn icon"
        aria-pressed={store.talentVisible}
        aria-label={store.talentVisible ? `Talent-Ausgabe aktiv (${st?.output.fullscreen ? 'Vollbild' : 'Fenster'}) – schließen` : 'Talent-Ausgabe öffnen'}
        title="Talent-Ausgabe (F = Vollbild)"
        onclick={toggleTalent}><Icon name="monitor" />{#if store.talentVisible}<span class="dot" aria-hidden="true"></span>{/if}</button>
      <button
        class="seg-btn icon"
        aria-label="Einstellungen ({displayChord('Ctrl+,')})"
        title="Einstellungen ({displayChord('Ctrl+,')})"
        aria-haspopup="dialog"
        onclick={() => (store.settingsOpen = true)}><Icon name="settings" /></button>
    </div>
    {#if store.live}
      <button class="close" aria-label="Live beenden" title="Live beenden" onclick={exitLive}><Icon name="exit" /></button>
    {/if}
    <WindowControls />
  </div>
</header>

<style>
  .top {
    height: 64px;
    display: flex; align-items: center; justify-content: space-between; gap: 12px;
    padding: 0 12px 0 20px;
    background: transparent;
    user-select: none;
  }
  .top.mac { padding-left: 84px; }
  .brand { display: flex; align-items: center; }
  .brand img { width: 28px; height: 28px; border-radius: 8px; box-shadow: var(--elev-1); user-select: none; }
  .right { display: flex; align-items: center; gap: 10px; min-width: 0; }
  .pill {
    display: inline-flex; align-items: center; gap: 2px; height: 40px; padding: 4px;
    border-radius: var(--radius-pill);
    background: var(--ui-surface-raised); border: 1px solid var(--ui-border);
    box-shadow: var(--elev-1);
  }
  .seg-btn {
    appearance: none; position: relative;
    display: inline-flex; align-items: center; justify-content: center; gap: 8px;
    height: 32px; min-width: 32px; padding: 0 12px; border: 0; border-radius: var(--radius-pill);
    background: transparent; color: var(--ui-text-muted);
    font: 500 13px/20px var(--font-ui); cursor: pointer;
    transition: background var(--dur-fast) var(--ease-standard), color var(--dur-fast) var(--ease-standard), transform var(--dur-fast) var(--ease-standard);
  }
  .seg-btn.icon { padding: 0; width: 32px; }
  .seg-btn:hover { background: var(--ui-surface-sunken); color: var(--ui-text); }
  .seg-btn:active { transform: scale(.94); }
  .seg-btn[aria-pressed='true'] { background: var(--ui-accent-subtle); color: var(--ui-text); }
  .seg-btn:focus-visible { border-radius: var(--radius-pill); }
  .cmd { padding: 0 8px 0 12px; }
  .cmd .kbd { background: transparent; }
  .dot { position: absolute; top: 6px; right: 6px; width: 6px; height: 6px; border-radius: 50%; background: var(--state-tracking); box-shadow: 0 0 0 2px var(--ui-surface-raised); }
  .close {
    appearance: none; display: grid; place-items: center;
    width: 40px; height: 40px; border-radius: 50%;
    border: 1px solid var(--ui-border); background: var(--ui-surface-raised); color: var(--ui-text-muted);
    box-shadow: var(--elev-1); cursor: pointer;
    transition: background var(--dur-fast) var(--ease-standard), color var(--dur-fast) var(--ease-standard), transform var(--dur-fast) var(--ease-standard);
  }
  .close:hover { color: var(--ui-text); background: var(--ui-surface-sunken); }
  .close:active { transform: scale(.94); }
  .close:focus-visible { border-radius: 50%; }
  @media (max-width: 760px) { .ph { display: none; } }
</style>
