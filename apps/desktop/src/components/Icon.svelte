<!-- Minimal inline icon set (own drawings, 16px grid, stroke = currentColor). -->
<script lang="ts">
  interface Props {
    name: keyof typeof PATHS;
    size?: number;
  }
  const PATHS = {
    play: 'M5 3.5v9l7.5-4.5z',
    pause: 'M5 3.5v9M11 3.5v9',
    home: 'M4 3.5v9M12 3.5 6.5 8l5.5 4.5z',
    paraPrev: 'M3 4h10M3 8h6M3 12h10M12 9.5 10 8l2-1.5',
    paraNext: 'M3 4h10M3 8h6M3 12h10M10 6.5 12 8l-2 1.5',
    mirror: 'M8 2v12M6 4 2.5 8 6 12zM10 4l3.5 4L10 12z',
    monitor: 'M2.5 3.5h11v7h-11zM6 13.5h4M8 10.5v3',
    search: 'M7 12a5 5 0 1 0 0-10 5 5 0 0 0 0 10zM10.8 10.8 14 14',
    mic: 'M8 2a2 2 0 0 0-2 2v4a2 2 0 0 0 4 0V4a2 2 0 0 0-2-2zM4 7.5a4 4 0 0 0 8 0M8 11.5V14',
    plus: 'M8 3v10M3 8h10',
    minus: 'M3 8h10',
    file: 'M4 2h5l3 3v9H4zM9 2v3h3',
    chevron: 'M6 4l4 4-4 4',
    timer: 'M8 14a5 5 0 1 0 0-10 5 5 0 0 0 0 10zM8 6.5V9l1.5 1M6.5 1.5h3',
    x: 'M4 4l8 8M12 4l-8 8',
    // Sliders (settings) – replaces the former sun-like gear glyph.
    settings: 'M2.5 4.5h6M11.5 4.5h2M2.5 11.5h2M7.5 11.5h6M10 3v3M6 10v3',
    type: 'M3 4.5V3h10v1.5M8 3v10M6 13h4',
    marker: 'M5.5 8h8M2 5.5 4.5 8 2 10.5M5.5 4.5h6M5.5 11.5h6',
    sidebar: 'M2.5 3.5h11v9h-11zM6 3.5v9',
    command: 'M5.5 6.5h5v3h-5zM5.5 6.5V5a1.5 1.5 0 1 0-1.5 1.5zM10.5 6.5V5a1.5 1.5 0 1 1 1.5 1.5zM5.5 9.5V11a1.5 1.5 0 1 1-1.5-1.5zM10.5 9.5V11a1.5 1.5 0 1 0 1.5-1.5z',
    exit: 'M6 3.5H3.5v9H6M9.5 5.5 12 8l-2.5 2.5M12 8H6',
    keyboard: 'M1.5 4.5h13v7h-13zM4 7h.01M6.5 7h.01M9 7h.01M11.5 7h.01M5 9.5h6',
    edit: 'M10.5 2.5l3 3L6 13H3v-3z',
    live: 'M8 10a2 2 0 1 0 0-4 2 2 0 0 0 0 4zM4.5 4.5a5 5 0 0 0 0 7M11.5 4.5a5 5 0 0 1 0 7',
    trash: 'M3 4.5h10M6 4.5V3h4v1.5M4.5 4.5l.7 9h5.6l.7-9',
    info: 'M8 14a6 6 0 1 0 0-12 6 6 0 0 0 0 12zM8 7.5V11M8 5h.01',
    external: 'M9 2.5h4.5V7M13.5 2.5 7.5 8.5M11.5 9.5v3.5h-9v-9H6',
    // Window controls (custom title bar, Windows/Linux).
    winMin: 'M3.5 8h9',
    winMax: 'M3.5 3.5h9v9h-9z',
    winRestore: 'M3.5 5.5h7v7h-7zM5.5 5.5v-2h7v7h-2',
    panelOpen: 'M2.5 3.5h11v9h-11zM6 3.5v9M8.5 6.5 10 8l-1.5 1.5',
    panelClose: 'M2.5 3.5h11v9h-11zM6 3.5v9M10 6.5 8.5 8l1.5 1.5',
  } as const;
  let { name, size = 16 }: Props = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 16 16"
  fill={name === 'play' ? 'currentColor' : 'none'}
  stroke="currentColor"
  stroke-width="1.5"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
  focusable="false"><path d={PATHS[name]} /></svg>
