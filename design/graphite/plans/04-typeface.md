# 04 · Geist, chosen by the theme

Lets a theme name a bundled font, bundles Geist (SIL OFL), and makes Graphite use it. This is the one
look phase that changes engine code: today no font is bundled and a theme cannot choose one.

- Branch: `ui/04-typeface`
- Needs: 01 (Geist files and OFL.txt), 02
- Unblocks: 05 (layout sizes are tuned with the final font)

## Facts this plan relies on (paths under `crates/app/`)

- Fonts are read from the system. `theme::ui_font()` (`src/theme.rs:567-576`) loads segoeui.ttf on
  Windows; `theme::fonts()` (`:643-695`) builds `FontDefinitions` ("segoe-ui" first, then
  `winui::install_fonts` families tahoma, tahoma-bold, arial-bold from `winui.rs:57-80`, then fallbacks
  and the `ScriptFallback` provider), then `set_fonts` and `text_styles`.
- On Windows, `classic_text` (`theme.rs:702`) installs a DirectWrite rasterizer, `win32::text_rasterizer`
  (`win32.rs:4441`), at `FontPriority::Highest` for Latin, Greek and Cyrillic. It maps families to system
  font files in `classic_font_file` (`win32.rs:476-487`). **Anything put in `FontDefinitions` is still drawn
  as Segoe UI for those scripts** unless this mapping learns about bundled fonts.
- Fonts load once in `theme::apply` (`theme.rs:750`); a theme switch (`apply_theme`, `:767`, from
  `Cmd::Theme` at `app.rs:5124`) only re-runs `text_styles`.
- A theme has `font.*` size metrics but no font family. `Theme` is defined at `palette.rs:587`.
- Interface size is egui `zoom_factor` (`src/scale.rs`), independent of the font.

## Steps

1. **Bytes**: `crates/app/assets/fonts/geist/` with Geist-Regular, Geist-Medium, Geist-SemiBold and
   `OFL.txt`; `include_bytes!` them in `theme.rs` behind a small `bundled_font(name) -> Option<&'static [u8]>`.
2. **Theme field**: `Theme { font: Option<Cow<'static, str>> }`; `themefile::parse` accepts a top-level
   `font = "geist"` (unknown names warn and fall back to the system font). Other themes: `None`.
3. **Font definitions**: in `fonts()`, when the selected theme has a font, insert it as the first
   Proportional family (keep the system font and all fallbacks after it, so other scripts still draw).
   The bold family ("tahoma-bold" equivalents) maps to Geist-SemiBold for the theme.
4. **Windows rasterizer**: teach `classic_font_file` / `text_rasterizer` to draw bundled faces: either
   (a) DirectWrite loads the bundled bytes through a custom font-file loader, or (b) the rasterizer is
   skipped for a bundled family and egui's own rasterizer draws it. Prototype both; pick by legibility at
   100% (Geist's hinting) and by code size. Record the choice in the PR.
5. **Reload on theme change**: `Cmd::Theme` re-runs `fonts(ctx)` when the font changes (not on every
   switch), so choosing Graphite or leaving it swaps the face live.
6. **Sizes**: Geist runs wider than Segoe UI. Set Graphite's `font.ui`, `font.body`, `font.small`,
   `font.heading` so the main window's labels fit as in the mock (11.5 toolbar labels, 13 body, 12
   column heads). Check every dialog for clipped labels at 100% and 200%; widen nothing in this phase:
   list any overflow for 05 and 07.
7. `graphite.toml`: `font = "geist"` (through `make_theme.py`'s `HEADER`, which writes the file).
   `ATTRIBUTION.md`: Geist, Vercel, SIL OFL 1.1. Note the added installer size in the PR.

## Tests

- `themefile` test: `font = "geist"` parses; an unknown font warns.
- A test that a theme switch from Classic to Graphite and back changes the first Proportional family.
- Existing text-rendering tests (sharp text, script fallback) still pass.

## Done when

- [ ] Graphite draws in Geist on Windows; every other theme still draws in Segoe UI
- [ ] Switching themes swaps the face without a restart
- [ ] No clipped labels in the main window; overflow list for later phases written in the PR
- [ ] Screenshots before and after (main window, Options, Download File Info), light and dark
- [ ] ATTRIBUTION, ROADMAP, CHANGELOG updated
