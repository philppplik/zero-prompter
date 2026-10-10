<!-- Toolbar (DESIGN §3.2): mode seam, scroll mode, voice status, output, palette. -->
<script lang="ts">
  import Icon from './Icon.svelte';
  import VoiceIndicator from './VoiceIndicator.svelte';
  import { store } from '../lib/store.svelte';
  import { setMode, toggleMode, toggleTalent } from '../lib/actions';
  import { displayChord } from '../lib/hotkeys';
  import type { ScrollMode } from '../lib/types';

  interface Props {
    title: string;
    onlost?: () => void;
    ontoggleinspector?: () => void;
  }
  let { title, onlost, ontoggleinspector }: Props = $props();

  const st = $derived(store.state);
  const mode = $derived<ScrollMode>(st?.scroll.mode ?? 'constant');
  const modes: { id: ScrollMode; label: string }[] = [
    { id: 'manual', label: 'Manuell' },
    { id: 'constant', label: 'Konstant' },
    { id: 'voiceTracking', label: 'Voice' },
  ];
  const locked = $derived(store.running);
</script>

<header class="toolbar">
  <div class="left">
    <span class="title" title={title}>{title}</span>
    <div class="seg" role="group" aria-label="Modus">
      <button aria-pressed={!store.live} disabled={locked} onclick={() => store.live && toggleMode()} title="Edit ({displayChord('Ctrl+E')})">
        Edit
      </button>
      <button aria-pressed={store.live} onclick={() => !store.live && toggleMode()} title="Live ({displayChord('Ctrl+E')})">
        Live
      </button>
    </div>
  </div>

  <div class="center">
    <div class="seg" role="group" aria-label="Scroll-Modus">
      {#each modes as m (m.id)}
        <button aria-pressed={mode === m.id || (m.id === 'constant' && mode === 'voiceActivated')} onclick={() => setMode(m.id)}>{m.label}</button>
      {/each}
    </div>
  </div>

  <div class="right">
    <VoiceIndicator {onlost} />
    <button
      class="btn icon"
      aria-pressed={!!st?.output.mirrorH}
      aria-label="Talent horizontal spiegeln (M)"
      title="Spiegeln (M, Shift+M vertikal)"
      onclick={() => store.send({ type: 'toggleMirrorH' })}><Icon name="mirror" /></button>
    <button
      class="btn"
      class:active={store.talentVisible}
      aria-pressed={store.talentVisible}
      title="Talent-Fenster (F = Vollbild)"
      onclick={toggleTalent}><Icon name="monitor" />{store.talentVisible ? (st?.output.fullscreen ? 'Vollbild' : 'Fenster') : 'Ausgabe'}</button>
    <button class="btn outline palette" onclick={() => (store.paletteOpen = true)} aria-label="Befehle suchen">
      <Icon name="search" /><span class="ph">Suchen…</span><span class="kbd">{displayChord('Ctrl+K')}</span>
    </button>
    <button class="btn icon" aria-label="Inspector ein-/ausblenden" title="Inspector" onclick={() => ontoggleinspector?.()}><Icon name="settings" /></button>
  </div>
</header>

<style>
  .toolbar {
    height: var(--layout-toolbar);
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    gap: 12px;
    padding: 0 12px;
    border-bottom: 1px solid var(--ui-border);
    background: var(--ui-surface);
  }
  .left, .right { display: flex; align-items: center; gap: 8px; min-width: 0; }
  .right { justify-content: flex-end; }
  .title { font: 600 13px/20px var(--font-ui); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 220px; }
  .palette { color: var(--ui-text-muted); font-weight: 400; }
  .seg button:disabled { opacity: .4; cursor: not-allowed; }
  @media (max-width: 1180px) { .ph { display: none; } }
</style>
