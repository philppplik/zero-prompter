<script lang="ts">
  import { store } from '../lib/store.svelte';
</script>

<div class="toasts" aria-live="polite">
  {#each store.toasts as t (t.id)}
    <div class="toast" class:error={t.kind === 'error'} role={t.kind === 'error' ? 'alert' : 'status'}>{t.text}</div>
  {/each}
</div>

<style>
  .toasts {
    position: fixed; left: 50%; bottom: calc(var(--layout-transport) + 16px); transform: translateX(-50%);
    display: grid; gap: 8px; justify-items: center; z-index: 60; pointer-events: none;
  }
  .toast {
    padding: 8px 14px; border-radius: var(--radius-md);
    background: var(--ui-surface-raised); color: var(--ui-text);
    box-shadow: var(--elev-2); font: 500 13px/20px var(--font-ui);
    animation: in var(--dur-base) var(--ease-enter);
  }
  .toast.error { border-left: 3px solid var(--state-lost); }
  @keyframes in { from { opacity: 0; transform: translateY(6px); } }
</style>
