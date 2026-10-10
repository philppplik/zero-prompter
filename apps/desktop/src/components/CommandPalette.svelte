<!--
  Command palette Ctrl/⌘K (DESIGN §3.4): combobox + listbox, fuzzy match with
  bold hits, grouped results, ↑/↓ navigate, Enter run, Esc close.
-->
<script lang="ts">
  import { tick } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { dur, EASE } from '../lib/motion';
  import Icon from './Icon.svelte';
  import { store } from '../lib/store.svelte';
  import { ACTION_SHORTCUT, paletteActions, type Action } from '../lib/actions';
  import { highlight } from '../lib/fuzzy';
  import { searchActions } from '../lib/palette';
  import { displayChord } from '../lib/hotkeys';

  let query = $state('');
  let active = $state(0);
  let input: HTMLInputElement;
  let list: HTMLUListElement;

  const all = $derived(store.paletteOpen ? paletteActions() : []);

  const results = $derived(searchActions<Action>(all, query));

  function shortcutOf(id: string): string | null {
    const hk = store.state?.settings.hotkeys[id];
    const c = hk ?? ACTION_SHORTCUT[id];
    return c ? displayChord(c) : null;
  }

  $effect(() => {
    if (store.paletteOpen) {
      query = '';
      active = 0;
      tick().then(() => input?.focus());
    }
  });
  $effect(() => {
    void query;
    active = 0;
  });

  function run(a: Action) {
    store.paletteOpen = false;
    a.run();
  }

  function onKey(e: KeyboardEvent) {
    e.stopPropagation(); // palette owns the keyboard while open
    if (e.key === 'ArrowDown') {
      active = Math.min(results.length - 1, active + 1);
      e.preventDefault();
    } else if (e.key === 'ArrowUp') {
      active = Math.max(0, active - 1);
      e.preventDefault();
    } else if (e.key === 'Enter') {
      const r = results[active];
      if (r) run(r.a);
      e.preventDefault();
    } else if (e.key === 'Escape') {
      store.paletteOpen = false;
      e.preventDefault();
    } else return;
    tick().then(() => list?.querySelector('[aria-selected="true"]')?.scrollIntoView({ block: 'nearest' }));
  }
</script>

{#if store.paletteOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="scrim" onclick={() => (store.paletteOpen = false)} transition:fade={{ duration: dur('base'), easing: EASE.standard }}></div>
  <div class="layer">
  <div
    class="palette"
    role="dialog"
    aria-modal="true"
    aria-label="Befehle"
    in:scale={{ start: 0.97, opacity: 0, duration: dur('panel'), easing: EASE.emphasized }}
    out:scale={{ start: 0.98, opacity: 0, duration: dur('fast'), easing: EASE.exit }}
  >
    <div class="search">
      <Icon name="search" />
      <input
        bind:this={input}
        bind:value={query}
        onkeydown={onKey}
        role="combobox"
        aria-expanded="true"
        aria-controls="palette-list"
        aria-activedescendant={results[active] ? `pa-${results[active].a.id}` : undefined}
        aria-autocomplete="list"
        placeholder="Aktion, Theme oder Einstellung suchen…"
        spellcheck="false"
      />
    </div>
    <ul id="palette-list" role="listbox" bind:this={list} aria-label="Ergebnisse">
      {#each results as r, i (r.a.id)}
        {#if !query.trim() && (i === 0 || results[i - 1].a.group !== r.a.group)}
          <li class="group caption" role="presentation">{r.a.group}</li>
        {/if}
        <li
          id="pa-{r.a.id}"
          role="option"
          aria-selected={i === active}
          class:active={i === active}
          onclick={() => run(r.a)}
          onmousemove={() => (active = i)}
          onkeydown={() => {}}
        >
          <span class="label">{#each highlight(r.a.label, r.idx) as seg, si (si)}{#if seg.hit}<b>{seg.s}</b>{:else}{seg.s}{/if}{/each}</span>
          {#if r.a.hint}<span class="hint num">{r.a.hint}</span>{/if}
          {#if shortcutOf(r.a.id)}<span class="kbd">{shortcutOf(r.a.id)}</span>{/if}
        </li>
      {:else}
        <li class="none" role="presentation">Keine Aktion für „{query}“</li>
      {/each}
    </ul>
    <div class="foot footnote">
      <span><span class="kbd">↑↓</span> wählen</span>
      <span><span class="kbd">↵</span> ausführen</span>
      <span><span class="kbd">esc</span> schließen</span>
    </div>
  </div>
  </div>
{/if}

<style>
  .scrim { position: fixed; inset: 0; background: var(--ui-overlay); z-index: 50; backdrop-filter: blur(6px); -webkit-backdrop-filter: blur(6px); }
  .layer { position: fixed; inset: 0; z-index: 51; pointer-events: none; display: flex; justify-content: center; align-items: flex-start; padding: 18vh 16px 0; }
  .palette {
    pointer-events: auto; transform-origin: 50% 0;
    width: min(600px, 100%); max-height: 60vh; display: flex; flex-direction: column; min-height: 0;
    background: var(--ui-surface-raised); border: 1px solid var(--ui-border); border-radius: var(--radius-xl);
    box-shadow: var(--elev-4); overflow: hidden;
  }
  .search { display: flex; align-items: center; gap: 10px; padding: 0 16px; height: 52px; border-bottom: 1px solid var(--ui-border); color: var(--ui-text-muted); }
  .search input { flex: 1; border: 0; outline: none; background: transparent; color: var(--ui-text); font: 500 15px/22px var(--font-ui); height: 100%; }
  ul { list-style: none; margin: 0; padding: 6px; overflow-y: auto; min-height: 0; overscroll-behavior: contain; }
  li[role='option'] {
    display: flex; align-items: center; gap: 10px; height: 34px; padding: 0 10px;
    border-radius: var(--radius-md); cursor: pointer;
    transition: background var(--dur-fast) var(--ease-standard);
  }
  li.active { background: var(--ui-surface-sunken); }
  .label { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .label b { font-weight: 700; color: var(--ui-text); text-decoration: underline; text-decoration-color: var(--ui-border-strong); text-underline-offset: 3px; }
  .hint { color: var(--ui-text-muted); font-size: 12px; }
  .group { padding: 10px 10px 4px; }
  .none { padding: 16px 10px; color: var(--ui-text-muted); }
  .foot { display: flex; gap: 16px; padding: 8px 16px; border-top: 1px solid var(--ui-border); }
</style>
