# Graphite: the design rules

The rules phases 02 to 09 build to, taken from the mockup (`grabbit-design.html`; its styles are
in `source/graphite.html`). Where this file and the mockup disagree, the mockup wins and this
file is corrected. Sizes are in points at 100 % interface size (CSS pixels in the mockup).

Not to be confused with the repository's root `DESIGN.md`, which is the website's visual system.

## Colour

The palette is "Jet black, royal amethyst, electric cyan" (`source/pal.py`, `amethyst`), light
and dark, designed dark first. Each token has one job:

| Token | Job |
| --- | --- |
| `bg`, `surface`, `field`, `tile`, `hover`, `press` | Window, panels (title bar, toolbar, side lists, footers), text boxes, icon tiles, pointer-over faces, pressed faces (a step deeper than `hover`, never the same) |
| `line`, `line2` | Borders and dividers; the faint row lines in lists |
| `ink`, `ink2`, `ink3` | Text: body, secondary (toolbar labels, field labels), muted (column heads, hints, counts) |
| `dis` | Every greyed thing: text, icons, check-box borders |
| `ctl` | Check-box and radio borders at rest |
| `acc`, `acct`, `onacc`, `sel`, `ring` | The primary colour: what you chose (selection, checked boxes, focus, links), its text, the mark on it, the selected row, the focus halo |
| `out`, `out2`, `outp`, `onout`, `glow` | The outlaw colour: what is moving or what you will do (progress, the default button), its lighter end, its pressed face, the text on it, the dark-only glow |
| `err`, `dng`, `ondng`, `errSoft`, `warnSoft` | Errors (their own hue), the danger button and its text, error and warning backgrounds |
| `okt`, `chip`, `chipt` | Success text; the Web chip and its text |

The app's theme file `crates/app/assets/themes/graphite.toml` maps these to its theme keys
(written by `source/make_theme.py`, which also checks the contrast). Two deliberate differences from the mockup:

- **Focus ring**: the mockup draws focus as an `acc` border with a 3 px `ring` halo (25 % light,
  33 % dark). The app draws a ring alone, and needs it seen at 3:1, so `focus.ring` is `acc` at
  63 % alpha.
- **Warning text**: the palette has only `warnSoft`; the text is amber, `#9d5d00` light and
  `#f5a524` dark (4.9:1 and 9.8:1 on the window).

Gradients and the glow are phase 09's; until then the theme is flat (`out` alone).

## Shape

- Corner radii: **6** items (menu rows, side-list rows, tree nodes, the find button), **8**
  buttons, text boxes, boxes, toolbar buttons, menus, **10** windows and the drop target. Check
  boxes and tags 4; radios round; chips fully round.
- The draft theme sets `radius.flyout` (a calendar or filter popup inside a window) to 10, as
  plan 02 says; the mockup's only popups are menus, at 8.
- Borders 1 px, `line`. Check boxes and radios 16 × 16 with a 1.5 px border.
- Meters: 6 high with radius 3 in windows; 4 high in a list's Status cell. Segment bars 8 high,
  radius 2, 2 apart.

## Sizes and spacing

| Part | Size |
| --- | --- |
| Title bar | 36 high, 12 from the start edge; caption buttons 42 × 35 |
| Buttons and text boxes | 30 high; buttons 14 padding each side, an icon button 30 × 30; text boxes 10 |
| Paste bar's box | 32 high |
| List rows | 32 high, 10 padding each side; column heads 7 above and below |
| Toolbar buttons | 60 wide at least (8 padding each side), grouped, a 1 px `line` divider between groups; the band on `panel` |
| Categories pane | 224 wide by default, rows 28, chevrons, no caption bar |
| Details pane | 276 wide |
| Icon tiles | 40 px, radius 8 in dialogs; 56 px, radius 14 for empty states and the About mark |
| Title bar | 36 high, window buttons 42 x 35 |
| Label column | 112; 240 for sentence-length labels; 12 between label and field, 10 between rows |
| Key and value lists | 132 label column (96 in the details pane), 12 across, 7 down |
| Dialog body | 16 padding, 14 between blocks |
| Footer | 12 above and below, 16 at the sides, 8 between buttons, a `line` above it on `surface` |

## Buttons and footers

- A footer holds secondary buttons at the start edge, then a gap, then the main ones at the end.
- The default button is **last** in every footer, filled with the outlaw colour (`out`, text
  `onout`), its border `out`. Other buttons are `surface` with a `line` border. A footer with
  one button puts it at the end, like the others; only the generic notice (`dialogs::notice`)
  centres its OK.
- This order holds under Graphite only; Classic keeps IDM's order.
- Keys never move with the order: Enter and Escape do what they did before in each window (plan 07a).

## Greyed

One colour, `dis`, for greyed text, icons and check-box and radio borders (a greyed checked box
is filled `dis`). Greyed text boxes are `tile` with a `line2` border. No opacity tricks.

## Status

Colour never alone: every state except downloading shows its icon (16) and its word (Complete,
Waiting, Error…).
In the list's Status cell, a download that is moving shows a 4 high meter and its percentage
instead; a stopped one shows the icon, the word, then the meter and the percentage.

## Icons

Tabler outline, stroke **1.75**, tinted with the theme's ink (`assets/icons/`, `SOURCES.txt`).
The app carries the same files as the Graphite icon set (`crates/app/assets/themes/graphite/icons/`);
`source/export_icons.py` writes both.

- **20** in the toolbar, **16** in lists, categories and menus.
- The mockup draws icons inside buttons, fields and menu rows at 15; the set is exported at 16
  and 24, and 16 is used there.
- Caption glyphs 15.

## Type

Geist (`assets/fonts/`), with tabular figures for numbers.

| Use | Size | Weight |
| --- | --- | --- |
| Toolbar labels, chips | 11.5 | Regular (chips Medium) |
| Tags, shortcut keys in menus | 11 | Regular |
| Column heads, hints, the status bar, counts | 12 | Medium for column heads, else Regular |
| Field labels, table cells, details values | 12.5 | Regular |
| Body text | 13 | Regular; buttons Medium |
| Dialog headings, the empty list's title | 15 to 16 | SemiBold |

Line height 1.35. Three faces are bundled: Regular (400), Medium (500), SemiBold (600).

The mockup sets shortcut keys in menus, the find button's key and checksums (`.mono`, 12) in
Geist Mono, which is not bundled. Until it is, keys are Geist with tabular figures, and text
that must line up (a checksum) keeps the monospaced font the app uses for it today; plan 04
decides whether to bundle Geist Mono.
