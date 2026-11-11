# 04 · Geist, chosen by the theme

Lets a theme name a bundled font, bundles Geist (SIL OFL), and makes Graphite use it. This is the one
look phase that changes engine code: today no font is bundled and a theme cannot choose one.

- Branch: `ui/04-typeface`
- Needs: 01 (Geist files and OFL.txt), 02
- Unblocks: 05b (layout sizes are tuned with the final font); 07 waves and 08 take their final sizes after it

## Facts this plan relies on (paths under `crates/app/src/`, checked on `ui/03-icons` c3541d0)

Line numbers drift as phases land: find each item by its name first.

- Fonts are read from the system. `theme::ui_font()` (`theme.rs:398-400`) loads segoeui.ttf (Windows
  only now: no platform branch); `load_fonts` (`:403`) wraps `theme::fonts()` (`:412-461`), which builds
  `FontDefinitions` ("segoe-ui" first, then `winui::install_fonts` families tahoma, tahoma-bold,
  arial-bold from `winui.rs:57-80`, then fallbacks and the `ScriptFallback` provider), then `set_fonts`
  and `text_styles`.
- `classic_text` (`theme.rs:466`) installs a DirectWrite rasterizer, `win32::text_rasterizer`
  (`win32.rs:4438`), at `FontPriority::Highest` (`:4452`) for Latin, Greek and Cyrillic. It maps families
  to system font files in `classic_font_file` (`win32.rs:4474-4484`). **Anything put in
  `FontDefinitions` is still drawn as Segoe UI for those scripts** unless this mapping learns about
  bundled fonts.
- Fonts load once in `theme::apply` (`theme.rs:512`); a theme switch (`apply_theme`, `:529`, from
  `Cmd::Theme` at `app.rs:5027`) only re-runs `text_styles`.
- A theme has `font.*` size metrics but no font family. `Theme` is defined at `palette.rs:587`.
- Interface size is egui `zoom_factor` (`scale.rs`), independent of the font.
- egui cannot switch OpenType features (no `tnum`), so tabular digits depend on the font's default figures.

## Steps

1. **Bytes**: `crates/app/assets/fonts/geist/` with Geist-Regular, Geist-Medium, Geist-SemiBold and
   `OFL.txt` (from `design/graphite/assets/fonts/`); `include_bytes!` them in `theme.rs` behind a small
   `bundled_font(name) -> Option<&'static [u8]>`.
2. **Theme field**: `Theme { font: Option<Cow<'static, str>> }`; `themefile::parse` accepts a top-level
   `font = "geist"` (unknown names warn and fall back to the system font). Other themes: `None`.
3. **Font definitions**: in `fonts()`, when the selected theme has a font, insert it as the first
   Proportional family (keep the system font and all fallbacks after it, so other scripts still draw).
   The bold family ("tahoma-bold" equivalents) maps to Geist-SemiBold for the theme, and a new
   `geist-medium` family carries Medium (500) for buttons, column heads and chips (DESIGN.md).
4. **Windows rasterizer**: teach `classic_font_file` / `text_rasterizer` to draw bundled faces: either
   (a) DirectWrite loads the bundled bytes through a custom font-file loader, or (b) the rasterizer is
   skipped for a bundled family and egui's own rasterizer draws it. Prototype both; pick by legibility at
   100% (Geist's hinting), by code size and by frame time (no more than 5% slower than today on the main
   window with 1,000 rows: the mean of `ctx.input(|i| i.unstable_dt)` over 600 frames while
   `cargo run -p mdm-labserver -- --faults "rate=2M"` feeds 10 running downloads). Record the choice in the PR.
5. **Reload on theme change**: `Cmd::Theme` re-runs `fonts(ctx)` when the font changes (not on every
   switch), so choosing Graphite or leaving it swaps the face live.
6. **Sizes**: Geist runs wider than Segoe UI. Set Graphite's `font.ui`, `font.body`, `font.small`,
   `font.heading` so the main window's labels fit as in the mock (11.5 toolbar labels, 13 body, 12
   column heads). Check every dialog for clipped labels at 100% and 200%; widen nothing in this phase:
   list any overflow for 05b and the 07 waves.
7. **Digits**: check that Geist's default figures are equal width (sizes and rates must line up in the
   list). If they are not, decide here between bundling Geist Mono for numbers and keeping the system
   font for numeric cells; default: Geist Mono not bundled. Record the decision in PLAN.md.
8. `graphite.toml`: `font = "geist"` (through `make_theme.py`'s `HEADER`, which writes the file).
   `ATTRIBUTION.md`: Geist, Vercel, SIL OFL 1.1.
9. **Size budget**: the installer grows by no more than 400 KB (three weights); state the actual figure in the PR.

## Tests

- `themefile` test: `font = "geist"` parses; an unknown font warns.
- A test that a theme switch from Classic to Graphite and back changes the first Proportional family.
- Existing text-rendering tests (sharp text, script fallback) still pass.

## Done when

- [x] Graphite draws in Geist on Windows; every other theme still draws in Segoe UI (Classic pixel-identical)
- [x] Switching themes swaps the face without a restart
- [x] No label newly clipped in the main window (file names end in "…" sooner; sizes left to 05b, step 6 below); overflow list for later phases written below
- [x] Digits decision recorded; size budget met; frame time measured as a CPU proxy (40 rows, debug build, below: +0.2 %), not by the live 1,000-row `unstable_dt` run
- [x] Screenshots before and after (main window, Options, Download File Info), light and dark
- [x] ATTRIBUTION, ROADMAP, CHANGELOG updated

## What was built and decided (2026-11-11, branch `ui/04-typeface`)

- **Step 4, the rasterizer: (a)**, DirectWrite reads the bundled bytes through
  `IDWriteFactory5::CreateInMemoryFontFileLoader` (`win32.rs` `classic_face`, `FaceKey::Bundled`),
  so Geist takes the same GDI-classic path as Segoe UI: whole-pixel advances, ttfautohint's
  hinting at 100 %. Both were prototyped and timed in one run, the three set-ups interleaved frame
  by frame (600 frames each after 60 of warm-up; 40 rows, 10 of them changing every frame; debug
  build, CPU time of `run_ui` + tessellation, so vsync can't hide a difference):

  | Set-up | Mean | Median |
  | --- | --- | --- |
  | Today: Segoe UI by DirectWrite | 8.724 ms | 5.703 ms |
  | (a) Geist by DirectWrite from memory | 8.744 ms (+0.2 %) | 5.666 ms |
  | (b) Geist by egui's rasterizer | 18.923 ms (+117 %) | 11.655 ms |

  (b) shapes every new galley in egui, which the DirectWrite path skips: it fails the 5 % budget.
  (a) is about 40 lines more than (b). Not run: the plan's live measurement (`unstable_dt` with
  the lab server feeding 10 downloads): at 60 Hz it is vsync's interval either way.
- **Families** (`theme.rs` `fonts_for`): with a bundled font, Regular leads Proportional,
  Monospace and `tahoma`; SemiBold `tahoma-bold` and `arial-bold`; Medium the new `geist-medium`
  (Segoe UI in every other theme, unused until 05b). Segoe UI follows each as the first fallback
  (on Geist's baseline), then the symbol fonts and `ScriptFallback`.
- **Step 5**: `apply_theme` remakes the fonts only when the theme's font changed
  (`needs_reload`); every theme path (View → Theme, Native and pywal following) goes through it.
  egui takes the new families at the next frame while DirectWrite's face changes at once, so
  glyphs first drawn in the rest of the switching frame can come in the other face; the next
  frame (asked for by the switch) makes a new atlas. Accepted: one frame at most.
- **Step 6, sizes**: Graphite keeps the classic sizes (`font.ui` 12, `font.body` 12,
  `font.small` 11, `font.heading` 16). In IDM's layout Geist at 12 already ends more file names
  in "…" than Segoe UI did; the mock's 11.5 toolbar labels, 13 body and 12 column heads need 05b's
  layout (its step 5 and 7), which sets them.
- **Step 7, digits**: Geist's default figures are proportional (Regular: 1 is 384 units, 0 is
  663; Segoe UI's are all 1104). Decision (PLAN.md): Geist Mono is not bundled; 05b draws the
  list's numeric cells in a `numbers` family on Segoe UI.
- **Step 9, size**: the app grows by 381,580 bytes (the three faces); compressed, LZMA (the
  installer's) 94,960 bytes, Deflate (the portable zip) 181,010 bytes: within 400 KB.

## Checks done

| Check | Result |
| --- | --- |
| fmt, clippy | `cargo fmt --all` clean; `cargo clippy -p mdm-app --all-targets -- -D warnings` clean |
| Tests | `cargo test -p mdm-app`: 317 pass (1 ignored, as before); new: `a_theme_names_a_bundled_font`, `a_theme_switch_swaps_the_face`, `fonts_are_remade_only_when_the_font_changes`, `directwrite_draws_the_bundled_face` (whole-pixel advances, Geist's 1 narrower than its 0), `every_mark_has_a_glyph` for both faces |
| Classic unchanged | first-start main window, Classic, against `pr-shots` `03-icons/classic-main-empty*.png`: 0 of 1862 × 998 pixels differ, light and dark |
| Shots | `pr-shots` `04-typeface/` (and every Graphite state, light, in `all/`): main window, Options General, Download File Info, light and dark; before-shots `03-icons/main-plain*`, `02-theme/options-general*`, `02-theme/fileinfo*` (Segoe UI and Tahoma; the dialogs' before-shots predate 03's icons) |
| External audit | two rounds by read-only subagents; round 1: double-encoded characters in `theme.rs`, `BUNDLED` leaking between tests, the switch test bypassing the reload, `numbers` unmapped in 05's plan, `font` missing from docs/themes.md; round 2: the switch test now goes through `fonts_if_changed`, two boxes above qualified, the one-frame mix documented. All fixed |
| Release build | thin LTO for the shots: the fat-LTO profile crashes rustc 1.98.1 here (0xc0000409 in `rustc_driver`), on `dev` without this change too: the toolchain on this machine, not 04 |

## Overflow left for 05b and the 07 waves (Geist, 100 %, IDM's layout; nothing widened here)

- Main window: file names end in "…" a few letters sooner; "Share ratio" touches the list's edge.
- Options, General: "Customize keys to prevent or force downloading with MDM" ends in "…"
  (new); "Customize MDM menu items in context menu of browsers" runs under its Edit… button (as
  with Tahoma before).
- Options, Downloads: "Start downloading immediately while displaying "Download File Info"
  dialog" ends in "…" past the section's rule. Options, Proxy / Socks: "Use proxy/socks from a
  browser's request…" runs past the rules; "Use FTP in PASV mode" sits on the page frame.
  Options, Save to: the paragraph's first line reaches the group's border. Options, Remote: the
  last paragraph sits on the page frame. Column heads "Numb…" (Connection) and "Passw…" (Sites
  Logins).
- Speed limiter: the note wraps with "not" alone on a line. Gallery: "Install gallery-dl" fills
  its button. Scheduler: the "Schedule" tab's title fills its tab. Grabber: "Size" and "Status"
  heads nearly touch. Progress window: "Resume capability" nearly touches its value.
- Every other window of the 56 Graphite states shot (light, 100 %; `pr-shots` `04-typeface/all/`):
  nothing clipped. 200 % was not shot: sizes scale with the zoom, the DirectWrite advances are
  whole pixels at every size.
