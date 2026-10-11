<!-- "?" overlay: all shortcuts incl. global (system-wide) ones (DESIGN §5). -->
<script lang="ts">
  import { onMount } from 'svelte';
  import Dialog from './Dialog.svelte';
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
    openFile: 'Öffnen', toggleSidebar: 'Skripte ein/aus', openSettings: 'Einstellungen', fontBigger: 'Schrift größer', fontSmaller: 'Schrift kleiner', theme1: 'Theme 1–6', escape: 'Abbrechen',
  };
  const mod = displayChord('Ctrl');
  const WHEEL_ROWS: [string, string][] = [
    ['Konstant: Tempo ±5 WPM', 'Mausrad'],
    ['Konstant: Tempo ±1 WPM', `${displayChord('Shift')} + Mausrad`],
    ['Konstant: Position', `${mod}/Alt + Mausrad`],
    ['Manuell: Position', 'Mausrad'],
    ['Manuell: Tempo ±5 (±1 mit Shift)', `${mod}/Alt + Mausrad`],
    ['Voice: kurz manuell (Voice pausiert 2 s)', 'Mausrad'],
  ];
  const rows = $derived.by(() => {
    const hk = store.state?.settings.hotkeys ?? {};
    const extra = Object.entries(EXTRA_BINDINGS).filter(([k]) => !/^theme[2-6]$|homeMac/.test(k));
    return [...Object.entries(hk), ...extra].map(([a, c]) => ({ label: LABELS[a] ?? a, chord: a === 'theme1' ? `${displayChord('Ctrl+1')}…6` : displayChord(c) }));
  });
</script>

<Dialog open={store.shortcutsOpen} label="Tastenkürzel" onclose={() => (store.shortcutsOpen = false)}>
  <div class="sheet">
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
    <h3 class="caption">Maus über dem Prompter</h3>
    <div class="grid">
      {#each WHEEL_ROWS as [what, how] (what)}<span>{what}</span><span class="kbd">{how}</span>{/each}
    </div>
    <p class="footnote">Klick auf ein Wort in der Vorschau = Resync · Marker per Drag verschieben</p>
    <button class="btn primary" onclick={() => (store.shortcutsOpen = false)}>Schließen</button>
  </div>
</Dialog>

<style>
  .sheet { min-height: 0; overflow-y: auto; padding: 24px; }
  h2 { font: 600 22px/28px var(--font-ui); margin: 0 0 16px; }
  h3 { margin: 20px 0 8px; }
  .grid { display: grid; grid-template-columns: 1fr auto; gap: 6px 16px; align-items: center; }
  .grid .kbd { justify-self: end; }
  p { margin: 16px 0; }
</style>
