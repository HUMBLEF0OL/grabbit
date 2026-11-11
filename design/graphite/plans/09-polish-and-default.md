# 09 · Polish, then make Graphite the default

Adds the finishing touches the theme shipped without (the two-tone hot colour and the dark-mode glow),
does a full review against the mockup, and switches new installs to Graphite.

- Branches: `ui/09a-finish`, `ui/09b-default`
- Needs: 06, 07e, 08b (and through them 05, 07a…07d, 08a; 02b's roles `button.accent.top`,
  `progress.bar.end`, `glow`)
- Unblocks: 10 (the rename's screenshots are taken once, on the final look)

## Facts this plan relies on (paths under `crates/app/src/`, checked on `ui/03-icons` c3541d0)

Line numbers drift as phases land: find each item by its name first.

- Gradients already exist: `winui::fill_shaded(p, rc, radius, stops)` (`winui.rs:1583`) and
  `fill_shaded_corners` (`:1589-1661`) build an `egui::Mesh` of vertical bands with rounded corners;
  `enum Shine { Luna, Aero, Aqua }` (`:1667`) with `stops(face, gloss)`.
- Buttons are painted in `paint_button` (`winui.rs:3639`, `fill_shaded` at `:3682`), colours from
  `button_colors_in` (`:3741`; the accent roles `button.accent` and `button.accent.bottom` at
  `palette.rs:210/213` are read at `winui.rs:3769`); progress bars in `progress_bar` (`:638`) and
  `era_bar` (`:1826`); the title bar in `winui::caption` (`style()` `:5485`, `title_bar` `:5954`).
- Looks are chosen by `ButtonLook` / `ProgressLook` / `CaptionLook` (`palette.rs:469/491/502`) through
  `[controls]` (`Controls::set`, `:546-582`). `ButtonLook::Graphite` and `ProgressLook::Graphite` do not
  exist yet. `button.glow` (`palette.rs:216`) is Luna's glow under the pointer, not a shadow: leave it.
- The theme's id is saved in `settings.theme`, default "classic" (`settings.rs:105-107, 364`).
  `Settings` is `#[serde(default)]`, so a change to `Settings::default()` would also fill a missing
  field in an existing user's settings file.

## 09a · Finish

1. **Two-tone hot colour**: a new `ButtonLook::Graphite` and `ProgressLook::Graphite` (`[controls]`
   values `button = "graphite"`, `progress = "graphite"`, written by `make_theme.py`). For the Accent and
   (under Graphite) Default buttons, `paint_button` fills with two stops (`button.accent.top` at the top,
   `button.accent` at the bottom) through `fill_shaded`; the progress fill runs left to right
   `progress.bar` → `progress.bar.end` (a horizontal variant of `fill_shaded`, or two meshes).
2. **Glow (dark only)**: a soft shadow under the default button and the progress fill in the `glow`
   role (about 35% alpha), drawn with `epaint::Shadow` (blur 12, offset y 4). Graphite light sets `glow`
   transparent, so nothing is drawn.
3. **App mark**: the title bar's small mark (16 px, radius 4) and the About window's (56 px, radius 14)
   as a primary → outlaw gradient tile (`accent` → `button.accent`), until the Grabbit logo from 10b
   replaces them. The extension offer and the video-helpers offer drop `winui::banner`'s logo band under
   Graphite, as the mock draws them.
4. **Cost check**: frame time of the main window with 1,000 rows and 10 running downloads, before and
   after, in Graphite dark. Budget: no more than 5% slower; otherwise ship flat (remove the glow first).
5. **Full review**: every mock state, light and dark, at 100% and 150%, side by side with the mock; a
   list of differences fixed or recorded as follow-ups. It includes:
   - a contrast pass with the real colours (the WCAG tests in `palette.rs` plus a manual check of greyed text);
   - a full keyboard pass (Tab order, Alt and menu letters, Enter and Escape) over every changed window;
   - an accessibility-tree pass over every changed window (names, roles, disabled states, bounds);
   - interface size 200%.

## 09b · Graphite by default

1. **New installs only**: when no settings file exists at start (`store` load finds nothing), the
   fresh settings get `theme = "graphite"`. `Settings::default().theme` stays "classic", so serde never
   switches an existing user. Test both: a missing file starts in Graphite; an existing file without a
   `theme` field keeps Classic.
2. Existing users keep their theme. On first start after the update, a one-time question (a
   `dialogs::question` with Try it / Not now) offers Graphite; "Try it" runs `Cmd::Theme("graphite")`;
   the answer is remembered in a new settings field. This is a new window: say so in the CHANGELOG.
3. **Downgrade**: an older build reading `theme = "graphite"` falls back to Classic (test against the
   last release's settings loader).
4. Website and docs screenshots (`tools/shots/`, `docs/`) retaken in Graphite; `docs/` theme help lists Graphite.
   Left from 02, which kept Graphite off the website until a release has it: with the release that
   ships it, add its tile to `website/content/themes.toml` (and the "Nine looks" line of
   `website/content/features/themes.md`), its `site-theme-graphite` shot to `tools/shots/shots.toml`
   (`take = "theme:graphite+@main"`, as the other themes'), `"graphite"` to the `inherits` list of
   `website/content/theme-prompt.txt` (and `chrome = classic | graphite` to its `[controls]`
   line, from 02b), and Graphite to the themes in `website/llms.txt`.
5. CHANGELOG entry for the new look; ROADMAP's "A new look, 'Graphite'" item marked ✅.

## Done when

- [ ] Graphite's default button and progress show the two-tone finish; dark mode shows the glow; light does not
- [ ] Frame-time budget met
- [ ] Every mock state reviewed (visual, keyboard, accessibility, 200%); follow-ups filed
- [ ] New installs start in Graphite; existing users keep their theme and are offered it once; downgrade falls back
- [ ] Screenshots, docs, website, ROADMAP, CHANGELOG
