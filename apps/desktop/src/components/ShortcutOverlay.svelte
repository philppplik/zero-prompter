<!-- "?" overlay: all shortcuts incl. global (system-wide) ones (DESIGN §5). -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { store } from '../lib/store.svelte';
  import { api } from '../lib/bridge';
  import { displayChord, EXTRA_BINDINGS } from '../lib/hotkeys';

  let globals = $state<[string, string][]>([]);
  onMount(() => {
    api.buildInfo().then((b) => (globals = b.globalShortcuts)).catch(() => {});
  });

  const LABELS: Record<string, string> = {
    togglePlay: 'Play/Pause', stop: 'Zum Anfang', faster: 'Tempo +5 WPM', slower: 'Tempo −5 WPM',
    nextParagraph: 'Absatz vor', prevParagraph: 'Absatz zurück', toggleMirror: 'Spiegeln H', toggleMirrorV: 'Spiegeln V',
    toggleFullscreen: 'Vollbild Talent', toggleVoice: 'Voice an/aus', resyncHere: 'Resync am Marker',
    commandPalette: 'Command Palette', shortcutOverlay: 'Diese Übersicht', toggleMode: 'Edit/Live',
    openFile: 'Öffnen', fontBigger: 'Schrift größer', fontSmaller: 'Schrift kleiner', theme1: 'Theme 1–6', escape: 'Abbrechen',
  };
  const rows = $derived.by(() => {
    const hk = store.state?.settings.hotkeys ?? {};
    const extra = Object.entries(EXTRA_BINDINGS).filter(([k]) => !/^theme[2-6]$|homeMac/.test(k));
    return [...Object.entries(hk), ...extra].map(([a, c]) => ({ label: LABELS[a] ?? a, chord: a === 'theme1' ? `${displayChord('Ctrl+1')}…6` : displayChord(c) }));
  });
</script>

{#if store.shortcutsOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="scrim" onclick={() => (store.shortcutsOpen = false)}></div>
  <div class="sheet" role="dialog" aria-modal="true" aria-label="Tastenkürzel">
    <h2>Tastenkürzel</h2>
    <div class="grid">
      {#each rows as r (r.label)}
        <span>{r.label}</span><span class="kbd">{r.chord}</span>
      {/each}
    </div>
    {#if globals.length}
      <h3 class="caption">Global (auch wenn eine andere App aktiv ist)</h3>
      <div class="grid">
        {#each globals as [chord, cmd] (chord)}<span>{cmd}</span><span class="kbd">{chord}</span>{/each}
      </div>
    {/if}
    <p class="footnote">Klick auf ein Wort in der Vorschau = Resync · Mausrad = manuell (Voice pausiert 2 s) · Marker per Drag verschieben</p>
    <button class="btn primary" onclick={() => (store.shortcutsOpen = false)}>Schließen</button>
  </div>
{/if}

<style>
  .scrim { position: fixed; inset: 0; background: var(--ui-overlay); z-index: 50; }
  .sheet {
    position: fixed; z-index: 51; left: 50%; top: 50%; transform: translate(-50%, -50%);
    width: min(560px, calc(100vw - 32px)); max-height: 80vh; overflow: auto; padding: 24px;
    background: var(--ui-surface-raised); border-radius: var(--radius-xl); box-shadow: var(--elev-4);
  }
  h2 { font: 600 22px/28px var(--font-ui); margin: 0 0 16px; }
  h3 { margin: 20px 0 8px; }
  .grid { display: grid; grid-template-columns: 1fr auto; gap: 6px 16px; align-items: center; }
  .grid .kbd { justify-self: end; }
  p { margin: 16px 0; }
</style>
