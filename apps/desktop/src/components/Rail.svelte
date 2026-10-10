<!--
  Narrow left rail: toggles the scripts drawer (Ctrl+B). The icon morphs
  between "open" and "close" and an accent bar marks the open state.
-->
<script lang="ts">
  import Icon from './Icon.svelte';
  import { store } from '../lib/store.svelte';
  import { displayChord } from '../lib/hotkeys';

  const open = $derived(store.sidebarOpen);
  const label = $derived(`${open ? 'Skripte ausblenden' : 'Skripte & Gliederung einblenden'} (${displayChord('Ctrl+B')})`);
</script>

<nav class="rail" aria-label="Seitenleiste">
  <button
    class="toggle"
    class:open
    aria-label={label}
    title={label}
    aria-expanded={open}
    aria-controls="scripts-drawer"
    onclick={() => (store.sidebarOpen = !store.sidebarOpen)}
  >
    <span class="ico a"><Icon name="panelOpen" /></span>
    <span class="ico b"><Icon name="panelClose" /></span>
  </button>
</nav>

<style>
  .rail {
    position: relative; z-index: 42;
    width: 52px; display: flex; flex-direction: column; align-items: center; gap: 8px;
    padding: 4px 0 0 12px;
  }
  .toggle {
    appearance: none; position: relative; display: grid; place-items: center;
    width: 40px; height: 40px; border-radius: var(--radius-pill);
    border: 1px solid var(--ui-border); background: var(--ui-surface-raised); color: var(--ui-text-muted);
    box-shadow: var(--elev-1); cursor: pointer;
    transition: background var(--dur-fast) var(--ease-standard), color var(--dur-fast) var(--ease-standard),
      transform var(--dur-fast) var(--ease-standard), box-shadow var(--dur-base) var(--ease-standard);
  }
  .toggle:hover { color: var(--ui-text); background: var(--ui-surface-sunken); box-shadow: var(--elev-2); }
  .toggle:active { transform: scale(.94); }
  .toggle:focus-visible { border-radius: var(--radius-pill); }
  .toggle.open { background: var(--ui-accent-subtle); color: var(--ui-text); }
  .toggle::before {
    content: ''; position: absolute; left: -12px; top: 50%; width: 3px; height: 0; border-radius: 0 2px 2px 0;
    background: var(--ui-accent); transform: translateY(-50%);
    transition: height var(--dur-base) var(--ease-emphasized, var(--ease-standard));
  }
  .toggle.open::before { height: 20px; }
  .ico { grid-area: 1 / 1; display: grid; transition: opacity var(--dur-base) var(--ease-standard), transform var(--dur-base) var(--ease-standard); }
  .ico.b { opacity: 0; transform: translateX(-4px) scale(.85); }
  .open .ico.a { opacity: 0; transform: translateX(4px) scale(.85); }
  .open .ico.b { opacity: 1; transform: none; }
  @media (prefers-reduced-motion: reduce) { .ico, .toggle::before { transition: none; } }
</style>
