<!--
  Onboarding ≤ 60 s, 4 steps (DESIGN §3.10): Willkommen · Mikrofon · Sprachmodell · Ausgabe.
  "Überspringen" always visible; finishing starts a rehearsal (countdown).
-->
<script lang="ts">
  import { onDestroy } from 'svelte';
  import Icon from './Icon.svelte';
  import { store } from '../lib/store.svelte';
  import { api, errorText } from '../lib/bridge';
  import type { ModelRow, MonitorInfo } from '../lib/types';

  let step = $state(0);
  let micOn = $state(false);
  let models = $state<ModelRow[]>([]);
  let monitors = $state<MonitorInfo[]>([]);
  let chosen = $state<number | null>(null);
  const steps = ['Willkommen', 'Mikrofon', 'Sprachmodell', 'Ausgabe'];
  const levelPct = $derived(Math.max(0, Math.min(100, ((store.level + 60) / 60) * 100)));

  async function go(n: number) {
    if (micOn && n !== 1) await mic(false);
    step = n;
    if (n === 2) models = await api.modelsList().catch(() => []);
    if (n === 3) monitors = await api.listMonitors().catch(() => []);
  }

  async function mic(on: boolean) {
    micOn = on;
    try {
      await api.micTest(on);
    } catch (e) {
      store.toast(errorText(e), 'error');
    }
  }

  function close() {
    if (micOn) mic(false);
    store.onboardingOpen = false;
    store.setPrefs({ onboarded: true });
  }

  async function finish() {
    const st = store.state;
    if (chosen !== null) await api.openTalent(chosen, false).catch(() => {});
    close();
    if (st && st.script.tokens.length > 0) {
      await store.send({ type: 'setScrollMode', value: 'constant' });
      await store.send({ type: 'play' });
    }
  }

  onDestroy(() => {
    if (micOn) api.micTest(false).catch(() => {});
  });

  function trap(e: KeyboardEvent) {
    e.stopPropagation();
    if (e.key === 'Escape') close();
  }
</script>

{#if store.onboardingOpen}
  <div class="scrim"></div>
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div class="modal" role="dialog" aria-modal="true" aria-labelledby="ob-title" tabindex="-1" onkeydown={trap}>
    <div class="top">
      <div class="dots" aria-label="Schritt {step + 1} von {steps.length}">
        {#each steps as s, i (s)}<span class="dot" class:on={i <= step} title={s}></span>{/each}
      </div>
      <button class="btn" onclick={close}>Überspringen</button>
    </div>

    {#if step === 0}
      <h1 id="ob-title">Willkommen bei zero-teleprompter</h1>
      <p class="lead">Ein Beispielskript ist geladen. In einer Minute bist du startklar: Mikrofon, Sprachmodell, Ausgabe.</p>
      <ul class="facts">
        <li><b>Offline.</b> Audio bleibt auf deinem Gerät.</li>
        <li><b>Tastatur zuerst.</b> Leertaste startet, <span class="kbd">Ctrl K</span> findet alles.</li>
        <li><b>Voice-Tracking.</b> Der Text folgt deiner Stimme – und wartet, wenn du pausierst.</li>
      </ul>
      <div class="actions"><button class="btn primary" onclick={() => go(1)}>Los geht’s</button></div>
    {:else if step === 1}
      <h1 id="ob-title">Darf zero-teleprompter zuhören?</h1>
      <p class="lead">Audio bleibt auf deinem Gerät. Kein Upload, keine Cloud.</p>
      <div class="meter" role="meter" aria-label="Eingangspegel" aria-valuemin={-60} aria-valuemax={0} aria-valuenow={Math.round(store.level)}>
        <span style:width="{levelPct}%"></span>
      </div>
      <p class="transcript">{micOn ? (store.transcript ? `„${store.transcript}“` : 'Sag etwas…') : ' '}</p>
      {#if store.engine && micOn && !store.engine.microphone}
        <p class="footnote warn">Kein Mikrofon verfügbar oder Zugriff verweigert. Voice kannst du später aktivieren.</p>
      {/if}
      <div class="actions">
        <button class="btn" onclick={() => go(2)}>Später</button>
        {#if micOn}
          <button class="btn primary" onclick={() => go(2)}>Weiter</button>
        {:else}
          <button class="btn primary" onclick={() => mic(true)}><Icon name="mic" />Mikro erlauben</button>
        {/if}
      </div>
    {:else if step === 2}
      <h1 id="ob-title">Sprachmodell</h1>
      <p class="lead">Für Voice-Tracking brauchst du ein lokales Modell (einmaliger Download).</p>
      {#if models.length === 0}
        <p class="footnote">Kein Modellkatalog in diesem Build. Voice läuft im Demo-Modus; Modelle kannst du später im Inspector laden.</p>
      {/if}
      {#each models as m (m.entry.id)}
        {@const p = store.modelProgress[m.entry.id]}
        <div class="model">
          <span>{m.entry.name} <span class="footnote">· {Math.round(m.totalSize / 1e6)} MB · {m.entry.license}</span></span>
          {#if m.status === 'installed' || p?.finished}
            <span class="ok">Installiert</span>
          {:else if p}
            <progress max={p.total} value={p.done}></progress>
          {:else}
            <button class="btn outline" onclick={() => api.modelsInstall(m.entry.id).catch((e) => store.toast(errorText(e), 'error'))}>Laden</button>
          {/if}
        </div>
      {/each}
      <div class="actions">
        <button class="btn" onclick={() => go(3)}>Im Hintergrund weiter</button>
        <button class="btn primary" onclick={() => go(3)}>Weiter</button>
      </div>
    {:else}
      <h1 id="ob-title">Ausgabe</h1>
      <p class="lead">Wo soll der Text erscheinen? Spiegeln für Teleprompter-Glas.</p>
      <div class="monitors" role="radiogroup" aria-label="Monitor">
        {#each monitors as m (m.index)}
          <button class="mon" role="radio" aria-checked={chosen === m.index} onclick={() => (chosen = m.index)}>
            <span class="num big">{m.index + 1}</span><span class="footnote">{m.width}×{m.height}</span>
          </button>
        {/each}
      </div>
      <label class="check">
        <input type="checkbox" checked={store.state?.output.mirrorH} onchange={() => store.send({ type: 'toggleMirrorH' })} /> Horizontal spiegeln
      </label>
      <div class="actions">
        <button class="btn" onclick={close}>Fertig</button>
        <button class="btn primary" onclick={finish}>Probelauf starten</button>
      </div>
    {/if}
  </div>
{/if}

<style>
  .scrim { position: fixed; inset: 0; background: var(--ui-overlay); z-index: 70; }
  .modal {
    position: fixed; z-index: 71; left: 50%; top: 50%; transform: translate(-50%, -50%);
    width: min(560px, calc(100vw - 32px)); padding: 24px 28px 24px;
    background: var(--ui-surface-raised); border-radius: var(--radius-xl); box-shadow: var(--elev-4);
    animation: pop var(--dur-slow) var(--ease-standard);
  }
  .top { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
  .dots { display: flex; gap: 6px; }
  .dot { width: 8px; height: 8px; border-radius: 50%; background: var(--ui-border-strong); }
  .dot.on { background: var(--ui-accent); }
  h1 { font: 700 28px/34px var(--font-ui); margin: 0 0 8px; }
  .lead { font: 500 15px/22px var(--font-ui); color: var(--ui-text-muted); margin: 0 0 16px; }
  .facts { margin: 0 0 8px; padding-left: 18px; display: grid; gap: 6px; }
  .actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 24px; }
  .meter { height: 10px; border-radius: 5px; background: var(--ui-surface-sunken); overflow: hidden; }
  .meter span { display: block; height: 100%; background: var(--state-tracking); transition: width 66ms linear; }
  .transcript { min-height: 22px; font-style: italic; color: var(--ui-text-muted); }
  .warn { color: var(--state-lost); }
  .model { display: flex; justify-content: space-between; align-items: center; gap: 12px; padding: 8px 0; border-bottom: 1px solid var(--ui-border); }
  .ok { color: var(--state-tracking); font-weight: 600; }
  .monitors { display: flex; gap: 10px; flex-wrap: wrap; margin-bottom: 12px; }
  .mon { width: 110px; height: 70px; border-radius: var(--radius-md); border: 1px solid var(--ui-border-strong); background: var(--ui-surface-sunken); color: var(--ui-text); cursor: pointer; display: grid; place-items: center; }
  .mon[aria-checked='true'] { border-color: var(--ui-accent); background: var(--ui-accent-subtle); }
  .big { font-size: 22px; font-weight: 600; }
  @keyframes pop { from { opacity: 0; transform: translate(-50%, -48%); } }
</style>
