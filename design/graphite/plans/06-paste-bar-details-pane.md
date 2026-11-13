# 06 · Paste bar and details pane

> **Adjusted after 05.5** (Graphite only, merged 2026-11-13): there is no Classic layout, no old theme and no Silk, Tango or Lucide icon left, so this plan's steps are Graphite's alone (`00-conventions.md`).

Two new, optional parts of the main window under the Graphite chrome: a paste bar under the toolbar
that adds a download from an address, and a details pane beside the list that shows the selected
download. Both reuse what exists (00's accepted exception: new routes to existing commands); neither
adds a new kind of download.

- Branch: `ui/06-panes` (06a, the paste bar), `ui/06b-details-pane` (06b, the details pane, stacked on `ui/06-panes`)
- Needs: 05a (`Layout`), 05b (the Graphite toolbar, for the Details toggle), 02b (the Graphite roles), 03b (`gx-*` glyphs)
- Unblocks: 09

## Facts this plan relies on (paths under `crates/app/src/`, checked on `ui/03-icons` c3541d0)

Line numbers drift as phases land: find each item by its name first.

- Add URL: `Cmd::AddUrl` (`app.rs:4947`) opens `dialogs::AddUrl` (`dialogs.rs:566`), pre-filled from the
  clipboard; its outcome is `NewUrl { url, login }` (`dialogs.rs:581-584`). Addresses are parsed with
  `crate::model::parse_address` (`dialogs.rs:611`); the "Invalid address" text is at `dialogs.rs:803`.
- The routing after OK is inline in the outcome match (`app.rs:7940-8005`): remember the history; a
  magnet → `open_torrent_dialog`; a torrent address (`torrents::is_torrent_url`) →
  `open_torrent_dialog_with(Remote…)`; a known video page (`mdm_tools::pages::is_video_page`) →
  `VideoDialog::for_page`; a gallery page → `GalleryDialog::new`; otherwise `duplicate_gate` then
  `open_file_info`.
- View options: `Settings` is serde with defaults (`settings.rs`), saved through `dirty` at most once a
  second (`app.rs:5458-5471`). A toggle = a settings field + default, a `Cmd`, a `run` arm, a View
  `I(label, cmd, enabled, checked)` item (View items about `app.rs:5640-5677`, "Always dark" at
  5666-5672), like `Cmd::ToggleAlwaysDark` (arm at `5022-5026`).
- Keys: Alt/F10 menu keys are skipped while a text box has focus (`app.rs:6429-6432`); Tab toggles
  between tree and list (`6639-6640`).
- The progress window already computes each download's connections and segments (`progress.rs`).

## Steps

1. **Extract the routing** into `fn route_new_url(&mut self, new: NewUrl, ctx)` and call it from the Add
   URL outcome. No behaviour change; one test per branch (magnet, torrent address, video page, gallery,
   plain file).
2. **Settings**: `show_paste_bar: bool`, `show_details: bool`, `details_width: u32`, defaults as decided
   in `../PLAN.md` (paste bar on, details pane off, width 276).
3. **Commands and View menu**: `Cmd::TogglePasteBar`, `Cmd::ToggleDetails` and two check items under
   View ("Paste bar", "Details pane"); the settings persist. The toolbar's Details toggle (end of the Graphite toolbar,
   `gx-sidebar` icon) runs `Cmd::ToggleDetails`; its on state is filled with the role
   `toolbar.checked` (added by 02b).
4. **Paste bar** (`layout.paste_h = 48` when shown, 0 when hidden):
   - A text field with the `gx-link` icon, placeholder "Paste an address or a magnet link", the address
     history as a drop-down (`settings.url_history`), the hint "Enter adds it", and a Download button
     with `gx-download`, filled as the default button.
   - Enter or Download: `parse_address`; invalid → the existing "Invalid address" message under the
     field; valid → `route_new_url(NewUrl { url, login: None })`, history remembered, field cleared.
   - Ctrl+V: only if the main window does not use Ctrl+V today (check `paint`'s shortcut keys); then
     Ctrl+V with no field focused puts the clipboard into the field and focuses it. Otherwise skip it.
   - Keyboard (00's keys exception, all old keys keep working): the field joins the Tab rotation
     (paste bar → tree → list); Escape leaves it; menu keys keep working when it is empty.
   - Accessibility: `Role::TextInput` named "Address", the button `Role::Button` "Download".
5. **Details pane** (`layout.details_w` when shown, a 4 px splitter like the categories one):
   - Shows the focused download: file icon and name, status line, connections as 8 segments (from the
     same data the progress window uses), then label/value rows: Status, Transfer rate and Time left
     (while running), Resume capability, Save To, Address, Queue, Last Try Date; buttons Resume or Stop
     (by state, the default one filled), Open folder, and "Show progress window" (`gx-window`).
   - No selection: a muted "Select a download to see its details".
   - Several selected: the count and the total size.
   - Updates live while the download runs (same repaint cadence as the list).
   - Accessibility: a `Role::Group` named "Details" with its label/value pairs as text.
6. **Layout**: with the pane open, the list keeps its columns and scrolls sideways (no column hiding).
7. **Help**: `docs/` gains the paste bar and the details pane (where to turn them on, what Enter does).

## Mock states to compare

Main window: Details pane (with the stopped download selected), Without it, First start (empty paste
bar, placeholder); Adding a download section (paste bar routing to Download File Info).

## Tests

- `route_new_url`: one test per branch.
- Paste bar: Enter with a valid address opens Download File Info; with a magnet opens Add torrent; with
  junk shows the invalid-address message; history grows.
- Settings round-trip for the three new fields; old settings files load with the defaults.
- Details pane: shows the focused row; updates after Stop/Resume; empty and multi-select states.

## Left from 06

Plan 09a does the full keyboard and accessibility review (09, step 5), so these go there:

- The on-screen keyboard pass (Tab rotation, arrows, Alt and menu letters with the field empty and
  not empty, Enter, Escape, Ctrl+V) and the Narrator pass (the field, the Download button, the
  toggles, the Details group). The user skipped them in 06; the headless tests cover the logic.
- Interface sizes 150 % and 200 % for the paste bar and the details pane (06's screenshots are at
  100 % only).
- The audit's deferred nits:
  - tests of `route_new_url`'s side effects (the history and `dirty`, `request_parts` for a
    .torrent, the Video and General folders, the duplicate check), which need an `App` built in
    a test;
  - a file name without break points stays on one line in the pane's header (`winui::fit_lines`
    does not break inside a word, and other windows share it);
  - tests of the splitter's arithmetic (pull `dragged_width` out of `App::details` first) and of
    a short pane (276 × 150);
  - `/show details` while a column filter is set (the shots run on a fresh data folder, so none
    is).

## Done when

- [x] Both parts toggle from View (and the toolbar, for details) and persist across restarts
- [x] The paste bar routes every kind of address exactly as Add URL does
- [ ] Keyboard and accessibility checks pass (the headless tests pass; the on-screen keyboard and Narrator pass is left to 09a, see "Left from 06")
- [x] Help updated; ROADMAP, CHANGELOG
- [x] Screenshots (light and dark at 100 %, before and after, in the PR)
