<script lang="ts">
  import { flip } from 'svelte/animate';
  import { fade, fly } from 'svelte/transition';
  import { store } from '../lib/store.svelte';
  import { dur, EASE } from '../lib/motion';
</script>

<div class="toasts" aria-live="polite">
  {#each store.toasts as t (t.id)}
    <div
      class="toast"
      class:error={t.kind === 'error'}
      role={t.kind === 'error' ? 'alert' : 'status'}
      animate:flip={{ duration: dur('base'), easing: EASE.standard }}
      in:fly={{ y: 12, opacity: 0, duration: dur('panel'), easing: EASE.emphasized }}
      out:fade={{ duration: dur('fast'), easing: EASE.exit }}
    >
      {#if t.kind === 'error'}<span class="dot" aria-hidden="true"></span>{/if}{t.text}
    </div>
  {/each}
</div>

<style>
  .toasts {
    position: fixed; left: 50%; bottom: calc(var(--layout-transport) + 24px); transform: translateX(-50%);
    display: grid; gap: 8px; justify-items: center; z-index: 60; pointer-events: none;
  }
  .toast {
    display: inline-flex; align-items: center; gap: 8px;
    padding: 8px 16px; border-radius: var(--radius-pill);
    background: var(--ui-accent); color: var(--ui-text-on-accent);
    box-shadow: var(--elev-3); font: 500 13px/20px var(--font-ui);
    max-width: min(560px, calc(100vw - 32px));
  }
  .dot { width: 8px; height: 8px; border-radius: 50%; background: var(--state-lost); flex: none; }
</style>
