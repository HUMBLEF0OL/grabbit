# 09 · Polish, then make Graphite the default

Adds the finishing touches the theme shipped without (the two-tone hot colour and the dark-mode glow),
does a full review against the mockup, and switches new installs to Graphite.

- Branches: `ui/09a-finish`, `ui/09b-default`
- Needs: 05, 06, 07, 08
- Unblocks: 10 (the rename's screenshots are taken once, on the final look)

## Facts this plan relies on (paths under `crates/app/src/`)

- Gradients already exist: `winui::fill_shaded(p, rc, radius, stops)` (`winui.rs:1583`) and
  `fill_shaded_corners` (`:1589-1661`) build an `egui::Mesh` of vertical bands with rounded corners;
  `enum Shine { Luna, Aero, Aqua }` (`:1666`) with `stops(face, gloss)`.
- Buttons are painted in `paint_button` (`winui.rs:3639`, `fill_shaded` at `:3682`), colours from
  `button_colors_in` (`:3741`); progress bars in `progress_bar` (`:638`) and `era_bar` (`:1826`); the
  title bar in `winui::caption` (`style()` `:5512`, `title_bar` `:5982`).
- Looks are chosen by `ButtonLook` / `ProgressLook` / `CaptionLook` (`palette.rs:469/491/502`) through
  `[controls]` (`Controls::set`, `:546-582`). The palette already has `button.glow` and `gloss` roles.
- The theme's id is saved in `settings.theme`, default "classic" (`settings.rs:105-107, 364`).

## 09a · Finish

1. **Two-tone hot colour**: a new `ButtonLook::Graphite` and `ProgressLook::Graphite` (and `[controls]`
   values `button = "graphite"`, `progress = "graphite"`). For the Accent/Default button, `paint_button`
   fills with two stops (`out2` at the top, `out` at the bottom) through `fill_shaded`; progress fills
   left to right `out` → `out2` (a horizontal variant of `fill_shaded`, or two meshes).
2. **Glow (dark only)**: a soft shadow under the default button and progress fill in the hot colour at
   about 35% alpha, drawn with `epaint::Shadow` (blur 12, offset y 4). Light variants set the glow colour
   transparent, so nothing is drawn. Use the existing `button.glow` role for the colour.
3. **App mark**: the title bar's small mark as a primary → outlaw gradient tile (until the Grabbit logo
   from 10b replaces it).
4. **Cost check**: frame time of the main window with 1,000 rows and 10 running downloads, before and
   after, in Graphite dark. Budget: no more than 5% slower; otherwise ship flat (remove the glow first).
5. **Full review**: every mock state, light and dark, at 100% and 150%, side by side with the mock; a
   list of differences fixed or recorded as follow-ups. Include a contrast pass with the real colours
   (the WCAG tests in `palette.rs` plus a manual check of greyed text).

## 09b · Graphite by default

1. New installs: `Settings::default().theme = "graphite"`.
2. Existing users keep their theme. On first start after the update, a one-time notice (a
   `dialogs::notice` with Try it / Not now) offers Graphite; "Try it" runs `Cmd::Theme("graphite")`.
   Remember the answer in settings.
3. Website and docs screenshots (`tools/shots/`, `docs/`) retaken in Graphite; `docs/` theme help lists Graphite.
4. CHANGELOG entry for the new look; ROADMAP item done.

## Done when

- [ ] Graphite's default button and progress show the two-tone finish; dark mode shows the glow; light does not
- [ ] Frame-time budget met
- [ ] Every mock state reviewed; follow-ups filed
- [ ] New installs start in Graphite; existing users are offered it once
- [ ] Screenshots, docs, ROADMAP, CHANGELOG
