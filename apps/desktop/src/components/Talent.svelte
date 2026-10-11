<!--
  Route /talent (talent.html): render-only output window (CONCEPT §4.1).
  No own logic beyond: keyboard passthrough to the same actions as the control
  window, cursor auto-hide after 1.5 s, Esc leaves fullscreen (no keyboard trap).
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import Prompter from './Prompter.svelte';
  import { store } from '../lib/store.svelte';
  import { bindingMap, chordOf, EXTRA_BINDINGS } from '../lib/hotkeys';
  import { runHotkeyAction } from '../lib/actions';

  let idle = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;

  const bindings = $derived(bindingMap(store.state?.settings.hotkeys ?? {}, EXTRA_BINDINGS));

  function poke() {
    idle = false;
    clearTimeout(timer);
    timer = setTimeout(() => (idle = true), 1500);
  }

  function onKey(e: KeyboardEvent) {
    const action = bindings.get(chordOf(e));
    // Talent window: no palette / file dialogs, only playback & output actions.
    if (!action || ['commandPalette', 'openFile', 'toggleMode', 'shortcutOverlay'].includes(action)) return;
    if (runHotkeyAction(action)) e.preventDefault();
  }

  onMount(() => {
    poke();
    return () => clearTimeout(timer);
  });
</script>

<svelte:window onkeydown={onKey} onmousemove={poke} />

<main class:idle>
  {#if store.state}
    <Prompter app={store.state} prefs={store.prefs} mirror report wheel />
  {/if}
</main>

<style>
  main {
    position: fixed;
    inset: 0;
    background: #0e0f12;
  }
  main.idle {
    cursor: none;
  }
</style>
