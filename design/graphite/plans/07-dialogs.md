# 07 · Dialogs

> **Adjusted after 05.5** (Graphite only, merged 2026-11-13): there is no Classic layout, no old theme and no Silk, Tango or Lucide icon left, so this plan's steps are Graphite's alone (`00-conventions.md`).

Brings every dialog to the mock under the Graphite chrome: the default button last in every footer and
filled, one label column, file cards, icon tiles, status banners, status as icon + word. Colours,
corners and controls already come from the theme (02); this phase is layout. **Keys never change**: each dialog keeps today's Enter and
Escape behaviour.

- Branches: `ui/07a-dialog-kit`, then one per wave (`ui/07b-…` to `ui/07e-…`)
- Needs: 02b (the Graphite roles), 03 (`st-*` and file icons), 03b (`gx-*` glyphs);
  04 before final sizes (04's plan lists the labels Geist overflows: limiter, gallery, Scheduler,
  grabber, progress window). Independent of 05 and 06.
- Unblocks: 08 (needs 07a), 09

## Facts this plan relies on (paths under `crates/app/src/`, checked on `ui/03-icons` c3541d0)

Line numbers drift as phases land: find each item by its name first.

- Every dialog is its own viewport through `dialogs::window` (`dialogs.rs:224`) / `question` (239) /
  `notice` (255) → `dialog_window` (295-446), with a fixed client size passed as literals (no shared
  constants), wrapped in `scroll()` (`dialogs.rs:83`); `shown_size` caps it to the screen.
- **Layout is absolute pixels in English coordinates**: `winui::r(o, x, y, w, h)` (`winui.rs:209`),
  `text`, `label`; mirroring goes through `rect()`.
- Controls: `button` / `button_light` with `ButtonKind {Normal, Default, Disabled, Accent}`
  (`winui.rs:2713-2724`, `3504`); rows with `button_row` (2903), `button_row_from_left` (2919),
  `buttons_to_right` (2871), `buttons_light_right` (3525); fields `edit_light` (3181) and friends; checks
  `checkbox_light` (3958); radios `radio_light` (4410); combos `combo` (2541); `group_box` (3123); tabs
  `tabs_light` (4982). No Danger or Ghost kind; buttons carry no icon.
- `ButtonKind::Default` is only a look (an accent border; Graphite's theme fills only `Accent`). Enter
  and Escape are wired per dialog with `enter_pressed` / `escape_pressed` (`winui.rs:3842-3847`) and
  `take_key` (3853). In the progress window Escape hides the window and the download goes on
  (`progress.rs:301-303`); its Cancel button stops the download.
- A helper `winui::label_column` exists (`winui.rs:513`) but only Add torrent uses it (`torrentdlg.rs:439`).
- `winui::banner` (`winui.rs:226`) is the first-start logo band, not a status banner.
- Greyed drawing already comes from the theme (`text.disabled`, `field.disabled`, `button.disabled`).
- The Scheduler's queue list already draws Graphite's `cat-queues`, `gx-refresh` and `gx-gauge`
  (`IconKey::Graphite`, since 05.5a).
- The tray menu is native (`tray_icon::menu`, `tray.rs:158`): the theme cannot draw it.
- The drop target (`droptarget.rs`) draws the app icon.

Footer order today (left to right, default marked *) and under Graphite:

| Dialog | Where | Today | Graphite (leading ; trailing) |
| --- | --- | --- | --- |
| Add URL | `dialogs.rs:748-768` | OK*, Cancel (stacked right) | ; Cancel, OK* (footer) |
| Download File Info | `dialogs.rs:1400` | Download Later, Start Download*, Cancel | ; Download Later, Cancel, Start Download* |
| Download complete | `dialogs.rs:1659` | Open*, Open with..., Open folder, Close | Open with..., Open folder ; Close, Open* |
| Confirm deletion | `dialogs.rs:2271-2283` | Yes*, No | ; No, Yes* (Danger) |
| Duplicate download link | `dialogs.rs:2480-2491` | OK*, Cancel | ; Cancel, OK* |
| Download later | `dialogs.rs:2561-2572` | OK*, Cancel | ; Cancel, OK* |
| Redownload (Confirm) | `dialogs.rs:2624-2636` | Yes*, No | ; No, Yes* |
| Progress window | `progress.rs:424` | Details, Pause/Resume, Cancel | Details ; Cancel, Pause/Resume* (look only: Enter does nothing, Escape hides, as today) |
| Scheduler | `scheduler.rs:371` | New queue, Delete, Start now, Stop, Help, Apply, Close (no default) | New queue and Delete move under the queue list; Stop ; Help (Ghost), Apply, Close, Start now* (look only: no Enter, as today) |
| Enter queue name | `scheduler.rs:1274` | OK*, Cancel | ; Cancel, OK* |
| Add torrent | `torrentdlg.rs:659` | Download Now*, Later, Cancel | Download in order (check) ; Later, Cancel, Download Now* |
| Torrent settings | `torrentdlg.rs:1314-1323` | OK*, Cancel | ; Cancel, OK* |
| Download video | `videodlg.rs:886-921` | Helpers... ; Download Now*, Later, Cancel | Helpers... ; Later, Cancel, Download Now* |
| File Properties | `properties.rs:412-456` | Files..., Open, OK* | Files..., Open ; OK* |
| Find file | `find.rs:151-173` | Find*, Cancel | ; Cancel, Find* |
| Batch | `batch.rs:365-377` | OK*, Cancel | ; Cancel, OK* |
| Link grabber | `grabber.rs:562` | Add to queue, Start*, Cancel | ; Add to queue, Cancel, Start* |
| RSS feeds | `rssdlg.rs:105` | Close | ; Close |
| Options | `options.rs:336-344` | OK*, Help | Help (Ghost) ; OK* (08) |
| Check for updates | `help.rs:441` | [Retry / Update now / Download]*, Close | ; Close, [action]* |
| About, Shortcuts | `help.rs:744, 856` | OK* | ; OK* |
| Tell a friend | `help.rs:938` | Copy*, Email, Close | ; Email, Close, Copy* |
| Generic notice (`dialogs::notice`) | `dialogs.rs:255` | OK*, centred | OK*, centred (the one centred footer) |

## 07a · The dialog kit (one PR, no visual change, no CHANGELOG line)

1. `winui::footer(ui, o, client_w, y, items: &[FooterItem])` where `FooterItem` is a `Button(FooterButton
   { label, kind, side: Leading | Trailing, icon: Option<IconKey>, mnemonic, enter: bool, escape: bool })`
   or a leading `Check(label, &mut bool)` (Add torrent's "Download in order", Confirm deletion's
   "Don't show this dialog again"). It
   draws the footer band (`panel` fill from 02b, top rule, 16 px padding), leading buttons left,
   trailing buttons right with the `Default` one last. A dialog moves to `footer` in its own wave, so
   07a itself changes no window. Returns which button was clicked.
2. **Keys stay as they are**: `footer` reports Enter only for the button the caller marks `enter: true`
   and Escape only for the one marked `escape: true`; each dialog marks exactly what it handles today.
   Marking a button `Default` changes its look only.
3. **Button kinds**: `ButtonKind::Danger` (roles `button.danger`, `button.danger.text` from 02b) and
   `ButtonKind::Ghost` (no face until hovered); a `Default` button paints filled like
   `Accent`. An optional icon before the label on `footer` buttons and on `button`.
4. `winui::form(o, x, y)`: a label column of 112 (or 240 for sentence labels); built on
   `label_column`.
5. `winui::file_card(ui, rc, icon, name, sub, action)`: the bordered header with the file icon (Download
   File Info, Download complete, File Properties, Verify checksum). Icon tiles (Confirm deletion,
   Redownload, queue done, updates, crash report, video helpers, Options page headers) use 02b's
   `icon_tile` at 40 px, radius 8.
6. `winui::status_banner(ui, rc, kind: Error | Info | Warning, icon, title, sub)` with the roles
   `banner.error`, `banner.info`, `banner.warning` from 02b (error window, torrent General, Options Remote
   and Accounts, gallery, Verify checksum). Not the existing `winui::banner`.
7. Each dialog takes the mock's size in its wave (no `dialog_size` for two layouts: 05.5 left one).
8. Tests: footer order per chrome; Enter and Escape fire only the marked buttons; a disabled marked
   button ignores Enter; **Escape in the progress window leaves the download running**; label column
   positions.

## Waves (each one PR, each dialog compared with its mock state)

| Wave | Dialogs | Mock section |
| --- | --- | --- |
| 07b Adding | Add URL, Download File Info, Download later, Duplicate download link | Adding a download |
| 07c Watching and finishing | Progress window (3 tabs, Hide tab, tab-row menu), error state (status banner), Speed Limiter settings, Download complete (drag handle with `gx-grip`; the mock's "Windows" tag is not built), File Properties | Downloading; Finished; Delete, redownload and errors |
| 07d Torrents, video and more | Add torrent, including the looking-up state (a spinner ring in `progress.bar`), torrent window (5 tabs, pieces grid), Torrent settings, Add trackers, Files already there, Download video, playlist, Add many downloads, Download a gallery, "Get ready for video downloads" (the offer), Helpers, Batch, Link grabber, RSS feeds | Torrents; Video from a page; Batch, link grabber and RSS |
| 07e Organising and the rest | Scheduler (Schedule, Files in the queue, Download limits; the wake line in its Windows variants only: will be woken, on battery, timers off, plugged in only, no alarm), Enter queue name, queue-done countdown, Find file, Add/Delete category, Confirm deletion, Redownload, Check for updates, About, Keyboard shortcuts, Tell a friend, Verify checksum, crash report, Archive password, 7-Zip needed, Create folder, Download limits notice, generic notice, extension offer, drop target | Queues; Finding and organising; Around the app; Short questions and notices |

Not built: the Safari steps (macOS only; no such code on `dev`), the mock's Linux and macOS wake
lines, and the tray menu (native: it stays as it is).

For each dialog in a wave:

1. Move its footer to `footer` with the order above and its keys marked as today.
2. Move its labels and fields onto `form`; group related options with a hairline rule instead of
   `group_box`.
3. Add `file_card`, `icon_tile` and `status_banner` where the mock has them; buttons get their mock icons
   (`gx-dots` on every "..." browse button, `gx-search` on Look up, `gx-plus`, `gx-external`, etc.).
4. Status words get their `st-*` icon (progress window status, error banner).
5. Size it as the mock does; check at 100% and 200% and with the longest strings (the app is English only).

Specific to 07e:

- **Scheduler queue list** (left from 03): done in 05.5a (`cat-queues`, `gx-refresh`, `gx-gauge`, as
  the mock does); 07e only lays the window out.
- **Drop target**: radius 10, the `gx-download` glyph in `accent` above its text.
- **About**: laid out as the mock, keeping today's app icon at 56 px; 09a step 3 swaps in the
  gradient mark (and 10b the Grabbit logo).

## Done when

- [ ] Every dialog listed matches its mock state in Graphite, light and dark
- [ ] Enter, Escape, Alt+letter and Tab order work in each dialog exactly as before
- [ ] Screenshots per wave; ROADMAP, CHANGELOG (none for 07a)
