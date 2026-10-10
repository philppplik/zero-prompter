<!--
  Hotkey list with click-to-record and duplicate detection (Inspector section
  "Tastenkürzel", DESIGN §3.5). Writes through ReplaceSettings.
-->
<script lang="ts">
  import { store } from '../lib/store.svelte';
  import { chordOf, displayChord, normalizeChord } from '../lib/hotkeys';

  const st = $derived(store.state);
  let recording = $state<string | null>(null);

  const HOTKEY_LABELS: Record<string, string> = {
    togglePlay: 'Play/Pause',
    stop: 'Zum Anfang',
    faster: 'Schneller',
    slower: 'Langsamer',
    nextParagraph: 'Absatz vor',
    prevParagraph: 'Absatz zurück',
    toggleMirror: 'Spiegeln',
    toggleFullscreen: 'Vollbild',
    toggleVoice: 'Voice an/aus',
    resyncHere: 'Resync am Marker',
    commandPalette: 'Command Palette',
    shortcutOverlay: 'Kürzel-Übersicht',
  };

  const conflict = $derived.by(() => {
    if (!st) return new Set<string>();
    const seen = new Map<string, string>();
    const dup = new Set<string>();
    for (const [a, c] of Object.entries(st.settings.hotkeys)) {
      const n = normalizeChord(c);
      if (seen.has(n)) {
        dup.add(a);
        dup.add(seen.get(n)!);
      } else seen.set(n, a);
    }
    return dup;
  });

  function recordKey(e: KeyboardEvent) {
    if (!recording || !st) return;
    e.preventDefault();
    e.stopPropagation();
    if (['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) return;
    if (e.key === 'Escape') {
      recording = null;
      return;
    }
    const hotkeys = { ...$state.snapshot(st.settings.hotkeys), [recording]: chordOf(e) };
    store.send({
      type: 'replaceSettings',
      value: { ...$state.snapshot(st.settings), hotkeys },
    });
    recording = null;
  }
</script>

<svelte:window onkeydowncapture={recordKey} />

{#if st}
  {#each Object.entries(st.settings.hotkeys) as [action, chord] (action)}
    <div class="hk">
      <span>{HOTKEY_LABELS[action] ?? action}</span>
      <button
        class="kbd rec"
        class:recording={recording === action}
        class:conflict={conflict.has(action)}
        onclick={() => (recording = recording === action ? null : action)}
        aria-label="{HOTKEY_LABELS[action] ?? action}: {displayChord(chord)}. Klicken zum Ändern"
      >
        {recording === action ? 'Taste drücken…' : displayChord(chord)}
      </button>
    </div>
  {/each}
  {#if conflict.size > 0}<p class="footnote bad">Doppelt belegte Kürzel sind markiert.</p>{/if}
{/if}

<style>
  .hk {
    display: flex;
    justify-content: space-between;
    align-items: center;
    height: 30px;
  }
  .rec {
    cursor: pointer;
    min-width: 64px;
    justify-content: center;
  }
  .rec.recording {
    border-color: var(--ui-accent);
    color: var(--ui-accent);
  }
  .rec.conflict {
    border-color: var(--state-lost);
    color: var(--state-lost);
  }
  .bad {
    color: var(--state-lost);
  }
</style>
