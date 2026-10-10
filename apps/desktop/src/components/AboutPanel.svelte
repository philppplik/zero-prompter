<!--
  Settings → "Über": app name, version (Tauri getVersion), build commit,
  licence, project links (opened via the fixed-key `open_link` command) and credits.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getVersion } from '@tauri-apps/api/app';
  import Icon from './Icon.svelte';
  import logo from '../../app-icon.png';
  import { api, errorText, isTauri } from '../lib/bridge';
  import { store } from '../lib/store.svelte';
  import { APP_NAME, formatBuild, LICENSE, LINKS, type LinkKey } from '../lib/about';

  let version = $state<string | null>(null);
  let features = $state<string[]>([]);
  const commit = typeof __APP_COMMIT__ === 'string' ? __APP_COMMIT__ : '';

  onMount(async () => {
    if (!isTauri) return;
    version = await getVersion().catch(() => null);
    try {
      const b = await api.buildInfo();
      features = [b.cpal && 'Audio (cpal)', b.sherpa && 'Voice (sherpa-onnx)', b.download && 'Modell-Download'].filter(Boolean) as string[];
    } catch {
      /* optional */
    }
  });

  async function open(key: LinkKey, url: string) {
    try {
      await api.openLink(key);
    } catch (e) {
      await navigator.clipboard?.writeText(url).catch(() => {});
      store.toast(`Link kopiert (${errorText(e)})`);
    }
  }
</script>

<section class="hero">
  <img src={logo} alt="" width="56" height="56" draggable="false" />
  <div>
    <h3 class="name">{APP_NAME}</h3>
    <p class="footnote">{formatBuild(version, commit)}</p>
    <p class="footnote">Freier, quelloffener Teleprompter mit Offline-Voice-Tracking.</p>
  </div>
</section>

<section>
  <h3 class="caption">Lizenz</h3>
  <p>Lizenziert unter <strong>{LICENSE}</strong> – nach Wahl.</p>
  {#if features.length}<p class="footnote">Build-Features: {features.join(' · ')}</p>{/if}
</section>

<section>
  <h3 class="caption">Links</h3>
  <ul class="links">
    {#each LINKS as l (l.key)}
      <li>
        <button class="link" onclick={() => open(l.key, l.url)} title={l.url}>
          <span>{l.label}</span><Icon name="external" size={14} />
        </button>
      </li>
    {/each}
  </ul>
</section>

<section>
  <h3 class="caption">Credits</h3>
  <p class="footnote">
    Gebaut mit Tauri, Svelte und Rust. Spracherkennung über sherpa-onnx (Apache-2.0) mit Modellen
    gemäß den Modelllizenzen. Schriften: Inter, DM Sans, Atkinson Hyperlegible, JetBrains Mono, Geist Mono (SIL OFL).
    Danke an alle Mitwirkenden.
  </p>
</section>

<style>
  .hero { display: flex; align-items: center; gap: 16px; }
  .hero img { border-radius: 14px; box-shadow: var(--elev-2); user-select: none; }
  .name { margin: 0 0 2px; font: 600 17px/24px var(--font-ui); letter-spacing: -0.01em; text-transform: none; color: var(--ui-text); }
  .hero p { margin: 0; }
  .links { list-style: none; margin: 0; padding: 0; display: grid; gap: 2px; }
  .link {
    display: flex; align-items: center; justify-content: space-between; gap: 8px; width: 100%;
    height: 34px; padding: 0 10px; border: 0; border-radius: var(--radius-sm);
    background: transparent; color: var(--ui-text); font: 500 13px/20px var(--font-ui); text-align: left; cursor: pointer;
    transition: background var(--dur-fast) var(--ease-standard), color var(--dur-fast) var(--ease-standard);
  }
  .link:hover { background: var(--ui-surface-sunken); }
  .link :global(svg) { color: var(--ui-text-muted); }
</style>
