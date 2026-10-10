<!-- Transport bar (DESIGN §3.3). -->
<script lang="ts">
  import Icon from './Icon.svelte';
  import { store } from '../lib/store.svelte';
  import { home } from '../lib/actions';

  const st = $derived(store.state);
  const total = $derived(st?.script.tokens.length ?? 0);
  const pos = $derived(Math.min(total, Math.max(0, store.position)));
  const kind = $derived(st?.playback.state.kind ?? 'stopped');
  const playing = $derived(kind === 'playing' || kind === 'countdown');
  const wps = $derived.by(() => {
    if (!st) return 2;
    if (st.voice.enabled && st.voice.rateWps > 0.2) return st.voice.rateWps;
    return Math.max(0.1, st.scroll.wpm / 60);
  });
  const fmt = (s: number) => {
    const t = Math.max(0, Math.round(s));
    return `${String(Math.floor(t / 60)).padStart(2, '0')}:${String(t % 60).padStart(2, '0')}`;
  };
  const remaining = $derived(fmt((total - pos) / wps));
  const totalTime = $derived(fmt(total / Math.max(0.1, (st?.scroll.wpm ?? 140) / 60)));
  const pct = $derived(total > 1 ? (pos / (total - 1)) * 100 : 0);

  let wpmInput = $state<number | null>(null);

  function setCountdown() {
    if (!st) return;
    const next = st.settings.countdownSecs === 0 ? 3 : st.settings.countdownSecs === 3 ? 5 : 0;
    store.send({ type: 'replaceSettings', value: { ...$state.snapshot(st.settings), countdownSecs: next } });
  }

  function seekPct(e: MouseEvent) {
    const el = e.currentTarget as HTMLElement;
    const r = el.getBoundingClientRect();
    const f = (e.clientX - r.left) / r.width;
    store.send({ type: 'seekToken', value: Math.round(f * Math.max(0, total - 1)) });
  }

  function progressKey(e: KeyboardEvent) {
    const step = e.key === 'ArrowRight' ? 10 : e.key === 'ArrowLeft' ? -10 : 0;
    if (!step) return;
    e.preventDefault();
    e.stopPropagation();
    store.send({ type: 'seekToken', value: Math.max(0, Math.min(total - 1, store.currentToken + step)) });
  }
</script>

<footer class="transport" id="transport" aria-label="Wiedergabe">
  <div class="group">
    <button class="btn icon t" aria-label="Zum Anfang (Home)" title="Zum Anfang (Home)" onclick={home}><Icon name="home" /></button>
    <button class="btn icon t" aria-label="Absatz zurück (Bild auf)" title="Absatz zurück (PgUp)" onclick={() => store.send({ type: 'jumpParagraph', value: -1 })}><Icon name="paraPrev" /></button>
    <button class="play" aria-label={playing ? 'Pause (Leertaste)' : 'Abspielen (Leertaste)'} onclick={() => store.send({ type: 'togglePlay' })} disabled={total === 0}>
      <Icon name={playing ? 'pause' : 'play'} size={18} />
    </button>
    <button class="btn icon t" aria-label="Absatz vor (Bild ab)" title="Absatz vor (PgDn)" onclick={() => store.send({ type: 'jumpParagraph', value: 1 })}><Icon name="paraNext" /></button>
  </div>

  <div class="group wpm" role="group" aria-label="Tempo">
    <button class="btn icon" aria-label="Langsamer" onclick={() => store.send({ type: 'nudgeWpm', value: -5 })}><Icon name="minus" /></button>
    <input
      type="number"
      min="40"
      max="400"
      step="5"
      aria-label="Wörter pro Minute"
      value={wpmInput ?? st?.scroll.wpm ?? 140}
      oninput={(e) => (wpmInput = Number((e.currentTarget as HTMLInputElement).value))}
      onchange={() => {
        if (wpmInput) store.send({ type: 'setWpm', value: wpmInput });
        wpmInput = null;
      }}
    />
    <button class="btn icon" aria-label="Schneller" onclick={() => store.send({ type: 'nudgeWpm', value: 5 })}><Icon name="plus" /></button>
    <span class="caption">WPM</span>
  </div>

  <div class="progress-wrap">
    <div
      class="progress"
      role="slider"
      tabindex="0"
      aria-label="Position im Skript"
      aria-valuemin="0"
      aria-valuemax={Math.max(0, total - 1)}
      aria-valuenow={Math.floor(pos)}
      aria-valuetext="Wort {Math.floor(pos) + (total ? 1 : 0)} von {total}"
      onclick={seekPct}
      onkeydown={progressKey}
    >
      <div class="bar" style:width="{pct}%"></div>
    </div>
    <span class="num time" title="Restzeit / Gesamtdauer">{remaining} / {totalTime}</span>
    <span class="num words footnote">Wort {Math.min(total, Math.floor(pos) + 1)}/{total}</span>
  </div>

  <button class="btn countdown" title="Countdown vor dem Start" aria-label="Countdown {st?.settings.countdownSecs ?? 3} Sekunden, umschalten" onclick={setCountdown}>
    <Icon name="timer" /><span class="num">{st?.settings.countdownSecs ?? 3}s</span>
  </button>
</footer>

<style>
  .transport {
    height: var(--layout-transport);
    display: flex; align-items: center; gap: 20px;
    width: 100%; max-width: 1200px; margin: 0 auto;
    padding: 0 24px;
    background: transparent;
  }
  .group { display: flex; align-items: center; gap: 4px; }
  .t { width: 32px; height: 32px; }
  .play {
    width: 40px; height: 40px; border-radius: 50%;
    display: grid; place-items: center; border: 0; cursor: pointer;
    background: var(--ui-accent); color: var(--ui-text-on-accent);
    transition: background var(--dur-fast) var(--ease-standard), transform var(--dur-fast) var(--ease-standard);
  }
  .play:hover { background: var(--ui-accent-hover); }
  .play:hover:not(:disabled) { transform: scale(1.04); box-shadow: var(--elev-2); }
  .play:active:not(:disabled) { transform: scale(.92); }
  .play:disabled { opacity: .4; cursor: default; }
  .wpm input { text-align: center; }
  .progress-wrap { flex: 1; display: flex; align-items: center; gap: 12px; min-width: 0; }
  .progress { flex: 1; height: 6px; border-radius: 3px; background: var(--ui-surface-sunken); border: 1px solid var(--ui-border); cursor: pointer; position: relative; }
  .progress::after { content: ''; position: absolute; inset: -9px 0; }
  .bar { height: 100%; border-radius: 3px; background: var(--ui-accent); transition: width var(--dur-fast) linear; }
  .time { font-size: 13px; }
  .words { white-space: nowrap; }
  @media (max-width: 1100px) { .words { display: none; } }
</style>
