<!--
  Custom window controls for the undecorated control window (Windows/Linux):
  minimise · maximise/restore · close (close = quit app, see lib.rs
  CloseRequested). On macOS the native traffic lights stay (overlay title bar),
  so this renders nothing there.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import Icon from './Icon.svelte';
  import { isTauri } from '../lib/bridge';
  import { customWindowControls, platformOf } from '../lib/about';

  const show = isTauri && customWindowControls(platformOf(navigator.userAgent));
  let maximized = $state(false);

  onMount(() => {
    if (!show) return;
    const win = getCurrentWindow();
    let un: (() => void) | undefined;
    let alive = true;
    const sync = () => win.isMaximized().then((m) => (maximized = m)).catch(() => {});
    sync();
    win.onResized(sync).then((u) => (alive ? (un = u) : u()));
    return () => {
      alive = false;
      un?.();
    };
  });

  const win = () => getCurrentWindow();
</script>

{#if show}
  <div class="wc" role="group" aria-label="Fenster">
    <button class="b" aria-label="Minimieren" title="Minimieren" onclick={() => win().minimize()}><Icon name="winMin" size={14} /></button>
    <button
      class="b"
      aria-label={maximized ? 'Wiederherstellen' : 'Maximieren'}
      title={maximized ? 'Wiederherstellen' : 'Maximieren'}
      onclick={() => win().toggleMaximize()}><Icon name={maximized ? 'winRestore' : 'winMax'} size={14} /></button>
    <button class="b close" aria-label="zero-teleprompter beenden" title="Schließen" onclick={() => win().close()}><Icon name="x" size={14} /></button>
  </div>
{/if}

<style>
  .wc { display: inline-flex; align-items: center; gap: 2px; margin-left: 4px; }
  .b {
    appearance: none; display: grid; place-items: center;
    width: 36px; height: 32px; border: 0; border-radius: var(--radius-sm);
    background: transparent; color: var(--ui-text-muted); cursor: pointer;
    transition: background var(--dur-fast) var(--ease-standard), color var(--dur-fast) var(--ease-standard), transform var(--dur-fast) var(--ease-standard);
  }
  .b:hover { background: var(--ui-surface-sunken); color: var(--ui-text); }
  .b:active { transform: scale(.94); }
  .b.close:hover { background: #c42b1c; color: #fff; }
</style>
