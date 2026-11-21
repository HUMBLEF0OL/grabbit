---
name: Grabbit website
description: Graphite at page scale, in the app's Graphite green. One stage where the real app does the work, one emerald for the action, Grabbit's own numbers and the way around.
colors:
  ground: "#fafafa"
  tray: "#ffffff"
  ink: "#18181b"
  mute: "#6b6b74"
  line: "#e4e4e7"
  go: "#0c704d"
  on-go: "#ffffff"
  selection: "#e6f2ec"
  ink-2: "#52525b"
  tray-2: "#f1f1f3"
  go-wash: "#f0f6f4"
  go-tint: "#e7f1ed"
  go-tint-strong: "#d8e8e3"
  ground-dark: "#121214"
  tray-dark: "#18181b"
  ink-dark: "#f4f4f5"
  mute-dark: "#8e8e98"
  line-dark: "#2b2b30"
  go-dark: "#3ecf8e"
  on-go-dark: "#06281a"
  selection-dark: "#15302a"
  ink-2-dark: "#b4b4bc"
  tray-2-dark: "#222227"
  go-wash-dark: "#1a2322"
  go-tint-dark: "#1c2a26"
  go-tint-strong-dark: "#1e352d"
typography:
  display:
    fontFamily: "Geist, Segoe UI, system-ui, sans-serif"
    fontSize: "clamp(46px, 6vw, 84px)"
    fontWeight: 600
    lineHeight: 1
    letterSpacing: "-0.045em"
  statement:
    fontFamily: "Geist, Segoe UI, system-ui, sans-serif"
    fontSize: "clamp(34px, 4.2vw, 58px)"
    fontWeight: 500
    lineHeight: 1.1
    letterSpacing: "-0.04em"
  headline:
    fontFamily: "Geist, Segoe UI, system-ui, sans-serif"
    fontSize: "clamp(32px, 3.6vw, 48px)"
    fontWeight: 600
    lineHeight: 1.05
    letterSpacing: "-0.035em"
  title:
    fontFamily: "Geist, Segoe UI, system-ui, sans-serif"
    fontSize: "22px"
    fontWeight: 600
    lineHeight: 1.25
    letterSpacing: "-0.02em"
  lede:
    fontFamily: "Geist, Segoe UI, system-ui, sans-serif"
    fontSize: "20px"
    fontWeight: 400
    lineHeight: 1.5
  body:
    fontFamily: "Geist, Segoe UI, system-ui, sans-serif"
    fontSize: "18px"
    fontWeight: 400
    lineHeight: 1.5
    fontFeature: "'tnum' 1"
  small:
    fontFamily: "Geist, Segoe UI, system-ui, sans-serif"
    fontSize: "15.5px"
    fontWeight: 400
    lineHeight: 1.5
  label:
    fontFamily: "Geist, Segoe UI, system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.5
  code:
    fontFamily: "Cascadia Mono, Consolas, SF Mono, ui-monospace, monospace"
    fontSize: "0.88em"
    fontWeight: 400
rounded:
  micro: "4px"
  window: "8px"
  well: "12px"
  core: "16px"
  shell: "24px"
  pill: "999px"
spacing:
  wrap: "1200px"
  gutter: "32px"
  gutter-phone: "16px"
  grid-gap: "16px"
  cell-pad: "28px"
  section: "160px"
components:
  button-go:
    backgroundColor: "{colors.go}"
    textColor: "{colors.on-go}"
    typography: "{typography.body}"
    rounded: "{rounded.pill}"
    padding: "0 24px"
    height: "52px"
  button-go-dark:
    backgroundColor: "{colors.go-dark}"
    textColor: "{colors.on-go-dark}"
    rounded: "{rounded.pill}"
    padding: "0 24px"
    height: "52px"
  button-ghost:
    backgroundColor: "transparent"
    textColor: "{colors.ink}"
    rounded: "{rounded.pill}"
    padding: "0 24px"
    height: "52px"
  nav-pill:
    backgroundColor: "{colors.tray}"
    textColor: "{colors.mute}"
    rounded: "{rounded.pill}"
    height: "48px"
  nav-link:
    textColor: "{colors.mute}"
    rounded: "{rounded.pill}"
    padding: "7px 12px"
  stage-bezel:
    rounded: "{rounded.shell}"
    padding: "10px"
  stage-core:
    backgroundColor: "{colors.tray}"
    rounded: "{rounded.core}"
    padding: "48px 64px 0"
  cell:
    backgroundColor: "{colors.tray}"
    textColor: "{colors.ink}"
    rounded: "{rounded.shell}"
    padding: "{spacing.cell-pad}"
  window:
    rounded: "{rounded.window}"
---

# Design System: Grabbit website

> **Status.** The home page (`website/index.html`, `website/home.css`, `website/home.js`, and the benchmark chart drawn by `bench_bars()` in `tools/website/build.py`) is the reference implementation of this system. The other pages (download, benchmark, IDM alternative, features, help, FAQ, news, legal) render from `website/styles.css`, whose tokens map onto this system: the ground (`--board`), the tray (`--paper`), ink, Graphite green (`--go`, and `--link` with the same value), Geist and pill buttons. Their layouts are the older ones (sheets with bands, the IDM alternative's label), restyled. New pages and reworked sections are built from this document and `home.css`.

## Overview

**Creative North Star: "One Take"**

The camera never cuts. A single centred stage holds the real Grabbit window doing its work. As the visitor scrolls, the stage stays put and the copy walks through the app's four jobs beside it. The site wears the app's own palette, Graphite green (`design/graphite/source/pal.py`, "graphite": Zinc + Emerald), at page scale: a zinc-black or near-white ground, one raised tray surface, hairlines lit along the top edge, and a single emerald that means "this is Grabbit acting". Proof comes from the app and its measured numbers. Nothing on the page is drawn to look like the app.

Density is calm and generous: a 1200px column, 160px between sections, big tight-tracked headings, muted ink for everything secondary. Colour is scarce on purpose. Emerald marks the Download action and Grabbit's own figures, and points the way (links, focus, selection) as it does in the app; the gradient lives only inside the double-chevron mark. Light and dark are equal citizens and follow the visitor's system setting by default.

IDM appears only as the benchmark's yardstick (grey bars, plain-weight figures) and as a "moving from IDM" path. It is never the frame of reference for the look.

**Key Characteristics:**
- Zinc-black / near-white ground, one tray surface, 1px hairlines with a top light edge.
- One accent, the app's emerald: filled for Download and Grabbit's own numbers, as text or a ring for links, focus and selection.
- Geist in three weights, tight negative tracking, tabular figures everywhere.
- Pills for controls, nested 24/16px shells for stages and cells, 8px for app windows.
- Every image is the real app in its light and dark versions, framed and lifted, never mocked.
- Motion plays once or runs to measured time, and every element has a finished state at rest.

## Colors

The app's Graphite green palette (pal.py "graphite", contrast-checked there): zinc neutrals with no tint, and one emerald that does the work and points the way. The site follows the app; when pal.py's "graphite" changes, these values change with it.

### Primary
- **Graphite Green** (go / go-dark): the Download pill (nav, hero, close), Grabbit's bar and figures in the benchmark chart, the green words of the benchmark statement, the active walk-through rail segment, the filling connection bars, and the "Complete" state. It sits on On-Green text: white in light mode, near-black green in dark mode. Its tints mark Grabbit's own row and results, never decoration: go-wash (6% on the tray) behind the comparison's own row, go-tint (10%) behind its live strip and table row, go-tint-strong (16%) for the result pills ("Delivered") with ink text.

### Pointing
- **The same green, as a line** (go / go-dark): inline links (underlined 1px at a 4px offset), the 2px focus ring, and the selection tint (selection / selection-dark), as the app does. As a pointer it is never a fill, a heading or a button.

### Neutral
- **Ground** (ground / ground-dark): the page itself.
- **Tray** (tray / tray-dark): the one raised surface. It carries the stage's core, the chart, the bento cells and the glass nav (at 78% with blur).
- **Ink** (ink / ink-dark): headings, primary text, the ghost button's label, and "Free". Translucent ink is the system's only tint: 4-9% for wells, ghost fills and hovers, 22% for the IDM bars, 8% for empty connection tracks.
- **Ink 2** (ink-2 / ink-2-dark): secondary reading text on the inner pages (answers, captions, values beside a figure). **Tray 2** (tray-2 / tray-2-dark): the band header of a tray and selected rows on the inner pages.
- **Mute** (mute / mute-dark): ledes, body copy in cells, the second line of the hero headline, nav links, chart labels and footer links.
- **Hairline** (line / line-dark): 1px rings around the tray surfaces, window borders, section rules, and the inactive rail.

### Named Rules
**The One Signal Rule.** A green fill means Grabbit is acting: the Download action, or a number Grabbit measured. Never use it for decoration, a heading or IDM's bars.

**The Green Points Rule.** Green as a line or text only shows where you can go or where you are: links, focus, selection. It never fills a surface.

**The Mark Owns The Gradient Rule.** The mark's emerald gradient (Graphite green, the app's palette) appears only inside the Grabbit mark (`website/assets/grabbit.svg`). There is no gradient text, gradient fill or glow anywhere else.

## Typography

**Display Font:** Geist SemiBold 600 (falling back to Segoe UI, then system-ui)
**Body Font:** Geist Regular 400 and Medium 500, self-hosted (`website/assets/fonts/geist`, OFL)

**Character:** One neutral grotesk in three weights. Hierarchy comes from size and negative tracking, which gets tighter as the size grows. All figures are tabular, so times line up.

### Hierarchy
- **Display** (600, clamp(46px, 6vw, 84px), 1.0, -0.045em, balanced wrap): the hero headline in two lines, the second in Mute. The closing "Get Grabbit" uses the same voice at clamp(44px, 5.4vw, 76px).
- **Statement** (500, clamp(34px, 4.2vw, 58px), 1.1, -0.04em): the one benchmark sentence, lit word by word, with its Grabbit figure in green.
- **Headline** (600, clamp(32px, 3.6vw, 48px), 1.05, -0.035em, max 18ch): section headings.
- **Title** (600, 21-22px, 1.25, -0.02em): walk-through steps and bento cells.
- **Lede** (400, 20px, 1.5, max 600px, pretty wrap): the line under the hero headline, in Mute.
- **Body** (400, 18px, 1.5): the page default. Copy in cells and steps sits at 15.5-17px in Mute.
- **Label** (400, 14px): nav links, chart key, chart note, footer links. The chart's figures are 600 at 13.5px for Grabbit and 400 for IDM. Axis ticks are 12px.
- **Code** (Cascadia Mono, 0.88em): only for what a person types or reads as code: command lines, switches, file paths and keys in the help and comparison pages. Never as a "technical" costume.

### Named Rules
**The Tighter As It Grows Rule.** Tracking scales with size: -0.01em at 15-18px, -0.02em at titles, -0.035 to -0.045em at headlines and display. Body copy is never tracked.

**The Tabular Figures Rule.** Tabular figures are set on the body, so every time, size and count lines up without extra work.

## Layout

A centred 1200px wrap with 32px side padding (16px under 560px). Sections sit 160px apart. The benchmark statement starts 120px below the walk-through and leaves 18% empty on its right so it wraps short.

- **Hero:** centred, 148px from the top (120px on phones). Headline, then the lede 24px below, then the buttons 36px below that, then the stage 64px below. The stage is up to 1080px wide, and a mask fades its lower ~38% into the ground at the fold.
- **Walk-through (signature):** a 300vh section with a sticky full-viewport grid. Steps on the left, stage on the right (5fr / 7fr, 72px gap). A 600px tall stage cross-fades between the jobs' windows. Under 1000px the pin goes away: the steps stack, and each one shows its own screenshot inline.
- **Benchmark chart:** one tray panel. Rows are a 250px name column plus the bars, all on one shared seconds axis (0-16 s, a tick every 4 s). Under 1000px each name sits above its bars.
- **Bento:** a 12-column grid, 320px rows, 16px gaps, with cells spanning 7/5(two rows)/4/3/6/6. It becomes one column under 1000px (cells at least 240px tall).
- **Close and footer:** a centred close section above a hairline, 120px padding. Then a four-column footer (2fr 1fr 1fr 1fr), two columns under 1000px.
- **Breakpoints:** 1000px (nav links hide, pin and bento collapse) and 560px (gutters shrink, hero buttons go full width, the nav's wordmark hides).

## Elevation & Depth

Depth comes from tonal layering plus a single lift. The ground is flat. The tray rises from it through a 1px hairline ring and a 1px inner light edge along the top. Only real app windows cast a shadow, so they read as objects sitting on the stage. The glass nav is the one floating control.

### Shadow Vocabulary
- **Tray edge** (`box-shadow: inset 0 1px 0 var(--edge), 0 0 0 1px var(--line)`): every tray surface (chart, bento cells, nav). Edge is white at 70% in light mode and 7% in dark mode. On hover a cell's ring deepens to 18% ink.
- **Window lift** (`box-shadow: 0 40px 80px -30px rgb(24 24 27 / .28), 0 2px 6px rgb(24 24 27 / .08)`, dark: `0 40px 80px -30px rgb(0 0 0 / .7), 0 2px 6px rgb(0 0 0 / .4)`): app screenshots only, always together with a 1px hairline border.
- **Nav float** (`inset 0 1px 0 var(--edge), 0 0 0 1px var(--line), 0 10px 30px -14px rgb(0 0 0 / .35)` with `backdrop-filter: blur(14px) saturate(1.4)`): the glass pill nav only.
- **Pill bevel** (`inset 0 1px 0 rgb(255 255 255 / .25), inset 0 -1px 0 rgb(0 0 0 / .15)`): the green Download pill only.

### Named Rules
**The Only Windows Lift Rule.** Drop shadows belong to the app's windows. Panels and cells rise through a hairline and a top light edge, never through a drop shadow or a glow.

## Shapes

The geometry is soft and nested. Controls are full pills (999px), and so are the skip link, nav links and the theme button (a circle). Stages are a double bezel: a 24px outer shell (4% ink, hairline ring, 10px padding) around a 16px tray core, so the inner radius equals the outer radius minus the padding. Bento cells and the chart use the same 24px. App windows sit at 8px (in the hero stage only the top corners are rounded, because the window bleeds out of the bottom). The path well is 12px. Tags and badges are pills like the controls; links, menus, logos and pictures inside pages take the window's 8px. Elements under 20px (code, keys, check boxes, flags, grid cells, thin bars) take a 4px micro radius, the one size below 8px. Bars and connection tracks have half-height radii (4px and 3px). The inactive rail is 3px wide with 2px ends. Lists use a short 8 x 1.5px dash, never a glyph bullet.

## Components

### Buttons
Few and round: one green pill and one ghost.
- **Shape:** pill (999px), 52px tall, 24px horizontal padding, 600 at 16px, an optional 18px stroke icon with a 10px gap. In the nav: 36px tall, 16px padding, 14px text.
- **Primary (Download):** Graphite Green with On-Green text and the pill bevel.
- **Ghost:** 5% ink fill with a hairline ring and ink text. Hover goes to 9%.
- **Hover / Active / Focus:** primary rises 1px and brightens to 1.06 on a springy `cubic-bezier(.34,1.56,.64,1)` over .45s. Both buttons press to scale .97. Focus is the global 2px green ring at a 4px offset.

### Cards / Containers (tray cells)
- **Corner Style:** 24px.
- **Background:** Tray, with the tray edge (see Elevation).
- **Internal Padding:** 28px. The chart uses 36px 40px 28px.
- **Content:** a 21px title, a Mute line no wider than 34ch, then one piece of proof pinned to the bottom (a live connection strip, browser logos at 38px, an 88px ink "Free", a lifted screenshot crop bleeding off an edge, or a menu-path well). Hovering lifts a crop 6px over .7s.

### Navigation
- **Style:** a floating glass pill fixed 16px from the top and centred, 48px tall. It holds the mark and wordmark (600, 15px), Mute pill links (14px) that hover to ink on a 6% ink tint, a 34px circular theme toggle, and the small green Download.
- **Mobile:** under 1000px only the mark, the toggle and Download remain. Under 560px the wordmark hides too.

### Stage (signature)
A double-bezel tray that frames real app windows. In the hero it is 1080px wide, the window has 48px 64px of breathing room, and the whole thing fades out at the fold. In the walk-through it is 600px tall, and each job's window enters with a .6s fade plus a .9s rise from 14px at .97 scale on `cubic-bezier(.16,1,.3,1)`.

### Walk-through Rail (signature)
Four steps, each with a 3px rail segment and a 22px title. Inactive steps sit at 38% opacity. The active step goes to full opacity, fills its rail with green, and opens its Mute description. The step that crosses mid-viewport becomes active.

### Benchmark Chart (signature)
Rendered from `website/bench/results.json` by `bench_bars()`, so the page and the data cannot disagree. Each scenario gets two 8px bars on a shared axis: Grabbit in green with a 600 green figure, IDM in 22% ink with a 400 Mute figure. A key and a sourcing note in Mute complete it. The bars grow once from zero over 1.1s when the chart scrolls into view, and they rest at full width without JS or with reduced motion.

### Connection Strip
Eight 6px tracks, numbered in 12.5px Mute, that fill in green in real time to the benchmark's 1.01 s median. Tracks still filling show 75% green, finished ones full green. A 34px clock counts up, the state reads Starting, Downloading, then Complete (in green), holds for 1.8 s and replays while visible. Reduced motion shows it complete.

## Do's and Don'ts

### Do:
- **Do** build every page from the Ground / Tray / Ink / Mute / Hairline tokens, in both themes, defaulting to the visitor's system choice.
- **Do** keep green fills to Download and Grabbit's own numbers, and green lines to links, focus and selection; take the values from the app's pal.py "graphite".
- **Do** frame real app screenshots, in light and dark versions, in an 8px window with a hairline and the window lift, inside a 24/16px double bezel or a tray cell.
- **Do** set type in Geist 400/500/600 with tracking that tightens as the size grows, and tabular figures.
- **Do** draw every chart from its data file at build time, and show IDM in neutral ink beside Grabbit's green.
- **Do** give every motion a finished state at rest: entrance rise once (.9s, `cubic-bezier(.16,1,.3,1)`), bars grow once, and reduced motion or no JS shows the end state.

### Don't:
- **Don't** use gradient text, glows, or any gradient outside the Grabbit mark.
- **Don't** put drop shadows on panels or cells. Only app windows lift, the nav floats, and the Download pill has its bevel.
- **Don't** draw a mock window or retouch a screenshot. Don't use IDM artwork, and don't present Grabbit as an IDM look-alike.
- **Don't** add a second typeface or set headings in uppercase or with wide tracking.
- **Don't** use square corners on controls or radii outside the 4 (micro, under 20px) / 8 / 12 / 16 / 24 / pill set.
- **Don't** bring back the old blue-board look (Archivo, the blue board, its bright flat green buttons, tilted labels) in any page.
