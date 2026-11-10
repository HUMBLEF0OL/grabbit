# 06 · Paste bar and details pane

Two new, optional parts of the main window: a paste bar under the toolbar that adds a download from
an address, and a details pane beside the list that shows the selected download. Both reuse what
exists; neither adds a new kind of download.

- Branch: `ui/06-panes`
- Needs: 05
- Unblocks: nothing (07 can run in parallel)

## Facts this plan relies on (paths under `crates/app/src/`)

- Add URL: `Cmd::AddUrl` (`app.rs:5039`) opens `dialogs::AddUrl` (`dialogs.rs:593`), pre-filled from the
  clipboard; its outcome is `NewUrl { url, login }` (`dialogs.rs:608-611`). Addresses are parsed with
  `crate::model::parse_address` (`dialogs.rs:638`).
- The routing after OK is inline in the outcome match (`app.rs:8049-8113`): remember the history; a
  magnet → `open_torrent_dialog`; a torrent address → `open_torrent_dialog_with(Remote…)`; a known video
  page (`mdm_tools::pages::is_video_page`) → `VideoDialog::for_page`; a gallery page →
  `GalleryDialog::new`; otherwise `duplicate_gate` then `open_file_info`.
- View options: `Settings` is serde with defaults (`settings.rs`), saved through `dirty` at most once a
  second (`app.rs:5551-5565`). A toggle = a settings field + default, a `Cmd`, a `run` arm, a View
  `I(label, cmd, enabled, checked)` item (`app.rs:5737-5769`), like `Cmd::ToggleAlwaysDark` (`5114-5118`).
- Keys: Alt/F10 menu keys are skipped while a text box has focus (`app.rs:6520-6522`); Tab toggles
  between tree and list (`6735-6744`).
- The progress window already computes each download's connections and segments (`progress.rs`).

## Steps

1. **Extract the routing** into `fn route_new_url(&mut self, new: NewUrl, ctx)` and call it from the Add
   URL outcome. No behaviour change; covered by the existing tests plus one per branch (magnet, torrent
   address, video page, gallery, plain file).
2. **Settings**: `show_paste_bar: bool`, `show_details: bool`, `details_width: u32` with the defaults
   decided in 01 (recommended: paste bar on, details off, width 276).
3. **Commands and View menu**: `Cmd::TogglePasteBar`, `Cmd::ToggleDetails` and two check items under
   View ("Paste bar", "Details pane"). The toolbar's Details toggle (end of the toolbar) runs
   `Cmd::ToggleDetails` and shows its state.
4. **Paste bar** (Graphite chrome only; `layout.paste_h = 48` when shown, 0 otherwise):
   - A text field with the link icon, placeholder "Paste an address or a magnet link", the address
     history as a drop-down (`settings.url_history`), the hint "Enter adds it", and a Download button.
   - Enter or Download: `parse_address`; invalid → the field shows the existing "Invalid address"
     message; valid → `route_new_url(NewUrl { url, login: None })`, history remembered, field cleared.
   - Ctrl+V in the main window with no field focused puts the clipboard into the field and focuses it.
   - Keyboard: the field joins the Tab rotation (paste bar → tree → list); Escape leaves it; menu keys
     keep working when it is empty.
   - Accessibility: `Role::TextInput` named "Address", the button `Role::Button`.
5. **Details pane** (`layout.details_w` when shown, a 4 px splitter like the categories one):
   - Shows the focused download: file icon and name, status line, connections as 8 segments (from the
     same data the progress window uses), Status, Resume capability, Save To, Address, Queue, Last Try
     Date; buttons Resume or Stop (by state), Open folder, and "Show progress window".
   - No selection: a muted "Select a download to see its details".
   - Several selected: the count and the total size.
   - Updates live while the download runs (same repaint cadence as the list).
   - Accessibility: a `Role::Group` named "Details" with its label/value pairs as text.
6. **Layout**: with the pane open, the list keeps its columns and scrolls sideways (no column hiding).

## Mock states to compare

Main window: Details pane (with the stopped download selected), Without it, First start (empty paste
bar, placeholder); Adding a download section (paste bar routing to Download File Info).

## Tests

- `route_new_url`: one test per branch.
- Paste bar: Enter with a valid address opens Download File Info; with a magnet opens Add torrent; with
  junk shows the invalid-address message; history grows.
- Settings round-trip for the three new fields; old settings files load with the defaults.
- Details pane: shows the focused row; updates after Stop/Resume; empty and multi-select states.

## Done when

- [ ] Both parts toggle from View and persist across restarts
- [ ] The paste bar routes every kind of address exactly as Add URL does
- [ ] Keyboard and accessibility checks pass
- [ ] Screenshots, ROADMAP, CHANGELOG
