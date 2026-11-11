# 02b · The chrome gate and the Graphite roles

The switch every layout phase depends on, and the theme colour roles those phases draw with. It adds a
theme control that says whether windows use IDM's layout or the Graphite layout, adds the new palette
roles, sets both in the Graphite theme, and nothing on screen reads them yet.

- Branch: `ui/02b-chrome-gate`
- Needs: 02
- Unblocks: 05b, 06, 07a, 08, 09a (every phase that changes layout or uses the new roles)

## Why its own phase

05b, 06, 07a and 08 all draw differently "under the Graphite chrome", and the main-window track (05,
06) and the dialog track (07, 08) both need roles the theme does not have yet (a panel fill, danger
buttons, banners). Adding them inside one track would make the other wait for it.

## Steps (paths under `crates/app/src/`)

1. `palette.rs`: `pub enum ChromeLook { Classic, Graphite }` beside the other looks (`ButtonLook`,
   `ProgressLook`, `CaptionLook`); a `chrome: ChromeLook` field on `Controls`, default `Classic`.
2. `Controls::set` accepts `("chrome", "classic")` and `("chrome", "graphite")`.
3. `theme::chrome() -> ChromeLook` for callers, next to the other control getters.
4. New roles in the `palette!` macro, each with today's colours as the default so other themes are unchanged:

   | Role | Used by | Graphite value (from `pal.py`) |
   | --- | --- | --- |
   | `panel` | status bar, toolbar band, footer band, title-bar band (05b, 07a) | `surface` |
   | `tile` | icon tiles, empty-state tiles (05b, 07a) | `tile` |
   | `menu.shortcut` | menu shortcut column, Find's "Ctrl+F" (05b) | `ink3` |
   | `toolbar.checked` | toolbar Details toggle on (06) | `sel` |
   | `button.danger`, `button.danger.text` | Confirm deletion, Delete category (07a) | `dng`, `ondng` |
   | `banner.error`, `banner.info`, `banner.warning` | status banners (07a) | `errSoft`, `sel`, `warnSoft` |
   | `glow` | dark-mode shadow under the default button and progress (09a) | `glow` (transparent in light) |
   | `button.accent.top` | two-tone default button, upper stop (09a) | `out2` |
   | `progress.bar.end` | two-tone progress, end stop (09a) | `out2` |

   `button.glow` stays what it is (Luna's glow under the pointer); the new `glow` is separate.
5. `design/graphite/source/make_theme.py`: write `chrome = "graphite"` in `[controls]` and the roles
   above (light and dark); add each new text pair to its contrast check (`button.danger.text` on
   `button.danger`, `text` on each banner); run it to regenerate `crates/app/assets/themes/graphite.toml`
   (never edit the file by hand).
6. `winui::icon_tile(ui, rc, icon, tint, radius)`: a `tile`-filled square holding a tinted glyph
   (40 px radius 8 in dialogs, 56 px radius 14 for empty states and the About mark); no caller yet,
   so 05b and 07a share one implementation.
7. Theme-file docs (`docs/themes.md`): the `chrome` control and the new roles.

## Tests

- `chrome = "graphite"` parses; an unknown value warns; every built-in except Graphite is `Classic`.
- `built_in_theme_files_read_cleanly` still passes with the new key and roles.
- The WCAG test in `palette.rs` covers `button.danger.text` on `button.danger`.

## Done when

- [ ] Graphite reports `ChromeLook::Graphite`, every other theme `Classic`; the new roles exist with
      defaults for every theme and Graphite's values from `pal.py`
- [ ] Nothing on screen changes (one Classic and one Graphite pair show it; no CHANGELOG line)
- [ ] docs/themes.md lists the key and roles
