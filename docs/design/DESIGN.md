# zero-teleprompter – Design-Spezifikation

Stand: 10.10.2026 · Grundlage: `docs/research/RESEARCH.md`, `docs/concept/CONCEPT.md` · Tokens: `docs/design/tokens.json` · Mockup: `docs/design/mockup.html`

---

## 1. Designprinzipien

1. **Der Text ist das Interface.** Im Talent-Fenster existiert nur Text, Lesemarker und (Voice) ein Wort-Highlight. Kein Chrome, keine Badges, keine Warnungen.
2. **Ruhe vor Reaktion.** Bewegung nur, wenn sie Information trägt. Scrollen startet und stoppt mit Ease (600 ms), nie ruckartig. Unsicherheit = Stillstand (Prinzip „Nicht raten“).
3. **Operator sieht alles, Talent nichts Unnötiges.** Diagnose (lost, Pegel, Confidence) lebt im Control-Fenster.
4. **Tastatur zuerst.** Jede Aktion hat einen Shortcut und ist in der Command Palette (Ctrl/⌘K) auffindbar.
5. **Zustand über Farbe *und* Form *und* Text.** Nie Farbe allein (WCAG 1.4.1).
6. **Native Zurückhaltung.** Systemfont-nahe UI (Inter/Segoe/SF), Mica/Vibrancy in Titlebar, 13-px-Basis wie macOS/Linear. Ein Akzent (Violett), sonst Graustufen.
7. **Messbar lesbar.** Themes tragen ihren Kontrastwert; alle Presets ≥ 7:1 (AAA).

---

## 2. Token-System

Namensschema: `ui.*` (Chrome), `prompter.<theme>.*`, `state.*`, `type.*`, `space.*`, `radius.*`, `elevation.*`, `motion.*`, `layout.*`, `focus.*`. Kanonisch: `tokens.json`. Implementierung als CSS Custom Properties (`--ui-bg`, `--pr-text`, …), Theme-Wechsel per `data-theme` am Root.

### 2.1 UI-Farben (Chrome)

Quelle: tweakcn-Theme **„Minimal Neutral“** (`tweakcn.com/themes/cmho4nr9l000h04l1gu419ckw`), oklch → sRGB-Hex umgerechnet. Mapping shadcn → UI-Token: background → `bg`, card → `surface`, muted/input → `surfaceSunken`, border → `border`, secondary → `borderStrong`, foreground → `text`, muted-foreground → `textMuted`, primary → `accent`, primary-foreground → `textOnAccent`, accent → `accentSubtle`, ring → `focusRing`, destructive → `danger`. Die Prompter-Themes (2.3) sind davon unabhängig.

| Token | Light | Dark | Kontrast (auf bg) |
|---|---|---|---|
| `ui.bg` | `#FFFFFF` | `#171717` | – |
| `ui.surface` | `#FDFDFD` | `#0E0E0E` | – |
| `ui.surfaceRaised` | `#FFFFFF` | `#1F1F1F` | Dialoge, Pille, Palette |
| `ui.surfaceSunken` | `#F5F5F5` | `#262626` | Hover, Inputs |
| `ui.border` / `borderStrong` | `#EEEEEE` / `#D4D4D4` | `#262626` / `#404040` | Hairlines; Strong für Inputs |
| `ui.text` | `#0A0A0A` | `#FAFAFA` | ≥ 18 : 1 (AAA) |
| `ui.textMuted` | `#737373` | `#A3A3A3` | 4,7 : 1 / 7,6 : 1 (AA) |
| `ui.accent` (primary) | `#171717` | `#E5E5E5` | Text auf Akzent ≥ 15 : 1 |
| `ui.accentSubtle` | `#EEEEEE` | `#404040` | aktive/pressed Flächen |
| `ui.focusRing` | `#A3A3A3` | `#737373` | Ring + 3-px-Halo |
| `ui.cardFrom` → `cardTo` | `#FFFFFF` → `#F4F4F5` | `#1F1F1F` → `#121212` | Verlauf der Hauptkarte |

### 2.2 Zustandsfarben (Voice)

| Zustand | Farbe | Form | Kontrast auf `#0E0F12` |
|---|---|---|---|
| idle | `#8A8F98` | Kreis, Outline | 5,90 : 1 |
| listening | `#4CC2FF` | Kreis, pulsierend | 8,93 : 1 (auf surface) |
| tracking | `#3DD68C` | Kreis, gefüllt | 10,22 : 1 |
| hold | `#FFB224` | Zwei Pausenbalken | 10,63 : 1 |
| lost | `#F2555A` | Dreieck mit „!“ | 5,31 : 1 (auf surface) |
| resync | `#8E7BFF` | rotierender Bogen | ≥ 4,5 : 1 |

### 2.3 Prompter-Themes (6 Presets)

| Theme | bg | text | Kontrast | WCAG | Marker | Gelesen-Opazität | Hinweis |
|---|---|---|---|---|---|---|---|
| **Studio Dark** (Default) | `#0E0F12` | `#F4F1EA` | 16,99 : 1 | AAA | `#FFB224` | 40 % | warmes Off-White, weniger Halo auf Glas |
| **Broadcast Classic** | `#000000` | `#FFFFFF` | 21,00 : 1 | AAA | `#FFE14D` | 40 % | klassisch für Spiegelglas |
| **Daylight** | `#FAF8F3` | `#1A1A1A` | 16,40 : 1 | AAA | `#C2410C` | 45 % | helle Räume, Overlay am Laptop |
| **High Contrast** | `#000000` | `#FFE14D` | 16,13 : 1 | AAA | `#FFFFFF` | 55 % | Sehschwäche; gelesener Text bleibt ≥ 4,5 : 1 |
| **Low Light** | `#120D08` | `#C9B79A` | 9,87 : 1 | AAA | `#B5542C` | 45 % | dunkle Sets, geringe Blendung |
| **Dyslexia** | `#FDF6E3` | `#1F2A44` | 13,22 : 1 | AAA | `#2563EB` | 45 % | Creme-Hintergrund, +0,04 em Laufweite, +0,16 em Wortabstand, ZA 1,6 |

Regel: Gelesener Text (dimmed) muss ≥ 3 : 1 bleiben (Wiederauffinden nach Rücksprung). Theme-Editor (v1.0) blockiert Speichern unter 7 : 1 mit Warnung, unter 4,5 : 1 hart.

### 2.4 Typografie

UI: **DM Sans** (Variable, via `@fontsource-variable/dm-sans`). Mono: **Geist Mono** (Timecodes, WPM, Kbd). Beide lokal gebündelt, kein CDN. Prompter: **Atkinson Hyperlegible Next** 500 (wählbar: Inter, System, Serif, JetBrains Mono).

| Rolle | px / Zeile | Gewicht | Verwendung |
|---|---|---|---|
| caption | 11 / 16 | 500 | Labels über Feldern, Kbd-Hints (Versalien, +0,04 em) |
| footnote | 12 / 16 | 400 | Hilfetexte, Statuszeile |
| body | 13 / 20 | 400 | Standard-UI |
| bodyStrong | 13 / 20 | 600 | Buttons, aktive Items |
| callout | 15 / 22 | 500 | Palette-Eingabe, Onboarding-Text |
| title3 | 17 / 24 | 600 | Panel-Titel |
| title2 | 22 / 28 | 600 | Dialog-Titel |
| title1 | 28 / 34 | 700 | Onboarding-Headline |
| display | 96 / 96 | 700 | Countdown (tabular-nums) |
| prompter | 24–300 (Default 72), ZA 1,4, 25–40 ch | 500 | Talent-Text |

Zahlen immer `font-variant-numeric: tabular-nums`.

### 2.5 Spacing (4-px-Raster, 2 als Halbschritt)

`0=0 · 1=2 · 2=4 · 3=8 · 4=12 · 5=16 · 6=20 · 7=24 · 8=32 · 9=40 · 10=48 · 11=64`. Komponenten-Innenabstand 8/12, Panel-Padding 16, Sektionen 24.

### 2.6 Radii

`xs 4` (Kbd, Tags) · `sm 8` (Buttons, Inputs) · `md 10` (Kacheln, Listen-Items) · `lg 16` (= tweakcn `--radius: 1rem`; Vorschau) · `xl 20` (Dialoge, Palette, Drawer) · `card 24` (Hauptkarte) · `pill 999` (Aktions-Pille, Segmented, Toasts, Statusindikator).

### 2.7 Elevation

| Stufe | Verwendung | Shadow |
|---|---|---|
| 0 | flache Flächen | none |
| 1 | Pille, Inputs, Logo | tweakcn `shadow-sm` (`0 1px 3px /10%, 0 1px 2px -1px /10%`) |
| 2 | Hauptkarte, Tooltips | tweakcn `shadow-lg` (`… 0 4px 6px -1px /10%`) |
| 3 | Live-Karte, Toasts | tweakcn `shadow-xl` + weicher 40-px-Schatten |
| 4 | Dialoge, Palette, Drawer | `0 24px 64px -12px /30%` |

Im Dark-Mode zusätzlich Oberflächen-Aufhellung (surface → surfaceRaised) statt stärkerer Schatten.

### 2.8 Motion

| Token | Dauer | Kurve | Einsatz |
|---|---|---|---|
| fast | 120 ms | standard `cubic-bezier(.2,0,0,1)` | Hover, Press, Toggle |
| base | 200 ms | enter `(0,0,0,1)` / exit `(.3,0,1,1)` | Popover, Panel |
| slow | 300 ms | standard | Palette, Dialoge |
| playEase | 600 ms | `(.45,0,.55,1)` | Scroll Start/Stop (Geschwindigkeit, nicht Position) |
| resync | ≤ 300 ms | Feder k=170, d=26 | Voice-Sprung |
| countdown | 1000 ms/Ziffer | scale .9→1 + fade | 3-2-1 |

| panel | 260 ms | emphasized `(.32,.72,0,1)` | Dialog (scale .96→1 + fade), Palette, Drawer (fly x −24), Toasts (fly y 12 + flip) |

UI-Transitions laufen über `src/lib/motion.ts` (`dur()`, `EASE.*` aus tokens.json, Bezier-Solver für Svelte-Transitions); Ausgang immer `fast` + exit-Kurve. Microinteractions: Press `scale(.96)` (Play `.92`, Hover `1.04`), Hover-Flächen 120 ms.
`prefers-reduced-motion` **oder** die Einstellung „Bewegung reduzieren“: alle UI-Transitions 0 ms. Prompter: Resync springt hart, kein Pulsieren, Countdown nur Crossfade, Popover ohne Scale. Scrollen selbst bleibt (Funktion).
Scroll-Rendering: `transform: translate3d(0,-y,0)` auf einem Layer, Position aus `requestAnimationFrame` mit Zeitdelta (nicht Frame-basiert), Subpixel aus, auf Linux Pixel-Snapping.

---

## 3. Komponenten

### 3.1 Control-Fenster
Minimal, viel Weißraum: Topbar 64 (links nur App-Logo; rechts Voice-Status, **Aktions-Pille** und runder **X**-Button), darunter eine große Karte (radius `card` 24, Verlauf `cardFrom → cardTo`, max. 1200 px) mit Kopfzeile (Skripttitel · Scroll-Modus · `Edit | Live`) und Editor bzw. Live-Vorschau, darunter die Transport-Leiste 56. Keine permanente Sidebar und kein Inspector: Skripte/Gliederung als Drawer von links (`Ctrl+B`), Einstellungen als Modal (`Ctrl+,`). Alle Flex/Grid-Ketten mit `min-height: 0`, damit Panels intern scrollen. X: in Live = Live beenden, in Edit = Fenster schließen. Min. 960×600.
Modi: **Edit** (Editor) / **Live** (1:1-Vorschau, verkleinert). Zustände: leer (Drop-Zone + „Ctrl+V zum Einfügen“), geladen, live, Talent-Fenster getrennt (Banner „Ausgabe nicht verbunden – Monitor wählen“).

### 3.2 Toolbar
Aktions-Pille (40 px, radius pill, surfaceRaised + elevation 1): Skripte (Sidebar-Icon, `Ctrl+B`) · „Befehle“ + `⌘K` · Talent-Ausgabe (Monitor-Icon, grüner Punkt wenn aktiv) · Einstellungen (Regler-Icon, `Ctrl+,`). Spiegeln, Themes usw. über Palette/Einstellungen/Tasten.
Buttons: 28 px hoch (Hit-Target 32 via Padding), Zustände default / hover (surfaceSunken) / pressed (−4 % Scale, 120 ms) / active (accentSubtle + text) / focus-visible (2-px-Ring, Offset 2) / disabled (40 % Opazität, `aria-disabled`).

### 3.3 Transport-Leiste
Play/Pause (primär, 40 px rund), Zurück zum Anfang, Absatz ±, WPM-Stepper (− 140 +, Mono), Fortschritt (Balken + `03:12 / 07:40` Restzeit), Countdown-Toggle.

### 3.4 Command Palette (Ctrl/⌘K)
600 px breit, 20 % von oben, radius lg, elevation 3, Eingabe callout 15 px. Gruppen: Zuletzt · Wiedergabe · Darstellung · Themes · Ausgabe · Voice · Hilfe. Jede Zeile: Icon 16, Label, rechts Shortcut-Kbd. Fuzzy-Match mit fett markierten Treffern.
Zustände: leer (Zuletzt + Vorschläge), Ergebnisse, kein Treffer („Keine Aktion für ‚xyz‘“), Unterseite (z. B. Theme-Liste mit Live-Vorschau beim Hovern), Ladezustand (Modelle). ↑/↓ navigiert, Enter ausführt, Esc schließt / geht eine Ebene zurück, Tab = Unterseite.

### 3.5 Einstellungen (Modal, ersetzt den Inspector)
840 × 640 px (max. Viewport − 32), radius xl, elevation 4, Scrim mit 6-px-Blur. Links vertikale Tabs (`role=tablist`, Pfeiltasten/Home/End), rechts scrollender Tab-Inhalt. Tabs: **Darstellung** (Theme-Kacheln, Schrift, Größe 24–300, Spaltenbreite, Zeilenabstand, Dimmen, Farben + Kontrast, Lesemarker, Oberfläche Hell/Dunkel/System, Frametime), **Scroll** (Modus, WPM, Countdown, Bewegung reduzieren), **Voice & Modelle** (Sprache, Mikro + Pegel + Test, Verhalten bei lost, Modellliste), **Tastenkürzel** (HotkeyEditor), **Ausgabe** (Monitor, Talent-Fenster, Vollbild, Spiegel H/V). Esc/Klick daneben schließt, Fokus-Trap, Fokus kehrt zum Auslöser zurück (`Dialog.svelte`, auch für die Kürzel-Übersicht).
Felder: Label caption oben, Control, Hilfetext footnote. Slider mit numerischem Feld daneben. Validierungsfehler inline in `state.lost`-Farbe + Icon + Text.

### 3.6 Talent-Fenster
Vollflächig `prompter.bg`; Textspalte zentriert, `measureCh` 32. Oben 35 % Leerraum, damit Satz 1 auf der Fokuslinie beginnt; unten 60 % Auslauf. Regieanweisungen kursiv, 55 % Opazität. Gespiegelt via `transform: scale(-1,1)` auf Container (nicht Text-Shaping). Cursor versteckt nach 1,5 s. Einzige erlaubte Zustandsdarstellung: `hold` = dezenter Ring am Marker (state.hold, 30 % Opazität).
Zustände: Countdown, Playing, Paused (Marker bleibt), Ende („— Ende —“ in direction-Farbe), Kein Skript (nur Logo-Glyphe, 10 %).

### 3.7 Lesemarker
Position Default 35 % Höhe (verschiebbar per Drag im Control-Vorschau, 5–70 %).
- **Linie:** 2 px, Marker-Farbe, volle Spaltenbreite + 24 px.
- **Band:** Zeilenhöhe, Marker 12 % Fläche, Kanten 1 px 40 %.
- **Pfeile:** Dreiecke links und rechts am Fensterrand (0,3 em × Größe).
- **Klammern:** eckige Klammern an den Spaltenkanten, Zeilenhöhe × Größe.
- **Fokus-Fenster (Spotlight):** alles außer der Lesezone mit Hintergrundfarbe abgedunkelt (Deckkraft × 85 %), optional weichgezeichnet (`backdrop-filter`, 0–8 px).
- **Verlauf:** weiches Farbband, darüber/darunter blendet der Text über 3 Zeilen in den Hintergrund aus.
- **Unterstreichung:** aktuelles Wort oder aktueller Satz (bis Satzzeichen `.!?…`), 0,07 em, mitlaufend im Content-Layer.
- **Seitenbalken:** vertikaler Akzentbalken (4 px × Größe) links der Spalte.
- **Kein Marker:** unsichtbar; in der Vorschau bleibt ein Drag-Griff (gestrichelt bei Hover/Fokus).

Einstellungen (`Settings.marker`, zt-core `marker.rs`, gespiegelt in `lib/marker.ts`): `style`, `color` (null = UI-Pref/Theme; Vorlagen beginnen mit dem Theme-Akzent), `opacity` 0,1–1, `size` 0,5–3, `dimRead` 0–0,9 (null = Theme-`readOpacity`), `dimUnread` 0–0,9, `glow` 0–1 (drop-shadow bis 16 px), `blur` 0–8 px, `animation` none/glide (180 ms transform)/pulse (Opacity-Flash bei Zeilenwechsel), `underlineScope` word/sentence. Rendering in `ReadingMarker.svelte` innerhalb der gespiegelten Stage: Talent, Vorschau und Einstellungs-Vorschau sind identisch, H/V-Spiegelung wirkt automatisch. Performance: Position nur über `transform`, Effekte über `opacity`/`filter`; DOM-Schreibzugriffe pro Frame nur bei Zeilen-/Bereichswechsel. `prefers-reduced-motion` bzw. „Bewegung reduzieren“ schaltet Animationen ab.
Text oberhalb = gelesen → `readOpacity`. Voice: aktuelles Wort mit `wordHighlight`-Hintergrund, radius xs, Übergang 120 ms.

### 3.8 Voice-Tracking-Statusindikator (Operator)
Pill 24 px hoch, Icon 12 + Label + optional Confidence-Balken. Zusätzlich `aria-live="polite"` (lost: `assertive`).

| Zustand | Icon / Form | Label | Verhalten | Talent sieht |
|---|---|---|---|---|
| idle | ○ Outline grau | „Voice aus“ | statisch | nichts |
| listening | ● blau, Pulse 1,6 s | „Hört zu…“ + Pegel | wartet auf 1. Match | nichts |
| tracking | ● grün gefüllt | „Folgt · 142 WPM“ | Confidence-Balken | Wort-Highlight |
| hold | ‖ amber | „Wartet“ | Text steht | dezenter Ring am Marker |
| lost | ▲! rot | „Verloren – klicke ein Wort“ | Pill wird Button, Shake 1× (nicht bei reduced motion), Banner im Hauptbereich nach 4 s | nichts |
| resync | ◠ violett rotierend | „Springt…“ | ≤ 300 ms, dann tracking | Ease-Sprung |

### 3.9 Countdown
Overlay auf Talent und Vorschau: Ziffer display 96 px × Skalierungsfaktor (= 2× Prompter-Größe), Farbe Marker, zentriert auf Fokuslinie; je 1 s, scale .9→1 + fade. Esc/Space bricht ab. Dauer 0/3/5 s einstellbar.

### 3.10 Onboarding (≤ 60 s, 4 Schritte)
Modal 560 px, radius xl, elevation 4, Step-Dots oben, „Überspringen“ immer sichtbar.
1. **Willkommen** – Beispielskript ist geladen (Primär: „Los geht’s“).
2. **Mikrofon** – Erklärung („Audio bleibt auf deinem Gerät“), dann OS-Prompt; Live-Pegel + Transkript. Abgelehnt → Hinweis, Voice später aktivierbar.
3. **Sprachmodell** – Deutsch (71 MB) / English, Fortschritt mit Resume; „Im Hintergrund weiter“.
4. **Ausgabe** – Monitor-Kacheln (schematisch, mit Nummer-Overlay auf echten Displays), Spiegeln-Toggle. Abschluss: „Probelauf starten“ → Countdown.

---

## 4. Layouts (ASCII-Wireframes)

### 4.1 Control-Fenster, Live-Modus
```
┌──────────────────────────────────────────────────────────────────────────────┐
│ ● ● ●   Keynote_Q4.md ▾                                       ⌘K  Suchen…    │ Titlebar 38
├──────────────────────────────────────────────────────────────────────────────┤
│ [Edit|Live]   [Manuell|Konstant|Voice]        (● Folgt · 142 WPM ▮▮▮▯) ⇋ 🖥2 │ Toolbar 44
├────────────┬──────────────────────────────────────────┬──────────────────────┤
│ SKRIPTE    │ ┌──────────── Talent-Vorschau ─────────┐ │ DARSTELLUNG       ▾ │
│ ▸ Keynote  │ │   gelesener Text (40 %)              │ │ [■][■][■][■][■][■]  │
│   Intro    │ │━━━━━ heute zeigen wir Ihnen ━━━━━━━━━│ │ Größe  ──●──── 72   │
│ OUTLINE    │ │   wie zero-teleprompter zuhört …     │ │ Breite ──●──── 32ch │
│ 1 Einstieg │ │                                      │ │ SCROLL            ▾ │
│ 2 Demo   ◀ │ └──────────────────────────────────────┘ │ WPM  [−] 140 [+]    │
│ 3 Fazit    │  Marker ⇕ 35 %                           │ VOICE             ▸ │
├────────────┴──────────────────────────────────────────┴──────────────────────┤
│ ⏮  ◀¶  ( ▶ )  ¶▶   −140+ WPM   ▬▬▬▬▬▬▬▬▬░░░░░░░  03:12 / 07:40   ⏲3s        │ Transport 56
└──────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 Talent-Fenster (Monitor B)
```
┌──────────────────────────────────────────────┐
│        Willkommen zur Präsentation.          │  ← 40 % Opazität
│        Heute zeigen wir Ihnen,               │
│ ▶━━━━━━━━ wie [zero‑teleprompter] ━━━━━━━━━━ │  ← Fokuslinie 35 %, Wort-Highlight
│        Ihrer Stimme folgt –                  │
│        auch auf Deutsch.                     │
│                                              │
└──────────────────────────────────────────────┘
```

### 4.3 Command Palette
```
        ┌────────────────────────────────────────────────┐
        │ 🔍 them|                                        │
        ├────────────────────────────────────────────────┤
        │ THEMES                                          │
        │ ▸ Studio **Them**e: Dark          17,0:1   ⌘1  │
        │   Broadcast Classic               21,0:1   ⌘2  │
        │   High Contrast                   16,1:1   ⌘4  │
        ├────────────────────────────────────────────────┤
        │ ↑↓ wählen   ↵ ausführen   ⇥ Details   esc      │
        └────────────────────────────────────────────────┘
```

### 4.4 Onboarding, Schritt 2
```
┌──────────────────────────────────────────────┐
│ ● ● ○ ○                         Überspringen │
│ Darf zero-teleprompter zuhören?              │
│ Audio bleibt auf deinem Gerät. Kein Upload.  │
│ Pegel ▮▮▮▮▮▯▯▯▯▯   „Hallo, Test eins zwei“   │
│                       [Später]  [Mikro erlauben] │
└──────────────────────────────────────────────┘
```

---

## 5. Interaktionen & Shortcuts

| Aktion | Win/Linux | macOS |
|---|---|---|
| Play/Pause | Space | Space |
| Tempo ±5 WPM | ↑ / ↓ | ↑ / ↓ |
| Absatz zurück/vor | PgUp / PgDn | PgUp / PgDn |
| Zum Anfang | Home | Home / ⌘↑ |
| Spiegeln H (V mit Shift) | M / Shift+M | M / ⇧M |
| Voice an/aus | V | V |
| Resync am Marker | R | R |
| Vollbild Talent | F | F |
| Schriftgröße | Ctrl + / − | ⌘ + / − |
| Command Palette | Ctrl+K | ⌘K |
| Theme 1–6 | Ctrl+1…6 | ⌘1…6 |
| Edit/Live | Ctrl+E | ⌘E |
| Öffnen / Einfügen | Ctrl+O / Ctrl+V | ⌘O / ⌘V |
| Shortcut-Übersicht | ? | ? |
| Abbrechen (Countdown, Dialog) | Esc | Esc |

Weitere Interaktionen: Klick auf Wort in Vorschau = Cursor/Resync; Mausrad im Live = manueller Override 2 s, danach Voice-Resync; Marker-Linie per Drag verschieben; Drag&Drop von .txt/.md auf jedes Fenster; Doppelklick (pausiert, 0.2) = Inline-Edit. Remapping via Shortcut-Recorder (Konfliktwarnung inline).

---

## 6. Accessibility

- **Kontrast:** UI-Text ≥ 4,5 : 1, alle Prompter-Themes ≥ 7 : 1 (AAA), Nicht-Text ≥ 3 : 1 (1.4.11).
- **Nicht nur Farbe:** Voice-Zustände haben Form + Label (1.4.1).
- **Fokus:** sichtbarer 2-px-Ring, nie verdeckt (2.4.11/2.4.13), logische Tab-Reihenfolge Titlebar → Toolbar → Sidebar → Haupt → Inspector → Transport; Skip-Link „Zur Wiedergabe“.
- **Ziele:** min. 24×24 px (2.5.8), Transport-Buttons 32–40 px.
- **Screenreader:** Statusindikator `role="status"`; lost `aria-live="assertive"`; Slider mit `aria-valuetext` („72 Pixel“); Palette als `combobox` + `listbox`.
- **Bewegung:** `prefers-reduced-motion` (s. 2.8); kein Blinken > 3 Hz.
- **Lesbarkeit:** Dyslexia-Theme, Größen bis 300 px, Gelesen-Dimmen abschaltbar, Ausrichtung links (0.2).
- **Sprache:** `lang` je Skript, DE/EN-UI.
- **Zoom:** UI skaliert bis 200 % ohne Inhaltsverlust (Inspector wird Sheet).
- **Tastatur:** 100 % Bedienbarkeit, keine Keyboard-Traps im Talent-Fenster (Esc verlässt Vollbild).
