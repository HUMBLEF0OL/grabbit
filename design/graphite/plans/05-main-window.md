# 05 · Main window layout

Rebuilds the main window to the mock: menus in the title bar, a grouped toolbar, a flatter category
list, progress inside the Status cell, a quieter status bar. Same commands, order and shortcuts.
The new layout applies only when the theme asks for it; Classic and the era themes keep IDM's layout.

- Branches: `ui/05a-layout-model` (refactor, no visual change), then `ui/05b-main-window`
- Needs: 02, 03, 04
- Unblocks: 06

## Facts this plan relies on (paths under `crates/app/src/`)

- `fn paint` is `app.rs:6503`. Order: `Surface::enter` (6507), `drawn_caption` (6510), menu keys
  (6520-6593), `chrome::menu_bar` (6596) and its popup (6602-6644), toolbar array (6653-6686) and
  `chrome::toolbar` (6688), splitter (6705-6729), tree keys (6732-6778), `chrome::categories` (6779),
  `list::show` (6837), drag border (6873), `empty_state` (6882), `status_strip` (6887), popups
  (6899-6978), shortcut keys (6981-7027), `run` (7029).
- Layout numbers are repeated, not shared: `chrome::MENU_H = 20` (`chrome.rs:15`), `chrome::pane_top()
  = 70` (`chrome.rs:1365`), `STRIP_H = 22` (`app.rs:309`), the popup anchor `o.y + MENU_H - 1`
  (`app.rs:6607`), the toolbar drop anchor `o.y + 85` (`chrome.rs:1542`), `menubar_title_at`
  (`chrome.rs:271`), the splitter rect vs the tree frame (x 6, `right`).
- Menu bar: `chrome::menu_bar` (`chrome.rs:182-267`) from the fixed `MENUS` table (`:166-172`); it is
  exposed as `Role::MenuBar` with `MenuItem` children.
- Drawn title bar (Windows, any theme whose `caption` is not `system`): `drawn_caption`
  (`app.rs:6422-6491`) → `win32::drawn_frame` (`win32.rs:650`), `winui::caption::title_bar`
  (`winui.rs:5982`, `CAPTION_H = 32`), hit areas published with `win32::set_caption_hits`
  (`win32.rs:346`); a rect in `buttons` is client area, the rest of the strip drags the window
  (`frame_hit`, `win32.rs:381-391`).
- Toolbar: `chrome::toolbar` (`chrome.rs:1369-1550`), 11 buttons back to back, 68 px band, 32 px icons,
  Tahoma labels, split drop arrows on Add Torrent, Start Queue and Stop Queue; **not exposed to
  accessibility**.
- Tree: `chrome::categories` (`chrome.rs:1627`), `TreeRow` (`:1600-1611`), 16 px rows, "Categories"
  caption with a close box, dotted connectors or chevrons per `tree_chevrons`; rows from `tree_rows`
  (`app.rs:6300-6412`), counts on level-0 rows only.
- List: `list.rs` `COLS = 13`, `TITLES` (53-76), `DEFAULT_ORDER` (96), row height from
  `theme::metrics().list_row`; cells drawn at 915-980; **Status is text only**; Source chip `chip()` (1241-1275).
- Status strip: `status_strip` (`app.rs:1291-1316`), no background.
- Right-to-left mirroring is switched off (`winui.rs:112-115`) but new code still goes through
  `winui::rect` / `mx`.

## 05a · One layout model (refactor, no visual change)

1. Add `chrome::Layout { caption_h, menu_h, toolbar_h, strip_h, tree_left, … }` computed once per frame
   from the theme, with today's values for every theme.
2. Replace every repeated number above with the struct: `paint`, `menu_bar`, `menubar_title_at`,
   `toolbar` and its drop anchor, `categories`, the splitter, `list::show`'s rect, `status_strip`,
   the popup anchor.
3. Test: screenshots of every main-window state before and after are identical (pixel diff) in Classic,
   Metro and Aqua.

## 05b · The Graphite main window

Gate everything below on a new theme control, `[controls] chrome = "classic" | "graphite"` (new
`ChromeLook` in `palette.rs`, parsed in `Controls::set`); `graphite.toml` sets `chrome = "graphite"`
(in `make_theme.py`'s `HEADER`, which writes the file).

1. **Menus in the title bar** (drawn caption only). Draw `menu_bar` inside the caption rect after the
   app mark and name; add each title's rect to `CaptionHits.buttons` so it is client area; keep
   `max_button` pointing at Maximize; move the popup anchor and `menubar_title_at` to the new row.
   `layout.menu_h = 0` so the panes move up. Fallback: when the caption is the system's, keep the menu row.
2. **Find button** in the title bar (between the menus and the window buttons): runs `Cmd::Find`, shows
   "Ctrl+F", added to `CaptionHits.buttons`.
3. **Toolbar**: 20 px icons, 11.5 px labels, button 62 px wide, band height 58; groups
   [Add URL, Add Torrent] [Resume, Stop, Stop All] [Delete, Delete Completed] [Options, Scheduler]
   [Start Queue, Stop Queue] with a 1 px rule between groups; hover = `toolbar.hot` rounded 8; the
   drop arrow drawn inside the button's right edge. Add the Details toggle at the end only after 06.
   **Expose each button** (`Role::Button`, name, disabled state) through `winui::expose`.
4. **Categories**: no "Categories" caption bar (View, Hide categories still closes the pane); 28 px rows,
   chevrons (no dotted lines), 16 px icons, the selected row filled with `selection` and its icon in
   `accent`, counts "(n)" muted on top-level rows only (unchanged rule).
5. **List**: `list.row = 32` (theme), header in `text.muted` 12 px with a bottom rule only; rows separated by a
   hairline `list.grid`; the Source chip as a pill.
6. **Status cell with progress**: give `list::Row` a `progress: Option<f32>` and a status kind; in
   column 3 draw, for running and stopped downloads, a 4 px bar (`progress.track` / `progress.bar`,
   stopped in `text.muted`) and the percent; for the other kinds an icon (`st-*` needs) and the word.
   Error keeps `winui::ERROR()`. The text in the cell stays the same for screen readers.
7. **Status bar**: `surface` background, top rule, 12 px muted text; content unchanged.
8. **Empty states**: centred icon tile, title, reason and link as in the mock (`empty_state`, `app.rs:1227-1287`).

## Mock states to compare

Main window: Without it (details off), Tasks / File / Downloads / View / Help menus, right-click a
download, blank space, a category, a queue, Column chooser, Column filter, First start, Filters hide
all: light and dark, at 100% and 200%.

## Tests

- 05a: pixel-identical screenshots (Classic, Metro, Aqua) before and after.
- Hit testing: with Graphite, a click on each menu title opens its menu; a drag on empty caption moves
  the window; double-click maximises.
- Accessibility: the toolbar exposes 11 buttons with names and disabled states; the menu bar still
  exposes `MenuBar` / `MenuItem`; the list still exposes its rows.
- Keyboard: Alt, F10, Alt+letter open the title-bar menus; Tab still moves between tree and list.

## Done when

- [ ] 05a merged with no visual change
- [ ] Every main-window mock state matches in Graphite; Classic and era themes unchanged
- [ ] Hit, keyboard and accessibility tests above pass
- [ ] Screenshots before and after; ROADMAP, CHANGELOG
