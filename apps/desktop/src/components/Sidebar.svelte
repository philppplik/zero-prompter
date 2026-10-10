<!-- Script drawer (CONCEPT §4.1): script library + outline (paragraph jump). Hidden by default, Ctrl+B. -->
<script lang="ts">
  import { ask } from '@tauri-apps/plugin-dialog';
  import Icon from './Icon.svelte';
  import { store } from '../lib/store.svelte';
  import { api, errorText } from '../lib/bridge';
  import { newScriptId, openFileDialog } from '../lib/actions';
  import type { ScriptMeta } from '../lib/types';

  interface Props {
    scripts: ScriptMeta[];
    onrefresh: () => void;
    onclose?: () => void;
  }
  let { scripts, onrefresh, onclose }: Props = $props();

  const st = $derived(store.state);
  const outline = $derived.by(() => {
    if (!st) return [];
    return st.script.paragraphs.map((p) => {
      const words = st.script.tokens
        .slice(p.tokens.start, Math.min(p.tokens.end, p.tokens.start + 8))
        .filter((t) => !/^#+$/.test(t.text))
        .map((t) => t.text);
      return { index: p.index, start: p.tokens.start, end: p.tokens.end, label: words.join(' ') };
    });
  });
  const current = $derived(store.currentToken);

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.stopPropagation();
      e.preventDefault();
      onclose?.();
    }
  }

  let nav = $state<HTMLElement | null>(null);
  $effect(() => {
    nav?.querySelector<HTMLElement>('li.active .item, .item, button')?.focus();
  });

  async function openScript(id: string) {
    if (store.running) return store.toast('Erst pausieren, dann Skript wechseln', 'error');
    try {
      await api.loadScript(id);
      store.scriptId = id;
      store.setPrefs({ lastScriptId: id });
    } catch (e) {
      store.toast(errorText(e), 'error');
    }
  }

  async function newScript() {
    if (store.running) return;
    const id = newScriptId('neues-skript');
    const text = '# Neues Skript\n\n';
    if (await store.send({ type: 'loadScript', value: text })) {
      await api.saveScript(id, text).catch(() => {});
      store.scriptId = id;
      store.setPrefs({ lastScriptId: id });
      if (store.live) store.send({ type: 'enterEdit' });
      onrefresh();
    }
  }

  async function remove(s: ScriptMeta) {
    const ok = await ask(`„${s.title}“ löschen? Die Datei wird aus der Bibliothek entfernt.`, { title: 'Skript löschen', kind: 'warning' });
    if (!ok) return;
    try {
      await api.deleteScript(s.id);
      if (store.scriptId === s.id) store.scriptId = null;
      onrefresh();
    } catch (e) {
      store.toast(errorText(e), 'error');
    }
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<nav class="sidebar" aria-label="Skripte und Gliederung" bind:this={nav} onkeydown={onKey}>
  <section>
    <div class="head">
      <span class="caption">Skripte</span>
      <span class="actions">
        <button class="btn icon" aria-label="Neues Skript" title="Neues Skript" onclick={newScript}><Icon name="plus" /></button>
        <button class="btn icon" aria-label="Skript öffnen (Ctrl+O)" title="Öffnen (Ctrl+O)" onclick={openFileDialog}><Icon name="file" /></button>
        {#if onclose}<button class="btn icon" aria-label="Skripte schließen (Esc)" title="Schließen (Esc)" onclick={onclose}><Icon name="x" /></button>{/if}
      </span>
    </div>
    {#if scripts.length === 0}
      <p class="footnote empty">Noch keine Skripte. Datei hierher ziehen oder Ctrl+V im Editor.</p>
    {/if}
    <ul>
      {#each scripts as s (s.id)}
        <li class:active={s.id === store.scriptId}>
          <button class="item" onclick={() => openScript(s.id)} title="{s.words} Wörter">
            <Icon name="file" /><span class="label">{s.title}</span>
          </button>
          <button class="btn icon del" aria-label="„{s.title}“ löschen" onclick={() => remove(s)}><Icon name="trash" size={14} /></button>
        </li>
      {/each}
    </ul>
  </section>

  <section class="outline">
    <div class="head"><span class="caption">Gliederung</span></div>
    <ol>
      {#each outline as o (o.index)}
        <li class:active={current >= o.start && current < o.end}>
          <button class="item" onclick={() => store.send({ type: 'seekToken', value: o.start })}>
            <span class="num idx">{o.index + 1}</span><span class="label">{o.label || '…'}</span>
          </button>
        </li>
      {/each}
    </ol>
  </section>
</nav>

<style>
  .sidebar {
    width: min(320px, calc(100vw - 48px));
    background: var(--ui-surface-raised);
    display: flex; flex-direction: column;
    min-height: 0; height: 100%;
    overflow: hidden;
  }
  section { padding: 14px 10px 8px; display: flex; flex-direction: column; min-height: 0; max-height: 45%; }
  .outline { flex: 1; max-height: none; border-top: 1px solid var(--ui-border); }
  .head { display: flex; align-items: center; justify-content: space-between; padding: 0 8px 6px; }
  .actions { display: flex; gap: 2px; }
  ul, ol { list-style: none; margin: 0; padding: 0; overflow-y: auto; min-height: 0; overscroll-behavior: contain; }
  li { display: flex; align-items: center; border-radius: var(--radius-sm); }
  li.active { background: var(--ui-accent-subtle); }
  li.active .item { color: var(--ui-accent); font-weight: 600; }
  .item {
    flex: 1; min-width: 0; display: flex; align-items: center; gap: 8px;
    height: 28px; padding: 0 8px; border: 0; background: none; color: var(--ui-text);
    font: 400 13px/20px var(--font-ui); text-align: left; cursor: pointer; border-radius: var(--radius-sm);
    transition: background var(--dur-fast) var(--ease-standard), transform var(--dur-fast) var(--ease-standard);
  }
  .item:active { transform: scale(.99); }
  .item:hover { background: var(--ui-surface-sunken); }
  .label { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .idx { color: var(--ui-text-muted); font-size: 11px; min-width: 16px; }
  .del { opacity: 0; }
  li:hover .del, .del:focus-visible { opacity: 1; }
  .empty { padding: 0 8px; margin: 0; }
</style>
