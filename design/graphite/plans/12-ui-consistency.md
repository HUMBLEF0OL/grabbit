# 12 · UI consistency and polish

Spec written on dev `ad995ea8`, 2026-11-21, and checked against `f2484ed4` (PR 97) before it was
locked: A1's remembered tick and the Options Tools headings were already fixed there (noted where they
apply). It comes from a read-only audit of the desktop app: four
sweeps (visual system, copy, dialog structure, flows), a second independent check of every
finding against the code, and an external advisor's review (APPROVED in round 2; revised after
the maintainer's merge-cost rule, below, and its round-3 changes applied). Line numbers drift: find each place by the name next to it.

**Goal.** The app says one thing one way, keeps the user's files safe, and shows the state it is
in. No new visual direction: Graphite (locked, 05–09) stays.

**Scope.** The desktop app (`crates/app/src`). Out of scope, by the maintainer's decision:
accessibility and window scaling (toolbar keyboard reach, interface size, minimum sizes,
resizable dialogs, menu access keys). The website and the browser extension are not covered.
The download (progress) window and the Done window are redesigned by plan 13
(`13-download-window.md`): their items moved there and are marked "moved to 13" below, so neither
plan does the same work twice.

**Rules.** 00-conventions applies: behaviour parts and look parts are separate PRs (a look part
changes no command, key, default or enabled rule); each part gets an external audit until
APPROVED, screenshots (before/after, light and dark), and ROADMAP/CHANGELOG lines. The mockup
`../grabbit-design.html` is updated with any label this plan changes.

## The balance rule: consistency against upstream merge cost

The maintainer, 2026-11-21: IDM was the inspiration; its names and formats are not binding. The
constraint is keeping the original MDM's fixes coming in (`/upstream-sync`): every edit in a line
upstream keeps changing can turn a clean pick into a conflict or a hand port. So each fix is
weighed by where it lands.

Upstream churn since the fork point `390a5016` (360 upstream commits, to 2026-11-21):

| Where | Upstream commits touching it | Cost of an edit |
| --- | --- | --- |
| `app.rs` (whole file) | 111 | high, except the menu table |
| `app.rs` menu entries (`I("…")` lines) | 3 | low |
| footer buttons (`FooterButton`, all files) | none: the fork's own | none |
| `dialogs.rs` 38 · `model.rs` 35 · `options.rs` 26 · `winui.rs` 24 · `videodlg.rs` 24 · `progress.rs` 22 | | high: keep edits few and local |
| `chrome.rs` 11 · `tray.rs` 10 · `torrentdlg.rs` 9 · `grabber.rs` 9 · `torrents.rs` 8 · `list.rs` 7 · `properties.rs` 6 · `errors.rs` 6 | | medium |
| `scheduler.rs` 3 · `categories.rs` 3 · `deleter.rs` 4 · `gallerydlg.rs` 2 · `rssdlg.rs` 1 · `find.rs` 0 | | low |
| `settingspage.rs`, `adddlg.rs` (the fork's own files) | none | none |

How the rule applies:

1. **Behaviour fixes** (12a, 12b) go in wherever they land: the user's files and the app's states
   are worth a hand-resolved pick.
2. **Words** go in when the line is cheap (menus, footers, low-churn files). In high-churn files,
   change the shared function or table once rather than each call site.
3. **Look refactors** of shared code (new helpers adopted across many call sites in high-churn
   files) are done only where the user sees a difference, at the helper where possible. Where it
   would need a re-layout or many edits in a high-churn file, it is marked **costly** and done last.
4. The upstream-sync ledger (`tools/upstream-sync/ledger.md`) notes, for each later pick that
   conflicts with a plan-12 change, which part caused it.

## Status

Spec locked 2026-11-21. The maintainer approved the mockups: eleven boards of now-vs-proposed
states, Graphite light and dark, one per area below. Building started 2026-11-22: 12b is done
(branch `ui/12b-states-routes`). 12a–12d need no owner answer; 12e waits for the two decisions
below.

12b left one D1 place out: Link grabber has no address box (its links come by Ctrl+V, Import or
the browser), and giving one of its boxes (the package's name, password or folder) the keys
would take Ctrl+V's link pasting, which works only while none of them has them.

Follow-up for the release PR (the website is outside this plan): the website's feature pages
still quote 12c's old labels (`website/content/features/torrents.md`, `video.md`, `galleries.md`,
`link-grabber.md`, `archives.md`, `scheduler.md`, and `dns-proxies.md` for the proxy/SOCKS options).

## Decisions (owner; default if no answer)

| # | Question | Default | Other option |
| --- | --- | --- | --- |
| DA6 | Scheduler "Start now" applies every pending Scheduler draft (queues, limits, queue list; `scheduler.rs:290-307`, `618-620`), so the Options footer's Discard can't undo them. Plan 11b1 chose this (test `start_now_applies_then_starts`, `scheduler.rs:2380-2406`; module doc `scheduler.rs:4`) | **keep** | ask "Save your changes first?" (change the test and the doc) |
| DA1t | A torrent's "Remove with files" deletes through the engine (`torrents.rs:794-806`, `t.remove(hash, files)`), not the deleter | **unchanged** (engine code, medium churn) | move a torrent's files to the Recycle Bin too |

## 12a · Delete safety (behaviour, about 500 lines)

- **A1. Deleted files go to the Recycle Bin.** `deleter.rs:202` uses `std::fs::remove_file`
  (permanent), and video pieces use `remove_dir_all`.
  - **Already fixed on dev:** the remembered tick. Upstream #1563 was ported in PR 97
    (`158b9f63`): `delete_files_too` is no longer read or written; the box starts unticked; only
    Shift+Delete and "Remove with files" tick it, and they always ask. While ticked, the question
    says "This also deletes N finished files." The audit's finding was made on `ad995ea8`, before
    this.
  - **Change (left):** move a finished file to the Recycle Bin with `win32::to_recycle_bin`
    (`win32.rs`). It returns `bool`, so wrap it and keep `delete_one`'s NotFound-as-success case.
    Partial pieces and temp folders may still be deleted outright. The box's words say where the
    files go: "Also move the {n} files to the Recycle Bin".
  - Torrents per DA1t.
  - **Tests:** the deleter tests inject their own remover (`deleter.rs:347`, `393`, `441`, `473`);
    keep `Deleter::with`. New: a deleted file lands in the test trash (`MDM_TEST_TRASH_DIR`,
    `extractq.rs:331-345`).
- **E3. Say what Delete left out.** `ask_delete` silently skips running downloads in a mixed
  selection (`app.rs:3765-3770`). Show "N running downloads were kept" in the confirmation, or in
  the status strip when the prompt is off.
- **B7. RSS removal.** Feed "Remove" and rule "Delete" act at once, on the Delete key too
  (`rssdlg.rs:263`, `273-281`, `496`, `506-512`). Use one verb ("Remove"). Ask before removing a
  feed (it holds rules); a rule goes at once.

## 12b · States and routes (behaviour, about 800 lines)

- **A5. Empty search.** `empty_state` (`app.rs:1677-1709`) checks only the column filters, but search
  hides rows too (`app.rs:1887-1890`). A search with no match says "No downloads here / This category
  or queue holds no downloads yet". Its link (`Cmd::ShowAll`, `app.rs:6874`) keeps the search, and in
  All view does nothing. Add a first branch for `find.active()`: "No downloads match "{text}"" (the
  text through `isolate`) with "Clear search", which clears the Find field.
- **B4. Copy address.** No command copies a download's address. Today it takes Properties' read-only
  field (`properties.rs:274-286`) or the progress window's values (`progress.rs:1134`); torrents have
  "Copy magnet link" (`app.rs:7754`). Add "Copy address" to the row menu (`item_entries`,
  `app.rs:7640`) for downloads that aren't torrents. Ctrl+C runs it the way Delete does, as a
  shortcut that works "with nothing focused" (`shortcut_cmd`), since the list never holds egui focus.
  The shortcut table test (`app.rs:12805`) gains a row.
- **E2. Refresh from a failed download.** The failed state offers only details, open page and Resume
  (`progress.rs:538-584`). `Error::LinkExpired` tells the user to "(refresh the download address)"
  (`errors.rs:74-76`), but `Cmd::RefreshAddress` exists only in the row menu (`app.rs:7660-7665`).
  Add a "Refresh download address..." button to the Details pane for LinkExpired and HTTP 403/410
  (`Error::Status`, `errors.rs:14-17`). The progress window's button is moved to 13 (13b); both use
  the same label and predicate.
- **C4. One rule for Open folder.** The Details pane's button is always enabled (`panes.rs:1374-1381`);
  the row menu enables it only when the download is done or a torrent (`app.rs:7645-7649`). Both run
  `Cmd::OpenFolder`. Share one predicate, the row menu's.
- **C7. Grey an OK that can't work.** Add trackers' OK is always enabled and validates on click with
  an inline error (`torrentdlg.rs:1498`, `1507-1511`). RSS "Add feed" is always enabled, and an empty
  box silently does nothing (`rssdlg.rs:171`, `176`). Grey them with a tip, as Category, Queue name,
  Login, Exception, Batch and Add do. Add trackers' Enter stays a new line.
- **D1. First field focused.** These open with nothing focused:
  - the Options sub-dialogs: login (`options.rs:3153-3172`), exception (`:3198`), account (`:3419`),
    addresses (`:3453`), DNS (`:3400`) and keys (`:3504`)
  - Download a gallery (`gallerydlg.rs:163-170`)
  - Link grabber's address box (`grabber.rs`)
  - RSS "Add feed" (`rssdlg.rs`)

  Give them the same once-flag `request_focus` the rest already use: Add (`adddlg.rs:246-247`),
  Batch (`batch.rs:269-271`), Category (`categories.rs:106-108`), Checksum (`checksum.rs:122-124`),
  Download File Info (`dialogs.rs:1286-1288`), Archive password (`extractq.rs:649`), Queue name
  (`scheduler.rs:1382-1387`), Add torrent (`torrentdlg.rs:495`), Add trackers (`:1467`) and Video
  (`videodlg.rs:639`).
- **E1. Show the Speed Limiter is on.** Only the menu and tray check marks show it
  (`tray.rs:323-326`, `app.rs:7154-7161`), not the status strip (`app.rs:1761-1847`). Upstream
  already built this: b3476e57 (upstream issues 1191 and 1195), where the status bar says while the
  Speed Limiter is on and the Transfer rate names the limit holding a download back. The ledger
  skipped it (`tools/upstream-sync/ledger.md:194`, "revisit on request").
  - **Port it through `/upstream-sync`** instead of writing new code, and fit it to Graphite's status
    strip.
  - It adds about 200 lines to `model.rs` near `rate()`, so it goes before F8's unit changes (12c).
- **A3. Drop "&Pause All".** It runs `Cmd::StopAll`, the same as "&Stop All" (`app.rs:7126-7127`).
  Remove the item, and change the icon map (`app.rs:374`), CHANGELOG and docs to match.
- **C1. Yes/No order.** "Files already there" makes No the default on purpose (`dialogs.rs:3326`,
  `3334-3338`). `footer_slots` (`winui.rs:4019-4046`) moves the primary button (Default, Accent or
  Danger, `winui.rs:3934-3937`) to the right, so this dialog reads "Yes, No" while Confirm deletion
  (`dialogs.rs:2490`), Confirm (`2985`) and Create folder (`3183`) read "No, Yes".
  - Fix: let `FooterButton` turn `primary` off, and use that on this No. It keeps its Default look
    and its Enter/Escape flags.
  - Leave `footer_slots` alone, because every other footer relies on it.
  - Swap the declared order so Tab matches what the user sees, and keep the `Some(0) => Done`
    mapping (`dialogs.rs:3342`) right.
- **C9 (order).** On Options → Connection, the exception buttons go New, Delete, Edit
  (`options.rs:1355-1376`). Make it New, Edit, Delete, and change the Tab order with it.
- **D5 (weekdays).** The weekdays run down columns (Sun/Mon, Tue/Wed…, `scheduler.rs:773-781`). Lay
  them out in rows in reading order, and change the Tab and arrow order with them.
- **DC8. Properties gets Cancel.** It has editable fields and only OK; Escape discards silently
  (`properties.rs:447-461`). Add Cancel (Escape), the footer every other editing dialog has.
- **DB1. Add gets Cancel.** Add has "Later" and "Add N" with no Cancel; only Escape closes it
  (`adddlg.rs:465-470`, the fork's own file). Add Cancel, as in its sibling dialogs. The footer's
  index mapping (`adddlg.rs:472-473`: `Some(1)` Later, `Some(2)` Add) shifts with it.
- **DB3. Remove the dead View items.** "Toolbar", "Tray icon", "Customize URL List..." and "Font" are
  always disabled, with empty submenus (`app.rs:7257-7259`, `7267`; menu lines, low cost). Remove
  them. Bring each back when it works.
- **B6 (hygiene).** The comment "can't be had back" (`scheduler.rs:424`) is stale: Discard rebuilds
  the Scheduler (`app.rs:5999`, `6010-6013`). Fix the comment; the question stays.

## 12c · Words (look, about 900 lines)

Words only: no command, route, key, Tab order or enabled rule changes (those are in 12b). Docs that
quote a changed label change with it (`docs/*.md`, e.g. `docs/getting-started.md:63`), and so does
the mockup. Most edits are menu, toolbar and footer lines (low cost); in high-churn files, the edit
is one table entry or one function.

Some icons are looked up by a label's text. Grep these tables with every rename, or the item loses
its icon: `chrome::menu_icon` (`chrome.rs:1656-1673`, e.g. "On Double click"), `app.rs:374`, and
`themeicons.rs:120` ("Delete from queue").

**One label per command** (menu lines and the toolbar table: low cost):
- **A2/B2.**
  - `Cmd::Resume`: "Resume" everywhere. Today it is also "Resume Download" (row menu `app.rs:7657`)
    and "&Download Now" (Downloads menu `app.rs:7116`). That frees "Download now" to mean only
    "start a new one" in the add dialogs.
  - `Cmd::Stop`: "Stop" (today "&Stop Download", `app.rs:7111`, `7658`).
  - `Cmd::Remove`: "Delete", matching the key, the toolbar (`app.rs:8450`) and "Confirm deletion".
    Today it is "&Remove" in the menus (`app.rs:7113`, `7671`). "Remove with files" becomes "Delete
    with files" (`7115`, `7673`).
  - `Cmd::DeleteCompleted`: "Delete completed" (today also "&Delete All Completed", `app.rs:7130`,
    `7584`).
  - Access keys in the Downloads menu: "&Stop", "&Delete", "Delete with &files", "&Resume",
    "R&edownload". The test `no_two_items_of_a_menu_share_an_access_key` (`app.rs:13220`) must pass.
- **A4. Moved to 13 (13b).** The progress window's "Paused" (`progress.rs:1187`) and "Pause"
  (`progress.rs:568`) become "Stopped" and "Stop" there. Nothing changes here: the list already
  says "Stopped" (`model.rs:1246`), which stays the Status filter value (`list.rs:134`).
- **`Cmd::TorrentReannounce`: "Ask trackers again"** (today "Reannounce", the Torrent submenu,
  `app.rs:7736`). Plan 13's torrent view uses the same name, so the rename is done once, here. Its
  failure message follows E5.
- **DF4.** "Delete from queue" (`app.rs:7684`; it deletes nothing) and "Remove from the queue"
  (`scheduler.rs:1227`) both become "Remove from queue".

**Capitalisation: sentence case everywhere** (PR 93's rule, extended from Options page names):
- **F1.**
  - Toolbar: "Stop All", "Delete Completed", "Start Queue", "Stop Queue" become "Stop all", "Delete
    completed", "Start queue", "Stop queue" (`app.rs:8448`, `8453`, `8463`, `8470`).
  - Menus: "Find Next" (`7137`), "Find Previous" (`7139`), "Grabbit &Home Page" (`7204`),
    "On Double click" (`7687`, with its `chrome::menu_icon` key) becomes "On double-click", and the
    Arrange files submenu ("By &Order Of Addition" `7487`, "By &Time Left" `7491`, …).
  - Columns: "File Name" and "Last Try Date" (`list.rs:57`, `66`; `panes.rs:794`; the Scheduler's
    own "File Name" column, `scheduler.rs:1104`) become "File name" and "Last try date". Saved
    layouts are kept by slot, not title (`list.rs:52-54`), so this is safe. Tests that assert the
    old title change with it (`list.rs:2043`, `2261`, `2272`).
  - Window titles: "Download File Info" (`dialogs.rs:1198`) and "File Properties"
    (`properties.rs:72`) become "Download file info" and "File properties".
  - Options: "Browser/System Integration" (`options.rs:4306`) becomes "Browser and system
    integration".
  - The whole Arrange files submenu (`app.rs:7487-7498`, 12 items): "By &Order Of Addition",
    "By &File Name", "By &Time Left", "By Transfer &Rate", "By &Last Try Date", "By Date &Added",
    "By Save &Path", "By Parent &Web Page" become "By &order of addition", "By &file name", and so on.
    Keep each access-key letter.
  - Found by the second casing sweep (below):
    - Download File Info's "Save As" (`dialogs.rs:1208`, `1282`, and the hint "Type the file's name
      in Save As", `1467`) becomes "Save as". The label keeps its meaning (F2).
    - Its buttons "Start Download" / "Download Later" (`dialogs.rs:1497`, `1492`) follow B1 ("Download
      now" / "Later" / "Cancel"). The queue question's own "Download later" (`dialogs.rs:2852`) stays
      as it is.
    - "Downloaded {size} ({bytes} Bytes)" (`dialogs.rs:1672`): moved to 13, whose Done window drops
      the byte count.
    - "Customize Grabbit Download panels in browsers" (`options.rs:698`) becomes "Customize
      Grabbit's download panels in browsers".
    - "Note: These settings don't relate to queue processing" (`options.rs:1150`) becomes "Note:
      these settings…", matching the app's lower case after a colon.
    - "Extend the availability of Manifest 2 Chrome extensions" (`options.rs:516`): the name is
      "Manifest V2".
    - The Themes help page's icon list (`themeicons.rs:139`) still says "Options: Sites logins".
      Use "Site logins" (PR 93's name).
  - Proper names stay as they are: "Speed Limiter", "Scheduler", "Link grabber" (PR 93), "Recycle
    Bin", "Windows Credential Manager", and browser product names ("Brave Browser", "Microsoft Edge
    Legacy", "Zen Browser"). So does a window or field named inside a sentence ("the Add window", "the
    Find field", "Type a letter in From and in To"), as the app already writes them.
  - **The second casing sweep** (2026-11-21) scripted a pass over the 1,914 capitalised string
    literals outside test modules and logging. It flagged every string with a capitalised word that
    isn't a proper noun, and every text written with two casings ("Start Queue" on the toolbar vs
    "Start queue" in the Downloads menu, `app.rs:8463` vs `7150`; "File Name" vs "File name" in Find
    and Video; "Download Later" vs "Download later"). Everything it found is listed in this section.
    Sample and test data (yt-dlp samples in `progress.rs:109`, `videodlg.rs:315-317`), the country
    list and clipboard format names are not UI.
  - Docs: the old names appear 48 times in 16 help pages (`docs/adding-downloads.md` 9,
    `docs/download-list.md` 9, …). They change in the same part, and the shot-based help pictures are
    retaken after 12d.

**Dialogs:**
- **B1.** The add-now/add-later pair has four names:
  - Add: "Add N" / "Later" (`adddlg.rs:465-470`)
  - torrent, video, gallery: "Download Now" / "Later" / "Cancel" (`torrentdlg.rs:707-715`,
    `videodlg.rs:1029-1038`, `gallerydlg.rs:355-360`)
  - Download File Info: "Start Download" / "Download Later" (`dialogs.rs:1492-1507`)
  - Link grabber: "Start" / "Add to queue" (`grabber.rs:671-681`)

  All of them become "Download now" / "Later" / "Cancel" (footer lines: low cost). Add may keep its
  count: "Download N".
- **C2.** Category delete asks Delete (Danger) / Cancel (`dialogs.rs:3083-3086`); download and queue
  delete ask Yes / No (`dialogs.rs:2490`, `2985-2988`). Use Yes / No.
- **C6 (words).** Link grabber's "Check all" / "Uncheck all" (`grabber.rs:639-641`) becomes "Select
  all" / "Select none", as in Add torrent and Playlist (`torrentdlg.rs:607`, `620`;
  `playlistdlg.rs:540`, `550`).
- **F9. Moved to 13 (13b).** The progress window's "<< Hide details" / "Show details >>"
  (`progress.rs:490-492`) become "Hide details" / "Show details" with a chevron there.

**Options and other labels:**
- **C9.** On Options → Connection, "Max. connections number" is drawn as a field label
  (`options.rs:1261-1262`, `form_label`), but the comment at `options.rs:1256` intends a group title.
  Draw it like the page's other group titles. The button order is in 12b.
- **D5.** "Number of retries for each file if downloading failed :" (`scheduler.rs:857`) loses the
  space before the colon. The weekday layout is in 12b.
- **F2.** "Save to:" (`grabber.rs:1012`, `rssdlg.rs:628`), "Save To:" (`properties.rs:150`, `215`)
  and "Save To..." (`options.rs:864`) become "Save to". The progress window's "Save To:"
  (`progress.rs:951`) goes with its Options on completion tab in 13. Download File
  Info's "Save As" (`dialogs.rs:1282`) stays: it edits the full path and the name.
- **F3.** Labels drop their colons, as Options mostly writes them (`options.rs:866`, `1568`, `1583`)
  and the add dialogs do. Remove them at `options.rs:1297`, `1682`, `2688`, `2727`, `3209`, `3764`,
  `3942`, and in the grabber, RSS and Category dialogs.
  Built (12c): these, and the login, exception, DNS and account windows' labels. Kept with their
  colons: Options → Regional, the video cookies ("From browser:", "Profile (optional):"), Batch
  download, Properties and the queue panel's "Add to queue:" (columns of read-outs, or pages in
  high-churn files).
- **F4.**
  - "UserName" becomes "User name" (`options.rs:1562`, `1584`).
  - "Get System" becomes "Get system settings" (`1482`).
  - "Max. connections number" becomes "Maximum connections" (`1261`).
  - "Temporary directory" (`1005`) and "Default download directory for…" (`842`, `928`) say
    "folder".
  - "Advanced / Socks..." (`1645`) becomes "Advanced / SOCKS...", matching the page name.
  - "Open &torrent file..." (`app.rs:7045`) and "Open a .torrent file..." (`adddlg.rs:345`, `850`)
    use one form.
  - "Create new queue" (`app.rs:7627`, `7832`) and "&New queue" (`scheduler.rs:476`, `484`) use one
    form.
  - "Always turn on Speed Limiter on Grabbit startup" (`dialogs.rs:1995`, `settingspage.rs:1003`)
    becomes "Turn on the Speed Limiter when Grabbit starts".
- **F5. Ellipses.** The rule: "..." (three ASCII dots) on an item that asks for more input before it
  acts. PR 93's rule stays: no "..." on items that open an Options section.
  - Add the dots to "&Add new download" (`app.rs:7044`, matching `7567`), "Add &batch download"
    (`7048`), "Add batch download from &clipboard" (`7050`), "Add category" (`7790`) and
    "Move/Rename" (`7652`).
  - "Create new queue" (`7627`, `7832`) opens a name prompt, so it takes the dots.
  - Use "..." for "…" at `app.rs:7310` and `crashui.rs:483`, `495`, `510`.
  - Remove the space in "Edit list ..." (`options.rs:803`, `809`).

**Messages, numbers and units** (each changed once, in its shared function):
- **E5.** Errors use one form, "The X could not be Y: {error}". Bring these into line:
  - "could not" / "couldn't" mixed: `help.rs:370` vs `486`; `model.rs:532`, `745`; `videodlg.rs:1273`
  - `checksum.rs:154` "Cannot read the file"
  - `extractq.rs:393` "Couldn't extract"
  - `extension.rs:664` "Setting it up failed:"
  - `app.rs:5445`, `5447` "Recheck failed:" / "Reannounce failed:" (the second names the renamed
    command: "The trackers could not be asked again: {error}")
  - `progress.rs:1192`, `2221` "Error: {error}": moved to 13, which writes the reason as its own line
- **F8.** Numbers say the same thing the same way:
  - Sizes: one precision for a given quantity. `size` (2 decimals, `model.rs:1359-1371`) and `size3`
    (3 decimals, `model.rs:1373-1384`) show the same file as "6.00 GB" in the list and "6.000 GB" in
    the progress window. Default: `size3` follows `size`, and the tests change with it. (Plan 13's
    window calls `model::size`; the function change stays here.)
  - Percent: the progress window's "( 1.58 % )" (`progress.rs:1241`) is moved to 13, whose reading
    and detail line replace it.
  - The double space before a unit, from `dialog_size` (`dialogs.rs:683-686`) and `size_list`
    (`model.rs:1395-1398`), becomes one space.
  - Units: "KBytes/sec" (`dialogs.rs:1948`, `2025`; `torrentdlg.rs:1301`; the progress window's
    `progress.rs:815`, `913` are moved to 13, which writes "KB/s")
    and "MBytes" (`quota.rs:12`, `dialogs.rs:2162`) become "KB/s" and "MB", matching the rates'
    style (`model.rs:1386-1393`; "MB/sec" there becomes "MB/s").
  - Costs a few lines in model.rs and dialogs.rs (high churn), but each is one function. It goes
    after the E1 port (12b), which also adds to `model.rs` near `rate()`.
  - Tests that change: `rate` (`model.rs:1597`), plus the tray tip and the status strip ("MB/s").
  - Checked and unaffected: settings store numbers, so the units are labels only; sorting is
    numeric (`sort_key`, `app.rs:1216`); no parser reads the size or rate text.
  - The remote web page (`crates/remote/web/app.js:34-35`) writes "KB/sec": align it in the same
    part.
- **D4 (date).** The list's date "%b %d %H:%M:%S %Y" is cut in the Added column.
  - **`STAMP` (`model.rs:1314`) is also the storage format.** `now_stamp` writes it and
    `stamp_time` reads saved lists with it (`model.rs:1308-1319`). It must not change.
  - Change only the display, `stamp_text` (`model.rs:1344`): drop the seconds, and show the year
    only when it isn't this year. Use `crate::clock::now()` for "this year" so tests stay
    deterministic.
  - The tests asserting `stamp_text(at(s)) == s` (`model.rs:1616-1628`) change with it.
  - The Options → Regional custom format is unchanged.

## 12d · Visual system (look, about 1,200 lines)

Changes made at the helper where possible. Costly items are last and can be split off.

- **C5. One browse control.** The control-height square `gx-dots` icon button (tooltip "Browse")
  everywhere. Today there are also:
  - text "Browse..." (`categories.rs:142`, `options.rs:2572`)
  - text "Browse" (the Options Save to page, `options.rs:956`, `1014`)
  - 23 px icons (`grabber.rs:1058`, `scheduler.rs:898`)
  - a 24×22 icon (`rssdlg.rs:639`)
- **D4 (widths).** `GRAPHITE_WIDTHS` (`list.rs:96-97`) sets File Name 216, Status 160, Time left 84
  and Added 140. "{m} min {s} sec" (`model.rs:1405`) is cut in Time left. Widen Time left (and Added
  if the 12c date still needs it), taking the width from Status, which has room. Keep
  `the_default_columns_fit_the_first_window` (`list.rs:2334`) passing. Only users who never resized
  a column get the new widths (`list.rs:340`).
- **G2. One separator.** The 2 px etched line (`properties.rs:302`, `317`, `380`;
  `scheduler.rs:504`, `699`, `707`, `833`, `871`; `dialogs.rs:2000`) becomes the 1 px hairline.
  Change the bodies of `etched_h`/`etched_v` (`winui.rs:5588-5601`) to a 1 px `fill_l` in
  `menubar_line`, as `rule()` draws. They take absolute coordinates, `rule` takes `o + x`, so this is
  a body swap with no call sites touched. It applies to every theme: take a shot in a non-Graphite
  theme too. (`progress.rs:982` goes away with plan 13's removal of the Options on completion tab.)
- **G3. Focus rings only from the keyboard.** Find's filter chips (`find.rs:531-537`) and the
  light/dark button (`chrome.rs:909-915`) draw an accent ring on `has_focus()`. Use
  `winui::focus_ring_round`, which checks `keyboard_in_use` (`winui.rs:4836-4839`).
- **G4. Find's box like every text box.** It is hand-drawn (`find.rs:238-246`): a `window` fill
  instead of `field`, and a 1 px full-accent border on focus instead of Graphite's field focus ring
  (`winui.rs:2075-2081`). Use the field roles and the shared focus look. Upstream hasn't touched
  `find.rs`.
- **E4. One loading look.** The spinner and a line (`torrentdlg.rs:111`, as in gallery, video and
  playlist) for:
  - checksum (`checksum.rs:176`)
  - update check (`help.rs:357`)
  - helpers (`helpers.rs:285`)
  - Options' default browser (`options.rs:1208`)
  - the File Info size (`dialogs.rs:1220`)
  - a busy RSS feed (`rssdlg.rs:200`)
- **G5.** The crash window's heading beside its icon tile is 13 (`crashui.rs:416`, `tahoma_bold`);
  Options page headings are 16 (`options.rs:4620`, `tahoma_title`). Use `tahoma_title`.
- **G6.** The pairing code is 26 bold (`options.rs:3063`); the sign-in code is monospace `font_code`
  22 (`options.rs:4021`). Use `font_code` for both.
- **G10.** Tick strokes are 1.3, 1.4 and 1.5 (`chrome.rs:2249`, `winui.rs:5027`, `5055`,
  `find.rs:528`). Use one metric.
- **C6 (layout), costly.** Link grabber (`grabber.rs:1035`, `1061`) and RSS feeds (`rssdlg.rs:161`,
  `168`, `631`) place 22–23 px boxes at fixed x. Move them onto `winui::form`, `CONTROL_H` and `EDGE`.
  RSS is cheap (1 upstream commit). The grabber is a re-layout of a medium-churn file: do it last, in
  its own commit, so a later grabber pick can be ported around it.
- **G9. One card frame, costly-light.** Radii 8 (`helpers.rs:521-527`), 6 (`options.rs:4014-4019`,
  `adddlg.rs:278-283`) and square (`grabber.rs:697`, `torrentdlg.rs:742`). Add `winui::card` (radius
  from metrics) and use it at these five places.
  - **As built (12d):** `winui::card` (`radius.card`) frames the three cards (Helpers, the
    sign-in code, Add's list) and `file_card`. Link grabber's and Add torrent's lists stay
    square: they are `winui::row_list`s, which paint their own square frame and row fills over
    the box; rounding them means rounding every row list in `winui.rs` (high churn), and the
    rows' fills would still poke out at the corners.
- **G7. Paragraph line height: only where the user sees it.** The same 11 pt text uses line heights
  of 13, 14, 15, 16 and 18. `winui.rs:797` takes a raw `line_h`; examples: `dialogs.rs:3279`,
  `helpers.rs:327`, `crashui.rs:432`, `extension.rs:390`. Add one metric (13), and use it in the
  fork's own files and in files you touch anyway. Don't sweep all 44 call sites through high-churn
  files.
- **Not done** (nothing visible, and they would add churn in shared files): replacing the Details
  pane's literal fonts (`panes.rs:998-1007`, equal to the metrics today), a `"geist-medium"` helper
  (7 places), `help.rs:373`'s hand-written `fit`, `chevron_light`'s literal 1.2, and the 17
  `o.x + 11.0` insets.

## 12e · After the owner's decisions

DA6 and DA1t, if the other option is chosen. Each is behaviour and gets its own PR.

## Dropped (checked, no change)

| ID | Finding | Why dropped |
| --- | --- | --- |
| B5 | No speed-limit presets in the tray or menu | A new feature, not consistency; add it to ROADMAP if wanted |
| B6 | The Scheduler page asks before deleting a queue | Asking costs nothing (12b fixes the comment) |
| C3 | The progress window fills both Pause and Resume | Moved to 13, which sets the rule: only a state's main action is filled |
| D2 | No dialog can be resized | Window scaling is out of scope |
| D3 | Add 560, torrent and video 600, gallery 520 wide | They hold different content |
| D6 | The crash report's hint is cut | By design, readable on hover (#164) |
| F6 | Shortcuts written into menu labels | Not visible: `chrome::without_shortcut` (`chrome.rs:1148-1164`) strips them |
| F7 | Menu access keys | Accessibility: out of scope |
| G12 | The list and Details bars are drawn by hand | They do a different job (status colours, segments); not missed reuse |

## Checks (every part)

00-conventions' checks 1–6: fmt, clippy, test, the extension test where touched, release build and
run, and a keyboard pass in every changed window. The external audit loop: a fresh read-only auditor
until APPROVED. Screenshots per part on `pr-shots`. Help pictures are retaken once, after 12d, not
per part; pictures of the progress and Done windows are left to plan 13, which retakes them after 13d. After each part, `python tools/upstream-sync/sync.py status --fetch`: any newly
conflicting upstream commit is noted in the PR.

## Definition of done

- [ ] 12a–12d merged into `dev`, each audited until APPROVED
- [ ] The owner's decisions recorded in this file; 12e merged if needed
- [ ] ROADMAP and CHANGELOG lines per part; help pictures, docs and the mockup updated for the
      changed names
