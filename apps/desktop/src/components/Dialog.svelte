<!--
  Modal dialog primitive (DESIGN §3.6): scrim fade + panel scale/fade, Escape
  and click-outside close, focus trap, focus restore, aria-modal. Owns the
  keyboard while open (global hotkeys do not fire underneath).
-->
<script lang="ts">
  import { tick, type Snippet } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { dur, EASE } from '../lib/motion';

  interface Props {
    open: boolean;
    label: string;
    onclose: () => void;
    width?: string;
    height?: string;
    placement?: 'center' | 'top';
    children: Snippet;
  }
  let { open, label, onclose, width = '560px', height, placement = 'center', children }: Props = $props();

  let panel = $state<HTMLDivElement | null>(null);
  let restoreTo: HTMLElement | null = null;

  const FOCUSABLE = 'a[href],button:not([disabled]),input:not([disabled]),select:not([disabled]),textarea:not([disabled]),[tabindex]:not([tabindex="-1"])';

  $effect(() => {
    if (open) {
      restoreTo = document.activeElement instanceof HTMLElement ? document.activeElement : null;
      tick().then(() => {
        if (!panel || panel.contains(document.activeElement)) return;
        const auto = panel.querySelector<HTMLElement>('[autofocus],[data-autofocus]');
        (auto ?? panel.querySelector<HTMLElement>(FOCUSABLE) ?? panel).focus();
      });
    } else if (restoreTo) {
      const el = restoreTo;
      restoreTo = null;
      tick().then(() => el.isConnected && el.focus());
    }
  });

  function onKey(e: KeyboardEvent) {
    e.stopPropagation();
    if (e.key === 'Escape') {
      if (e.defaultPrevented) return;
      e.preventDefault();
      onclose();
      return;
    }
    if (e.key !== 'Tab' || !panel) return;
    const items = [...panel.querySelectorAll<HTMLElement>(FOCUSABLE)].filter((el) => el.offsetParent !== null);
    if (items.length === 0) {
      e.preventDefault();
      return;
    }
    const first = items[0];
    const last = items[items.length - 1];
    if (e.shiftKey && document.activeElement === first) {
      last.focus();
      e.preventDefault();
    } else if (!e.shiftKey && document.activeElement === last) {
      first.focus();
      e.preventDefault();
    }
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="scrim" onclick={onclose} transition:fade={{ duration: dur('base'), easing: EASE.standard }}></div>
  <div class="layer" class:top={placement === 'top'}>
    <div
      bind:this={panel}
      class="panel"
      role="dialog"
      aria-modal="true"
      aria-label={label}
      tabindex="-1"
      style:width="min({width}, calc(100vw - 32px))"
      style:height={height ? `min(${height}, calc(100vh - 48px))` : undefined}
      onkeydown={onKey}
      in:scale={{ start: 0.96, opacity: 0, duration: dur('panel'), easing: EASE.emphasized }}
      out:scale={{ start: 0.98, opacity: 0, duration: dur('fast'), easing: EASE.exit }}
    >
      {@render children()}
    </div>
  </div>
{/if}

<style>
  .scrim {
    position: fixed; inset: 0; z-index: 50;
    background: var(--ui-overlay);
    backdrop-filter: blur(6px); -webkit-backdrop-filter: blur(6px);
  }
  .layer {
    position: fixed; inset: 0; z-index: 51; pointer-events: none;
    display: grid; place-items: center; padding: 24px 16px;
  }
  .layer.top { place-items: start center; padding-top: 18vh; }
  .panel {
    pointer-events: auto;
    max-height: calc(100vh - 48px);
    display: flex; flex-direction: column; min-height: 0;
    background: var(--ui-surface-raised); color: var(--ui-text);
    border: 1px solid var(--ui-border); border-radius: var(--radius-xl);
    box-shadow: var(--elev-4); overflow: hidden;
    transform-origin: 50% 40%;
  }
  .panel:focus { outline: none; }
</style>
