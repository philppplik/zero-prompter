<!--
  Edit mode (CONCEPT F1/F2, DESIGN §3.1): Markdown textarea with debounced
  EditScript + autosave into the library. Read-only while playing (text lock).
-->
<script lang="ts">
  import { store } from '../lib/store.svelte';
  import { api } from '../lib/bridge';
  import { newScriptId, openFileDialog } from '../lib/actions';
  import { displayChord } from '../lib/hotkeys';

  interface Props {
    onsaved?: () => void;
  }
  let { onsaved }: Props = $props();

  const st = $derived(store.state);
  let text = $state(store.state?.script.source ?? '');
  let lastSent = text;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  let status = $state<'saved' | 'dirty' | 'saving'>('saved');

  // External loads (import, library, palette) replace the buffer.
  $effect(() => {
    const src = st?.script.source ?? '';
    void st?.script.version;
    if (src !== lastSent && src !== text) {
      text = src;
      lastSent = src;
    }
  });

  const words = $derived(st?.script.tokens.filter((t) => !t.is_cue && !/^#+$/.test(t.text)).length ?? 0);
  const minutes = $derived(words / Math.max(40, st?.scroll.wpm ?? 140));
  const longLines = $derived(text.split('\n').filter((l) => l.length > 400).length);
  const locked = $derived(store.running);

  function onInput() {
    status = 'dirty';
    clearTimeout(timer);
    timer = setTimeout(async () => {
      lastSent = text;
      await store.send({ type: 'editScript', value: text });
      scheduleSave();
    }, 250);
  }

  function scheduleSave() {
    clearTimeout(saveTimer);
    saveTimer = setTimeout(async () => {
      status = 'saving';
      if (!store.scriptId) {
        const id = newScriptId(text.split('\n').find((l) => l.trim())?.replace(/^#+\s*/, '') ?? 'skript');
        store.scriptId = id;
        store.setPrefs({ lastScriptId: id });
      }
      try {
        await api.saveScript(store.scriptId!, text);
        status = 'saved';
        onsaved?.();
      } catch (e) {
        status = 'dirty';
        store.toast(`Speichern fehlgeschlagen: ${e}`, 'error');
      }
    }, 800);
  }
</script>

<div class="editor">
  {#if text.trim() === ''}
    <div class="drop" aria-hidden="true">
      <p class="t">Skript hier einfügen</p>
      <p class="footnote">{displayChord('Ctrl+V')} zum Einfügen · .txt/.md hierher ziehen · <button class="link" onclick={openFileDialog}>Datei öffnen</button></p>
    </div>
  {/if}
  <textarea
    bind:value={text}
    oninput={onInput}
    readonly={locked}
    spellcheck="true"
    lang={st?.settings.language ?? 'de'}
    aria-label="Skripttext (Markdown). Regieanweisungen in [eckigen Klammern]."
    placeholder=""
  ></textarea>
  <div class="status footnote">
    <span class="num">{words} Wörter · ca. {Math.floor(minutes)}:{String(Math.round((minutes % 1) * 60)).padStart(2, '0')} min bei {st?.scroll.wpm ?? 140} WPM</span>
    {#if longLines > 0}<span>· {longLines} sehr lange Absätze</span>{/if}
    <span class="spacer"></span>
    {#if locked}<span>Gesperrt während der Wiedergabe</span>{:else}<span>{status === 'saved' ? 'Gespeichert' : status === 'saving' ? 'Speichert…' : 'Ungespeichert'}</span>{/if}
  </div>
</div>

<style>
  .editor { position: relative; display: flex; flex-direction: column; height: 100%; background: var(--ui-bg); }
  textarea {
    flex: 1; width: 100%; resize: none; border: 0; outline: none;
    padding: 32px max(24px, calc((100% - 720px) / 2));
    background: transparent; color: var(--ui-text);
    font: 400 15px/26px var(--font-ui);
    tab-size: 2;
  }
  textarea[readonly] { color: var(--ui-text-muted); }
  .drop {
    position: absolute; inset: 24px 24px 56px; display: grid; place-content: center; text-align: center;
    border: 1.5px dashed var(--ui-border-strong); border-radius: var(--radius-lg); pointer-events: none;
  }
  .drop .t { font: 600 17px/24px var(--font-ui); margin: 0 0 4px; }
  .drop .link { pointer-events: auto; border: 0; background: none; color: var(--ui-accent); font: inherit; cursor: pointer; padding: 0; text-decoration: underline; }
  .status { display: flex; gap: 6px; align-items: center; height: 28px; padding: 0 16px; border-top: 1px solid var(--ui-border); }
  .spacer { flex: 1; }
</style>
