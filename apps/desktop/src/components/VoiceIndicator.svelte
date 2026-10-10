<!--
  Voice-tracking status pill (DESIGN §3.8). State is shown by colour AND shape
  AND label (WCAG 1.4.1). Operator only — the talent never sees this.
-->
<script lang="ts">
  import { store } from '../lib/store.svelte';
  import { STATE_STYLE } from '../lib/tokens';
  import type { VoicePhase } from '../lib/types';

  interface Props {
    onlost?: () => void;
  }
  let { onlost }: Props = $props();

  const voice = $derived(store.state?.voice);
  const phase = $derived<VoicePhase>(voice?.enabled ? voice.phase : 'idle');
  const style = $derived(STATE_STYLE[phase]);
  const wpm = $derived(Math.round((voice?.rateWps ?? 0) * 60));
  const label = $derived.by(() => {
    switch (phase) {
      case 'idle': return 'Voice aus';
      case 'listening': return 'Hört zu…';
      case 'tracking': return wpm > 0 ? `Folgt · ${wpm} WPM` : 'Folgt';
      case 'hold': return 'Wartet';
      case 'lost': return 'Verloren – klicke ein Wort';
      case 'resync': return 'Springt…';
    }
  });
  const demo = $derived(store.engine?.demo && voice?.enabled);
</script>

<div class="wrap" role="status" aria-live={phase === 'lost' ? 'assertive' : 'polite'}>
  {#if phase === 'lost'}
    <button class="pill lost" style:--c={style.color} onclick={() => onlost?.()} title="Klicke in der Vorschau auf das Wort, an dem du bist (oder R)">
      <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true"><path d="M6 1 11 10.5H1z" fill="currentColor" /><path d="M6 4.5v3M6 9h.01" stroke="#000" stroke-width="1.4" stroke-linecap="round" /></svg>
      <span>{label}</span>
    </button>
  {:else}
    <span class="pill {phase}" style:--c={style.color}>
      <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
        {#if phase === 'idle'}
          <circle cx="6" cy="6" r="4.5" fill="none" stroke="currentColor" stroke-width="1.5" />
        {:else if phase === 'listening' || phase === 'tracking'}
          <circle cx="6" cy="6" r="5" fill="currentColor" />
        {:else if phase === 'hold'}
          <path d="M4 2.5v7M8 2.5v7" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
        {:else}
          <path d="M2 6a4 4 0 0 1 7.5-2" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
        {/if}
      </svg>
      <span>{label}</span>
      {#if phase === 'tracking' && voice}
        <span class="conf" aria-label="Konfidenz {Math.round(voice.confidence * 100)} Prozent">
          <span style:width="{Math.round(voice.confidence * 100)}%"></span>
        </span>
      {/if}
      {#if demo}<span class="demo" title="Kein Sprachmodell installiert – Demo-Modus liest das Skript automatisch">Demo</span>{/if}
    </span>
  {/if}
</div>

<style>
  .wrap { display: inline-flex; }
  .pill {
    display: inline-flex; align-items: center; gap: 6px;
    height: 24px; padding: 0 10px; border-radius: 999px;
    font: 600 12px/16px var(--font-ui);
    color: var(--ui-text);
    background: color-mix(in srgb, var(--c) 14%, var(--ui-surface));
    border: 1px solid color-mix(in srgb, var(--c) 40%, transparent);
    white-space: nowrap;
  }
  .pill svg { color: var(--c); flex: none; }
  .pill.listening svg { animation: pulse 1.6s var(--ease-standard) infinite; }
  .pill.resync svg { animation: spin 0.8s linear infinite; }
  button.pill { cursor: pointer; animation: shake 0.4s var(--ease-standard) 1; }
  button.pill svg { color: var(--c); }
  .conf { width: 36px; height: 4px; border-radius: 2px; background: var(--ui-border); overflow: hidden; }
  .conf span { display: block; height: 100%; background: var(--c); transition: width var(--dur-base) var(--ease-standard); }
  .demo { font: 500 10px/14px var(--font-ui); padding: 0 4px; border-radius: 4px; background: var(--ui-surface-sunken); color: var(--ui-text-muted); }
  @keyframes pulse { 50% { opacity: 0.35; } }
  @keyframes spin { to { transform: rotate(360deg); } }
  @keyframes shake { 25% { transform: translateX(-3px); } 75% { transform: translateX(3px); } }
</style>
