# 05 · Main window layout

Rebuilds the main window to the mock: menus in the title bar, a grouped toolbar, a flatter category
list, progress inside the Status cell, a quieter status bar, icons and shortcuts in menus. Same
commands, order and shortcuts. The new layout applies only under the Graphite chrome (02b); Classic
and the era themes keep IDM's layout, pixel for pixel.

- Branches: `ui/05a-layout-model` (refactor, no visual change), then `ui/05b-main-window`
- Needs: 05a needs 02 only; 05b needs 02b (the chrome gate), 03 and 03b (icons, extra glyphs), 04 (sizes), 05a
- Unblocks: 06

## Facts this plan relies on (paths under `crates/app/src/`, checked on `ui/03-icons` c3541d0)

Line numbers drift as phases land: find each item by its name first.

- `fn paint` is `app.rs:6413`. Order: `Surface::enter` (6417), `drawn_caption` (6420), menu keys
  (6425-6495), `chrome::menu_bar` (6498) and its popup (6505-6545), toolbar array (6548-6588) and
  `chrome::toolbar` (6590), splitter (6603-6631), tree keys (6632-6680), `chrome::categories` (6681),
  `list::show` (6739), drag border (6775), `empty_state` (6785), `status_strip` (6789), popups
  (6801-6880), shortcut keys (6881-6930), `run` (about 6931).
- Layout numbers are repeated, not shared: `chrome::MENU_H = 20` (`chrome.rs:15`), `chrome::pane_top()
  = 70` (`chrome.rs:1325`), `STRIP_H = 22` (`app.rs:308`), the popup anchor `o.y + MENU_H - 1`
  (`app.rs:6509`), the toolbar drop anchor `o.y + 85` (`chrome.rs:1502`), `menubar_title_at`
  (`chrome.rs:271`), the splitter rect vs the tree frame (x 6, `right`).
- Menu bar: `chrome::menu_bar` (`chrome.rs:182-267`) from the fixed `MENUS` table (`:166-172`); exposed
  as `Role::MenuBar` with `MenuItem` children. Menu rows are `MenuEntry` (`chrome.rs:330`): label,
  command, enabled, checked; **no icon and no shortcut column** (shortcuts come from `Cmd::hint`).
- Drawn title bar (any theme whose `caption` is not `system`; Graphite's is `metro`): `drawn_caption`
  (`app.rs:6338-6401`) → `win32::drawn_frame` (`win32.rs:649`), `winui::caption::title_bar`
  (`winui.rs:5954`, `CAPTION_H = 32` at `:5454`), hit areas published with `win32::set_caption_hits`
  (`win32.rs:345`); a rect in `buttons` is client area, the rest of the strip drags the window
  (`frame_hit`, `win32.rs:380-390`).
- Toolbar: `chrome::toolbar` (`chrome.rs:1329-1510`), 11 buttons back to back, 68 px band, 32 px icons,
  Tahoma labels, split drop arrows on Add Torrent, Start Queue and Stop Queue; **not exposed to
  accessibility**.
- Tree: `chrome::categories` (`chrome.rs:1587`), `TreeRow` (`:1560-1571`), 16 px rows, "Categories"
  caption with a close box, dotted connectors or chevrons per `tree_chevrons`; rows from `tree_rows`
  (`app.rs:6210-6333`), counts on level-0 rows only. 03 already gives Finished and Queues their icon
  alone (`themeicons::tree_badges`); a synchronized queue keeps a tick badge.
- List: `list.rs` `COLS = 13`, `TITLES` (60-83), `DEFAULT_ORDER` (103), row height from
  `theme::metrics().list_row` (Graphite already sets `list.row = 32` in 02); cells drawn at about
  930-995; **Status is text only**; Source chip `chip()` (1251-1283).
- Status strip: `status_strip` (`app.rs:1259-1284`), no background. There is no `surface` palette
  role (it is only a seed); the nearest roles are `window` and `list`.
- Empty states: `empty_state` (`app.rs:1195-1255`).
- Right-to-left mirroring is switched off (`winui.rs:113`) but new code still goes through
  `winui::rect` / `mx`. A comment at `app.rs:6501-6503` still mentions X11: tidy it in 05a.

## 05a · One layout model (refactor, no visual change)

1. Add `chrome::Layout { caption_h, menu_h, toolbar_h, strip_h, tree_left, … }` computed once per frame
   from the theme, with today's values for every theme.
2. Replace every repeated number above with the struct: `paint`, `menu_bar`, `menubar_title_at`,
   `toolbar` and its drop anchor, `categories`, the splitter, `list::show`'s rect, `status_strip`,
   the popup anchor.
3. Test: screenshots of every main-window state before and after are identical in Classic, Metro, Aqua
   and Graphite, taken with `design/graphite/source/baseline.ps1` and compared by a small
   `source/pixel_diff.py` (PIL, exits non-zero on any differing pixel) written in this phase.
   No CHANGELOG line.

## 05b · The Graphite main window

Everything below draws only when `theme::chrome()` is `ChromeLook::Graphite` (02b).

1. **Roles**: draw with the roles 02b added: `panel` (status bar, title-bar band) and `menu.shortcut`
   (shortcut column, Find's "Ctrl+F").
2. **Menus in the title bar** (drawn caption only). Draw `menu_bar` inside the caption rect after the
   app mark and name; add each title's rect to `CaptionHits.buttons` so it is client area; keep
   `max_button` pointing at Maximize; move the popup anchor and `menubar_title_at` to the new row.
   `layout.menu_h = 0` so the panes move up. Fallback: when the caption is the system's, keep the menu row.
   Under Graphite the title bar is 36 high (`layout.caption_h`; `CAPTION_H = 32` stays for other
   themes) with 42 x 35 window buttons, as DESIGN.md.
3. **Find button** in the title bar (between the menus and the window buttons), a new route to the
   existing `Cmd::Find` (00's accepted exception): `window` face, `button.border`, radius 6, 26 high,
   `gx-search` icon, the label "Find" and "Ctrl+F" in `menu.shortcut`; added to `CaptionHits.buttons`;
   exposed as `Role::Button` "Find".
4. **Menu rows**: give `MenuEntry::Item` an optional icon (a need or `gx-*` slot) and show the shortcut
   from `Cmd::hint` in its own right-aligned column in `menu.shortcut`. Under Graphite draw a 16 px icon
   at the row's start; labels keep their wording (for example "Move/Rename (Ctrl-M)"). The icons per row
   are those in the mock (`MENUS` and the right-click menus in `design/graphite/source/graphite.html`);
   under Classic menus look as today.
5. **Toolbar**: 20 px icons, 11.5 px labels, buttons at least 60 px wide (8 px padding each side), band
   height 58; groups [Add URL, Add Torrent] [Resume, Stop, Stop All] [Delete, Delete Completed]
   [Options, Scheduler] [Start Queue, Stop Queue] (today's order) with a 1 px rule between groups;
   the band on `panel` with a bottom rule (then remove `make_theme.py`'s "until plan 05" mix of the
   toolbar faces on `bg` and regenerate); hover = `toolbar.hot` rounded 8; the drop arrow drawn inside
   the button's right edge. The Details toggle at the end comes with 06. **Expose each button**
   (`Role::Button`, name, disabled state) through `winui::expose`, in every theme (00's accessibility
   exception: invisible, the tree only gains nodes).
6. **Categories**: under Graphite the pane width is its own setting, `graphite_categories_width`
   (default 224, saved like `categories_width`, whose default 143 stays for Classic); no "Categories" caption bar (View, Hide
   categories still closes the pane); 28 px rows, chevrons (no dotted lines), 16 px icons, the selected
   row filled with `selection` and its icon in `accent`, counts "(n)" muted on top-level rows only
   (unchanged rule). The Synchronization queue draws `gx-refresh` (03b) instead of the tick badge.
7. **List**: header in `text.muted` 12 px with a bottom rule only, the sort chevron after the sorted
   column's title, `gx-filter` after a filtered column's title; rows separated by a hairline
   `list.grid`; the Source chip as a pill. Numeric cells (size, rate, time left, percent) in a
   `numbers` family that keeps Segoe UI's tabular figures (04's digits decision: Geist's are
   proportional), so they line up as they change; `win32.rs` `classic_face` maps `numbers` to
   `segoeui.ttf` in both its branches, or egui would draw it with fractional advances. 04's main-window overflow (in its plan) is fixed here.
8. **Status cell with progress**: give `list::Row` a `progress: Option<f32>` and a status kind; in
   column 3 draw: a running download, a 4 px bar (`progress.track` / `progress.bar`) and the percent;
   a stopped one, the `st-paused` icon, the word, then the bar in `text.muted` and the percent; every
   other kind, its `st-*` icon and the word. Error keeps `winui::ERROR()`. The cell's text stays the
   same for screen readers.
9. **Status bar**: `panel` background, top rule, 12 px muted text; content unchanged.
10. **Empty states**: centred `icon_tile` (02b; 56 px, radius 14) with `gx-download` (no downloads yet)
    or `gx-filter` (filters hide all), title, reason and link, as in the mock.

## Mock states to compare

Main window: Without it (details off), Tasks / File / Downloads / View / Help menus with the Speed
Limiter and Arrange files submenus, right-click a download, blank space, a category, a queue, Column
chooser, Column filter, First start, Filters hide all: light and dark, at 100% and 200%.

## Tests

- 05a: pixel-identical screenshots (Classic, Metro, Aqua, Graphite) before and after.
- Hit testing: with Graphite, a click on each menu title opens its menu; a drag on empty caption moves
  the window; double-click maximises; the Find button opens Find file.
- Accessibility: the toolbar exposes 11 buttons with names and disabled states; the menu bar still
  exposes `MenuBar` / `MenuItem`; the list still exposes its rows; the Find button is a `Button`.
- Keyboard: Alt, F10, Alt+letter open the title-bar menus; Tab still moves between tree and list.
- Classic: every main-window state pixel-identical to the 01 baseline.

## Done when

- [ ] 05a merged with no visual change
- [ ] Every main-window mock state matches in Graphite; Classic and era themes unchanged
- [ ] Hit, keyboard and accessibility tests above pass
- [ ] Screenshots before and after; ROADMAP, CHANGELOG
