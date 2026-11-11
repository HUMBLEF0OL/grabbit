# 02 · The Graphite theme (colours, corners, controls)

Adds a built-in theme "Graphite" in the chosen palette, light and dark, selectable under View, Theme.
Colours and shapes only: no new icons (03), no font (04), no layout change (05+).

- Branch: `ui/02-theme`
- Needs: 01 (palette chosen, draft `design/graphite/assets/graphite.toml`, DESIGN.md)
- Unblocks: 02b, 03, 04, 05a

## Facts this plan relies on (paths under `crates/app/`)

- A theme file is parsed by `themefile::parse` (`src/themefile.rs:423-497`). Top-level keys: `name`,
  `inherits` (a built-in may inherit only `classic`), `seeds`, `contrast`, `icon-set`, `icon-mono`,
  `icons`, and the tables `sizes`, `controls`, `colors`, `light`, `dark`. An unknown key warns, and
  `builtin()` asserts no warnings, so a typo fails the tests.
- Seeds (`themefile.rs:502-554`): background, surface, header, text, accent, error, warning, success,
  selection. Any seed re-derives its whole variant (`derive::derive`); hand-set roles then override.
- Roles in `[light]`/`[dark]`: the `palette!` macro (`src/palette.rs:146-325`). The ones this theme sets
  are listed in step 3.
- `[controls]` values (`Controls::set`, `palette.rs:546-582`): field-focus line/border/ring; check
  fluent/square/…; button fluent/breeze/…; progress line/bar/blocks/gloss; scroll overlay/classic;
  caption system/metro/…; focus dotted/ring/frame.
- `[sizes]` (`metrics!`, `palette.rs:363-390`): radius, radius.button, radius.part, radius.check,
  radius.flyout, radius.menu-item, radius.egui, radius.popup, border, focus-line, list.row, list.header, font.*.
- Built-ins: `palette.rs:625-661` `builtins() -> [&'static Theme; 8]`, order CLASSIC, HIGH_CONTRAST,
  METRO, ADWAITA, BREEZE, LUNA, AERO, AQUA. Tests index by position (`palette.rs:875-935`,
  `native.rs:112,132`), so the new theme is **appended** as index 8.
- View, Theme is hand-written: `app.rs:5727` `theme_menu`; the handler is `app.rs:5027` `Cmd::Theme` (as of c3541d0).
- The choice is saved as `settings.theme` (`settings.rs:105-107`); screenshot automation takes `theme:<id>` (`app.rs:2124`, as of c3541d0).
- "graphite" is free as an id (only comments mention it).

## Steps

1. **Theme file** `assets/themes/graphite.toml`, from the draft in `design/graphite/assets/`. Header
   comment in the style of `metro.toml` (what it is, what it borrows: nothing; icons from 03 later).
   ```toml
   name = "Graphite"
   icons = "mono"            # until 03 adds icon-set = "graphite"

   [controls]
   field-focus = "ring"
   focus = "ring"
   check = "fluent"
   button = "fluent"
   progress = "line"
   scroll = "overlay"
   caption = "metro"         # a drawn, flat title bar coloured by the caption roles

   [sizes]
   radius = 8
   "radius.button" = 8
   "radius.part" = 6
   "radius.check" = 4
   "radius.flyout" = 10
   "radius.menu-item" = 6
   "radius.popup" = 8
   "radius.egui" = 8
   "list.row" = 32           # the mockup's row height; check the list still fits at 200%
   ```
2. **Seeds** `[seeds.light]` and `[seeds.dark]` from the palette: background = `bg`, surface = `surface`,
   header = `surface`, text = `ink`, accent = `acc`, selection = `sel`, error = `err`, warning = amber,
   success = `okt`.
3. **Roles set by hand** (both variants), from `design/graphite/source/pal.py`:
   - Text: `text.disabled` = `dis`, `text.muted` = `ink3`, `link` = `acct`, `focus.ring` = `ring`
   - Selection: `selection.text` = `ink`, `selection.inactive` = `hover`
   - Fields: `field` = `field`, `field.border` and `field.bottom` = `line`, `field.disabled` = `tile`
   - Buttons: `button` = `surface`, `button.border`/`button.bottom` = `line`, `button.hot` = `hover`,
     `button.pressed` = `press`, `button.accent` = `out`, `button.accent.pressed` = `outp`, `button.accent.text` = `onout`,
     `button.accent.hot` = `out2`, `button.default.border` = `out`
   - Checks: `check.on` = `acc`, `check.mark` = `onacc`, `check.border` = `ctl`, `check.border.disabled` = `dis`
   - Progress: `progress.track` and `meter.track` = `track`, `progress.bar` and `meter.fill` = `out`,
     `segments.done` = `out`, `segments.missing` = `track`, `pieces.have` = `out`, `pieces.missing` = `track`
   - Lists: `list` = `bg`, `header` = `bg`, `header.line` = `line`, `grid`/`list.grid` = `line2`,
     `chip.web` = `chip`, `chip.text` = `chipt`, `chip.torrent` = `tile`
   - Menus: `menubar` = `surface`, `menubar.line` = `line`, `menu` = `surface`, `menu.hot` = `sel`,
     `menu.border` = `line`, `menu.text.disabled` = `dis`
   - Toolbar: `toolbar.hot` = `hover`, `toolbar.pressed` = `press`, `toolbar.text.disabled` = `dis`, `icon.disabled` = `dis`
   - Tree: `tree` = `surface`, `tree.expander` = `ink3`
   - Caption: `caption` = `surface`, `caption.text` = `ink`, `caption.text.inactive` = `ink3`,
     `caption.hot` = ink at 6% alpha, `caption.pressed` = ink at 12%
4. **Register**: in `palette.rs` add `static GRAPHITE` (same pattern as AQUA), change the array to
   `[&'static Theme; 9]` and append `&GRAPHITE`. Add `("graphite", "Graphite", include_str!(…))` to
   `built_in_theme_files_read_cleanly` (`palette.rs:838-862`).
5. **Menu**: in `app.rs` `theme_menu`, add `Radio("Graphite", Cmd::Theme("graphite"), true, on("graphite"))`
   after Aqua, with the comment convention the file uses now.
6. **Tests** (in `palette.rs` tests): a `graphite_is_built_in` test checks index 8 is "graphite", radius 8,
   focus ring, caption Metro. Known from 01: the step 3 mapping makes pressed equal to hot (`hover` and
   `tile` are both `#1c1828` in dark, one step apart in light: issue #7's problem), and
   `button_states_differ` never compares the two. Give pressed its own value in `pal.py` (a deeper tile)
   and add that comparison. The existing whole-set tests already cover it: WCAG contrast
   (`palette.rs:748`), focus ring 3:1 (`:810`), no bright menu-bar line in dark (`:970`),
   `winui.rs:6858` pressed buttons. Fix any colour they reject in the theme file, not in the tests.

## Mock states to compare

Main window (details pane off), Add URL, Download File Info, progress window (status), Options
(General), Scheduler (Schedule), Confirm deletion: light and dark. Layout will still be the old
layout; check colours, corners, focus ring, checkboxes, progress, title bar and menus.

## Not in this phase

Icons (03), Geist (04), gradients and glow (09: the theme ships flat), layout (05+), making
Graphite the default theme (decided in 01; switched for new installs in 09b).

## Done when

- [ ] View, Theme lists Graphite; choosing it restyles every window live; the choice survives a restart
- [ ] All palette and winui tests pass, including the new one
- [ ] Screenshots of the states above, before (Classic) and after (Graphite), light and dark
- [ ] ROADMAP item "Graphite theme" done; CHANGELOG line
