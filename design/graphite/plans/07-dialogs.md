# 07 · Dialogs

Brings every dialog to the mock under the Graphite chrome: the default button last in every footer,
one label column, the file header card, status as icon + word. Colours, corners and controls already
come from the theme (02); this phase is layout. Classic and era themes keep their IDM layouts.

- Branches: `ui/07a-dialog-kit`, then one per wave (`ui/07b-…` to `ui/07e-…`)
- Needs: 02 (can start before 05); 04 for final sizes
- Unblocks: 09

## Facts this plan relies on (paths under `crates/app/src/`)

- Every dialog is its own viewport through `dialogs::window` / `question` / `notice` → `dialog_window`
  (`dialogs.rs:237-367`), with a fixed client size passed as literals (no shared constants), wrapped in
  `scroll()` (`dialogs.rs:83`); `shown_size` caps it to the screen.
- **Layout is absolute pixels in English coordinates**: `winui::r(o, x, y, w, h)` (`winui.rs:209`),
  `text`, `label`; mirroring goes through `rect()`.
- Controls: `button` / `button_light` with `ButtonKind {Normal, Default, Disabled, Accent}`
  (`winui.rs:2713-2724`, `3504`); rows with `button_row` (2903), `button_row_from_left` (2919),
  `buttons_to_right` (2871), `buttons_light_right` (3525); fields `edit_light` (3181) and friends; checks
  `checkbox_light` (3958); radios `radio_light` (4410); combos `combo` (2541); `group_box` (3123); tabs
  `tabs_light` (4985).
- `ButtonKind::Default` is only a look. Enter and Escape are wired per dialog with `enter_pressed` /
  `escape_pressed` (`winui.rs:3842-3847`) and `take_key` (3853).
- A helper `winui::label_column` exists (`winui.rs:513`) but only Add torrent uses it (`torrentdlg.rs:439`).
- Greyed drawing already comes from the theme (`text.disabled`, `field.disabled`, `button.disabled`), so
  02 restyles it everywhere.

Footer order today (left to right, default marked *):

| Dialog | Where | Today | Graphite target |
| --- | --- | --- | --- |
| Add URL | `dialogs.rs:776-785` | OK*, Cancel (stacked right) | Cancel, OK* (footer) |
| Download File Info | `dialogs.rs:1427` | Download Later, Start Download*, Cancel | Download Later, Cancel, Start Download* |
| Download complete | `dialogs.rs:1686` | Open*, Open with..., Open folder, Close | Open with..., Open folder, Close, Open* |
| Confirm deletion | `dialogs.rs:2335-2343` | Yes*, No | No, Yes* (danger fill) |
| Duplicate download link | `dialogs.rs:2552-2560` | OK*, Cancel | Cancel, OK* |
| Download later | `dialogs.rs:2633-2641` | OK*, Cancel | Cancel, OK* |
| Redownload (Confirm) | `dialogs.rs:2696-2705` | Yes*, No | No, Yes* |
| Progress window | `progress.rs:445` | Details, Pause/Resume, Cancel | Details, Cancel, Pause/Resume* |
| Scheduler | `scheduler.rs:371` | New queue, Delete, Start now, Stop, Help, Apply, Close (no default) | New queue, Delete (in the queue list); Stop, Start now* left; Help, Apply, Close right |
| Enter queue name | `scheduler.rs:1290` | OK*, Cancel | Cancel, OK* |
| Add torrent | `torrentdlg.rs:659` | Download Now*, Later, Cancel | Later, Cancel, Download Now* |
| Torrent settings | `torrentdlg.rs:1321-1330` | OK*, Cancel | Cancel, OK* |
| Download video | `videodlg.rs:898-930` | Helpers... ; Download Now*, Later, Cancel | Helpers... ; Later, Cancel, Download Now* |
| File Properties | `properties.rs:412-456` | Files..., Open, OK* | Files..., Open ; OK* |
| Find file | `find.rs:151-173` | Find*, Cancel | Cancel, Find* |
| Batch | `batch.rs:365-377` | OK*, Cancel | Cancel, OK* |
| Link grabber | `grabber.rs:562` | Add to queue, Start*, Cancel | Add to queue, Cancel, Start* |
| RSS feeds | `rssdlg.rs:104` | Close | Close |
| Options | `options.rs:336-344` | OK*, Help | Help, OK* (08) |
| Check for updates | `help.rs:473` | [Retry / Update now / Download]*, Close | Close, [action]* |
| About, Shortcuts | `help.rs:778, 890` | OK* | OK* |
| Tell a friend | `help.rs:974` | Copy*, Email, Close | Email, Close, Copy* |

## 07a · The dialog kit (one PR)

1. `winui::footer(ui, o, client_w, y, buttons: &[FooterButton])` where `FooterButton { label, kind,
   side: Leading | Trailing, mnemonic }`. Under Graphite chrome it draws the footer band (surface fill,
   top rule, 16 px padding), puts leading buttons left and trailing buttons right with the `Default` one
   last; under other chromes it keeps the order given (so a dialog migrated to `footer` looks the same in
   Classic). Returns which button was clicked.
2. Enter and Escape: `footer` reports Enter as the Default button and Escape as the Cancel/Close button
   (named by kind), replacing the per-dialog wiring; dialogs that take Enter for a control keep `take_key`.
3. `winui::form(o, x, y)`: a label column of 112 (or 240 for sentence labels) under Graphite, the
   dialog's own English x under other chromes; built on `label_column`.
4. `winui::file_card(ui, rc, icon, name, sub, action)`: the bordered header with the file icon used by
   Download File Info, Download complete, File Properties and Verify checksum.
5. `winui::dialog_size(classic: Vec2, graphite: Vec2)`: lets a dialog keep its IDM size in Classic and
   take the mock size in Graphite.
6. Tests: footer order per chrome; Enter triggers the Default button, Escape the cancel one, a disabled
   Default ignores Enter; label column positions.

## Waves (each one PR, each dialog compared with its mock state)

| Wave | Dialogs | Mock section |
| --- | --- | --- |
| 07b Adding | Add URL, Download File Info, Download later, Duplicate download link | Adding a download |
| 07c Watching and finishing | Progress window (3 tabs, tab-row menu), Speed Limiter settings, Download complete, File Properties, error state | Downloading; Finished; Delete, redownload and errors |
| 07d Torrents, video and more | Add torrent (incl. looking-up state), torrent window (5 tabs), Torrent settings, Add trackers, Files already there, Download video, playlist, Add many downloads, Download a gallery, Helpers, Batch, Link grabber, RSS feeds | Torrents; Video from a page; Batch, link grabber and RSS |
| 07e Organising and the rest | Scheduler (Schedule, Files, Download limits, wake lines), Enter queue name, queue-done countdown, Find file, Add/Delete category, Confirm deletion, Redownload, Check for updates, About, Keyboard shortcuts, Tell a friend, Verify checksum, crash report, Archive password, 7-Zip needed, Create folder, Download limits notice, extension offer, Safari steps | Queues; Finding and organising; Around the app; Short questions and notices |

For each dialog in a wave:

1. Move its footer to `footer` with the target order above.
2. Move its labels and fields onto `form`; group related options with a hairline rule instead of
   `group_box` under Graphite.
3. Add `file_card` where the mock has one.
4. Status words get their `st-*` icon (progress window status, error banner).
5. Size from `dialog_size`; check at 100% and 200% and with the longest strings (the app is English only).

Left from 03 for 07e: the Scheduler's queue list draws Silk's coloured folders in every theme
(`scheduler.rs`, `IconKey::Silk(icon)` with `folder`, `folder_link`, `folder_delete`). The mock
draws `cat-queues`, `x-refresh` and `x-gauge`; the last two are not among the 53 needs, so 07e adds
them to `NEEDS` and the Graphite set (`export_icons.py`), or the list takes `IconKey::Themed`
with `cat-queues` for all three.

## Done when

- [ ] Every dialog listed matches its mock state in Graphite, light and dark
- [ ] Every dialog in Classic is unchanged (screenshot diff)
- [ ] Enter, Escape, Alt+letter and Tab order work in each dialog
- [ ] Screenshots per wave; ROADMAP, CHANGELOG
