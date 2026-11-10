# Grabbit UI revamp: the plan

As of 2026-11-10. The approved mockups are the spec: `grabbit-design.html` in this folder (open it in
a browser; same page as the published artifact, version 10). The detailed plans are in `plans/`, one
per phase, in the order they are built. Start with `plans/00-conventions.md`.

## Decisions

| Decision | State | Note |
| --- | --- | --- |
| Layout direction: Graphite | Decided | IDM layout kept; paste bar and details pane added, both optional |
| Icon set: Tabler outline, stroke 1.75 | Decided | 53 icon slots mapped, MIT licence |
| Default button last in every window | Decided | Under the Graphite look; Classic keeps IDM's order |
| App name: Grabbit | Decided in the mockups | Renaming code, installer, extension and website is phase 10 |
| Colour direction | **Open** (blocks 02) | Recommended: jet black, royal amethyst, electric cyan, dark first |
| Typeface: Geist | Proposed | Bundled (SIL OFL); needs engine work (04) |
| Paste bar and details pane defaults | Open | Recommended: paste bar on, details pane off |
| The two proposal dialogs | Open | Recommended: not built now |
| Graphite as the default theme | Open | Recommended: after 09, new installs only |

## Build order

| # | Plan | What it delivers | Needs | Branch(es) |
| --- | --- | --- | --- | --- |
| 00 | [Conventions](plans/00-conventions.md) | How every phase is branched, checked, screenshotted and documented | none | none |
| 01 | [Prepare](plans/01-prepare.md) | Decisions, the 106 icon files, the draft theme file, Geist files, DESIGN.md, baseline screenshots | none | `ui/01-prepare` |
| 02 | [Theme](plans/02-theme.md) | Built-in "Graphite" theme: colours, corners, controls, light and dark | 01 | `ui/02-theme` |
| 03 | [Icons](plans/03-icons.md) | The Tabler icon set for all 53 slots | 01, 02 | `ui/03-icons` |
| 04 | [Typeface](plans/04-typeface.md) | Themes can choose a bundled font; Graphite uses Geist | 01, 02 | `ui/04-typeface` |
| 05 | [Main window](plans/05-main-window.md) | One layout model (05a); then menus in the title bar, grouped toolbar, flatter categories, progress in the Status cell (05b) | 02, 03, 04 | `ui/05a-layout-model`, `ui/05b-main-window` |
| 06 | [Paste bar and details pane](plans/06-paste-bar-details-pane.md) | The two optional panes, toggled in View | 05 | `ui/06-panes` |
| 07 | [Dialogs](plans/07-dialogs.md) | The dialog kit (07a), then every dialog in four waves (07b to 07e) | 02 (04 for sizes) | `ui/07a-dialog-kit`, `ui/07b…07e` |
| 08 | [Options](plans/08-options.md) | Options pages as a side list, sub-dialogs restyled | 07a | `ui/08-options` |
| 09 | [Polish and default](plans/09-polish-and-default.md) | Two-tone hot colour, dark glow, full review (09a); Graphite for new installs (09b) | 05 to 08 | `ui/09a-finish`, `ui/09b-default` |
| 10 | [Rename to Grabbit](plans/10-rename-grabbit.md) | Display text, assets, migrated identifiers, distribution | 01 decision; best after 09 | `rename/10a…10d` |

What can run side by side: after 02, the tracks **03 → 04 → 05 → 06** (main window) and **07a → 07b…07e
→ 08** (dialogs) are independent; they meet at 09. The rename (10) touches everything, so it waits.

```
01 ─► 02 ─┬─► 03 ─► 04 ─► 05a ─► 05b ─► 06 ─┐
          │                                  ├─► 09a ─► 09b ─► 10a ─► 10b ─► 10c ─► 10d
          └─► 07a ─┬─► 07b ─► 07c ─► 07d ─► 07e ┤
                   └─► 08 ──────────────────────┘
```

## Progress

- [ ] 01 Prepare
- [ ] 02 Theme
- [ ] 03 Icons
- [ ] 04 Typeface
- [ ] 05a Layout model
- [ ] 05b Main window
- [ ] 06 Paste bar and details pane
- [ ] 07a Dialog kit
- [ ] 07b Adding
- [ ] 07c Watching and finishing
- [ ] 07d Torrents, video and more
- [ ] 07e Organising and the rest
- [ ] 08 Options
- [ ] 09a Finish
- [ ] 09b Default
- [ ] 10a to 10d Rename

## What is in this folder

| Path | What it is |
| --- | --- |
| `PLAN.md` | This index |
| `plans/` | One plan per phase, with file and line references taken from the code on 2026-11-10 (re-check them when starting a phase: `dev` moves) |
| `grabbit-design.html` | The mockup page: 60 windows, 54 states, light, dark, right to left, 4 palettes |
| `source/pal.py` | The 4 palettes, every token in light and dark, with the contrast check (`python pal.py`) |
| `source/iconmap.py` | Each of the 53 icon slots to its Tabler icon name; `gen3.py` holds the review changes (`REFINE`) |
| `source/graphite.html`, `source/gen3.py` | The page template and the script that builds the mockup (update the `S` paths before reuse) |
| `vendor/` | Tabler Icons 3.31.0 package and licence |

## Risks across phases

- **Code references drift.** The plans cite lines from 2026-11-10; `dev` is changing (English only done;
  Windows only done (6d9d5f8, a75224e, 051c207); the old state is on `archive/multiplatform-i18n`).
  Re-find each reference before starting a phase.
- **The main window's layout numbers are duplicated** (05a exists to fix that before anything moves).
- **Dialogs are laid out in absolute pixels** with per-dialog button wiring (07a's kit exists so each
  dialog changes once, in a small diff).
- **Fonts**: Windows draws Latin text through a DirectWrite path tied to system font files (04 must
  change it, or Geist never shows).
- **The rename moves users' data and integrations** (saved logins, autostart, native messaging host,
  update feed); 10c's migrations and the never-change list exist for that.
- **Accessibility**: the toolbar and status strip are not exposed today; 05 adds the toolbar, and every
  layout change re-checks the accessibility tree.
