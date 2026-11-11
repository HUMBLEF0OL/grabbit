# Grabbit UI revamp: the plan

As of 2026-11-11 (audited: code references checked against `ui/03-icons` c3541d0). The approved
mockups are the spec: `grabbit-design.html` in this folder (open it in a browser; same page as the
published artifact, version 10). The detailed plans are in `plans/`, one per phase, in the order they
are built. Start with `plans/00-conventions.md`.

## Decisions

Rows marked "Decided in 01" took the recommendation in `plans/01-prepare.md` section 1 on
2026-11-10; changing one means changing the phases it blocks before they start. Open rows name the
phase that must decide them before it starts.

| Decision | State | Note |
| --- | --- | --- |
| Layout direction: Graphite | Decided | IDM layout kept; paste bar and details pane added, both optional |
| Icon set: Tabler outline, stroke 1.75 | Decided | 53 icon slots (03) plus 36 extra `gx-*` slots carried only by the Graphite set (03b) |
| Default button last in every window | Decided | Under the Graphite look, filled; Classic keeps IDM's order. Keys (Enter, Escape) never move (07a) |
| App name: Grabbit | Decided in the mockups | Renaming code, installer, extension and website is phase 10 |
| Colour direction: jet black, royal amethyst, electric cyan | Decided in 01 | `source/pal.py` `amethyst`; the theme `crates/app/assets/themes/graphite.toml` (02) |
| Light or dark: follow the system | Decided in 01 | Today's behaviour; the palette is designed dark first (02) |
| Graphite as the default theme | Decided in 01 | Opt-in under View, Theme from 02; the default for new installs (no settings file) in 09b; existing users keep their theme and are offered it once |
| Typeface: Geist | Decided in 01 | Bundled (SIL OFL), Regular, Medium, SemiBold in `assets/fonts/`; needs engine work (04) |
| Paste bar and details pane defaults | Decided in 01 | Paste bar on, details pane off, width 276, for new and existing users (06); both only under the Graphite look |
| The two proposal dialogs | Decided in 01 | Not built; Advanced / Socks... and Virus checking settings stay greyed as today (08) |
| When to rename to Grabbit | Decided in 01 | Last (10): it touches data paths and the extension |
| Tray menu | Decided (audit) | Stays native (`tray_icon`): the mock's tray menu is not built (07e) |
| Safari steps, Linux and macOS wake lines | Decided (audit) | Not built: `dev` is Windows only (07e) |
| Options: a Cancel button under Graphite | **Open, owner 08** | Decide before 08a; recommended: none (Escape and the close box already discard changes, `options.rs:3`) |
| Tabular digits: Geist's own or Geist Mono | **Open, owner 04** | Decide in 04 step 7; default: Geist Mono not bundled |
| Which repository the app points at (updates, crash reports); exe and crate names | **Open, owner 10** | Decide before 10a; recommended: keep names, change repository constants only when that repository publishes Grabbit releases |

## Build order

| # | Plan | What it delivers | Needs | Branch(es) |
| --- | --- | --- | --- | --- |
| 00 | [Conventions](plans/00-conventions.md) | How every phase is branched, checked, screenshotted and documented; pushes go to the fork only | none | none |
| 01 | [Prepare](plans/01-prepare.md) | Decisions, the 106 icon files, the draft theme file, Geist files, DESIGN.md, baseline screenshots | none | `ui/01-prepare` |
| 02 | [Theme](plans/02-theme.md) | Built-in "Graphite" theme: colours, corners, controls, light and dark | 01 | `ui/02-theme` |
| 02b | [Chrome gate and roles](plans/02b-chrome-gate.md) | `[controls] chrome` (Classic / Graphite) and the new roles (panel, danger, banners, glow, two-tone stops…); nothing on screen changes | 02 | `ui/02b-chrome-gate` |
| 03 | [Icons](plans/03-icons.md) | The Tabler icon set for all 53 slots | 01, 02 | `ui/03-icons` |
| 03b | [Extra glyphs](plans/03b-extra-glyphs.md) | 36 `gx-*` slots (menu, button, Options page, queue icons) in the Graphite set | 03 | `ui/03b-extra-glyphs` |
| 04 | [Typeface](plans/04-typeface.md) | Themes can choose a bundled font; Graphite uses Geist; digits decided | 01, 02 | `ui/04-typeface` |
| 05 | [Main window](plans/05-main-window.md) | One layout model (05a); then title-bar menus and Find, menu icons and shortcuts, grouped toolbar, flatter categories, progress in the Status cell (05b) | 05a: 02. 05b: 02b, 03, 03b, 04, 05a | `ui/05a-layout-model`, `ui/05b-main-window` |
| 06 | [Paste bar and details pane](plans/06-paste-bar-details-pane.md) | The two optional panes, toggled in View and the toolbar | 05a, 05b (and 02b, 03b through 05b) | `ui/06-panes` |
| 07 | [Dialogs](plans/07-dialogs.md) | The dialog kit (07a: footer, keys as today, danger/ghost/icon buttons, form, file card, icon tile, banners), then every dialog in four waves (07b to 07e) | 02b, 03, 03b; 04 before final sizes | `ui/07a-dialog-kit`, `ui/07b…07e` |
| 08 | [Options](plans/08-options.md) | Side list with pages unchanged inside (08a), then pages on the form and sub-dialogs (08b) | 07a, 02b, 03b; 04 before final sizes | `ui/08a-options-list`, `ui/08b-options-pages` |
| 09 | [Polish and default](plans/09-polish-and-default.md) | Two-tone hot colour, dark glow, app mark, full visual/keyboard/accessibility review (09a); Graphite for new installs (09b) | 06, 07e, 08b | `ui/09a-finish`, `ui/09b-default` |
| 10 | [Rename to Grabbit](plans/10-rename-grabbit.md) | Display text, assets, migrated identifiers, distribution | 09b; the logo (maintainer) before 10b | `rename/10a…10d` |

Tracks:

```
Foundation   01 ─► 02 ─┬─► 02b
                       ├─► 03 ─► 03b
                       ├─► 04
                       └─► 05a

Main window  [02b 03 03b 04 05a] ─► 05b ─► 06 ──────────────────────┐
                                                                     ├─► 09a ─► 09b ─► 10a ─► 10b ─► 10c ─► 10d
Dialogs      [02b 03 03b] ─► 07a ─┬─► 07b ─► 07c ─► 07d ─► 07e ──────┤
                                  └─► 08a ─► 08b ───────────────────┘
```

What can run side by side: once 02b, 03 and 03b are in, the main-window track (05b, 06) and the dialog
track (07, 08) are independent; 04 must land before 05b and before the 07 waves and 08 take their final sizes.
05a can start right after 02. The rename (10) touches everything, so it waits for 09b.

## Progress

A phase is ticked when its PR is merged into `dev` on origin (00, Paperwork).

- [x] 01 Prepare
- [ ] 02 Theme (in origin's `dev` since 2026-11-11, pushed directly, no PR; its Done-when list is
      still open)
- [ ] 02b Chrome gate and roles
- [x] 03 Icons (complete, in origin's `dev` since 2026-11-11, pushed directly instead of a PR;
      checks in its plan)
- [ ] 03b Extra glyphs
- [ ] 04 Typeface
- [ ] 05a Layout model
- [ ] 05b Main window
- [ ] 06 Paste bar and details pane
- [ ] 07a Dialog kit
- [ ] 07b Adding
- [ ] 07c Watching and finishing
- [ ] 07d Torrents, video and more
- [ ] 07e Organising and the rest
- [ ] 08a Options side list
- [ ] 08b Options pages
- [ ] 09a Finish
- [ ] 09b Default
- [ ] 10a to 10d Rename

## What is in this folder

| Path | What it is |
| --- | --- |
| `PLAN.md` | This index |
| `plans/` | One plan per phase. File and line references were checked against `ui/03-icons` c3541d0 on 2026-11-11; they drift as phases land, so each plan names the function too: re-find it before starting |
| `grabbit-design.html` | The mockup page: 60 windows, 54 states, light, dark, right to left (not built: the app is English only), 4 palettes (only `amethyst` is built) |
| `DESIGN.md` | The design rules the build phases follow: colour jobs, radii, sizes, type, icons |
| `assets/icons/` | The 53 icons at 16 and 24 (106 SVGs), `SOURCES.txt`, Tabler's `LICENSE` (03); 03b adds the `gx-*` slots |
| `assets/fonts/` | Geist Regular, Medium, SemiBold (TTF), `OFL.txt`, `SOURCES.txt` (for 04) |
| `source/export_icons.py` | Writes `assets/icons/` from the Tabler package, `iconmap.py` and `gen3.py`'s `REFINE` (03b extends it with `EXTRA`) |
| `source/make_theme.py` | Writes the app's theme file `crates/app/assets/themes/graphite.toml` from `pal.py` and checks its contrast (02b adds the chrome key and new roles) |
| `baseline/` | Today's app in every mock state, light and dark (not committed; on origin's `pr-shots` as `baseline/`) |
| `source/pal.py` | The 4 palettes, every token in light and dark, with the contrast check (`python pal.py`) |
| `source/iconmap.py` | Each of the 53 icon slots to its Tabler icon name; `gen3.py` holds the review changes (`REFINE`) and the extras (`EXTRA`) |
| `source/graphite.html`, `source/gen3.py` | The page template and the script that builds the mockup (update the `S` paths before reuse) |
| `vendor/` | Tabler Icons 3.31.0 package and licence |

## Risks across phases

- **Code references drift.** `dev` is English only and Windows only (the old state is on
  `archive/multiplatform-i18n`); each landed phase moves line numbers (01 and 02 moved `app.rs` by 85 to
  120 lines). Re-find each reference by name before starting a phase.
- **The main window's layout numbers are duplicated** (05a exists to fix that before anything moves).
- **Dialogs are laid out in absolute pixels** with per-dialog key wiring (07a's kit exists so each
  dialog changes once, in a small diff, and keeps its keys).
- **Fonts**: Windows draws Latin text through a DirectWrite path tied to system font files (04 must
  change it, or Geist never shows).
- **Classic must not move**: every layout phase gates on 02b's chrome control and shows Classic
  pixel-identical in its PR.
- **The rename moves users' data and integrations** (saved logins, autostart, native messaging host,
  update feed); 10c's migrations and the never-change list exist for that.
- **Accessibility**: the toolbar and status strip are not exposed today; 05b adds the toolbar, and 09a
  re-checks the whole accessibility tree.
- **Pushes**: everything goes to origin, `HUMBLEF0OL/grabbit`, never to the upstream repository (00).
