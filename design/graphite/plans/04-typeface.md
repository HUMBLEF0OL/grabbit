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

- [ ] Graphite draws in Geist on Windows; every other theme still draws in Segoe UI (Classic pixel-identical)
- [ ] Switching themes swaps the face without a restart
- [ ] No clipped labels in the main window; overflow list for later phases written in the PR
- [ ] Digits decision recorded; size and frame-time budgets met
- [ ] Screenshots before and after (main window, Options, Download File Info), light and dark
- [ ] ATTRIBUTION, ROADMAP, CHANGELOG updated
