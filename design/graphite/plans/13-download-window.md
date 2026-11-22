# 13 · The download window and the Done window

Spec written on dev `ad995ea8`, 2026-11-21, from the owner's review of the current window, a design
advisor's direction, the mockup canvas "Download window mockups" (14 boards, in the owner's claude.ai
artifacts: https://claude.ai/artifact/74JwJKN7NCwNjDtQAKTCx8), and three independent audits of that
canvas (visual, flows against the app, accessibility and copy), each APPROVED after two rounds of
fixes. Line numbers drift: find each place by the name next to it.

**Goal.** The per-download window stops looking like IDM's and shows what matters at a glance: one
large reading and one exact strip drawn from real data, with everything else one click away. The Done
window opens the right thing.

**Scope.** `crates/app/src/progress.rs` (the progress window), the Done window in
`crates/app/src/dialogs.rs`, the Torrent submenu in `app.rs`, and the theme tokens they use. Out of
scope: the main window, the list's own progress bars, the website, the extension.

**Rules.** 00-conventions applies: behaviour parts and look parts are separate PRs, each audited until
APPROVED, with before/after screenshots (light and dark) and ROADMAP and CHANGELOG lines. The canvas
replaces `../grabbit-design.html` as the reference for these two windows; each part names the boards
it builds. Graphite stays the visual system; this plan reopens only these two windows' layout.

## Upstream merge cost (plan 12's balance rule)

`progress.rs` (22 upstream commits since the fork point) and `dialogs.rs` (38) are high churn. This
plan rewrites the progress window's painting, so later upstream fixes to it become hand ports. To
keep that cost down:

1. Keep the data side where it is and untouched: the status words and states (`status_page`'s match,
   `finishing_fraction`, `item.finishing`, `torrent_checking`, `at_service`, `video_helper`), the
   torrent details fetch (`torrent_details`' `d`), `whereis::line`, `window_title`, `caption_buttons`,
   `on_escape`, the frame and tray code in `show`. Rewrite only the painting under `body`.
2. Move the new layout into a new file (`progress_view.rs`) that `body` calls. Upstream picks that
   touch the data functions still apply cleanly; picks that touch painting are ported by hand.
3. The Done window: change only its body and footer inside the struct with `opening`/`moved_away`;
   `show_in_folder_now`, `open_download_now`, `open_download_with` and the drag-out stay.
4. Note each later upstream pick that conflicts with a plan-13 change in
   `tools/upstream-sync/ledger.md`, naming the part.

## Overlap with plan 12

Each piece of work is in one plan only. Plan 12 marks these "moved to 13"; plan 13 does them:

| Plan 12 item | Done in plan 13 |
| --- | --- |
| A4 | "Stopped" / "Stop" in this window (`progress.rs` status match, Pause button), 13b |
| E2, the progress window's half | "Refresh download address..." as the expired-link state's main button, 13b |
| F9 | "Show details" / "Hide details" with a chevron, 13b |
| F8, this window's lines | The percent (`progress.rs:1241`) and "KBytes/sec" (`progress.rs:815`, `913`), 13b-13c |
| E5, this window's lines | "Error: {error}" (`progress.rs:1192`, `2221`) becomes the reason line, 13d |
| F2, this window's line | "Save To:" (`progress.rs:951`) goes with the Options on completion tab, 13b |
| F1, the Done window's byte count | "({bytes} Bytes)" (`dialogs.rs:1672`) is dropped, 13a |
| C3 (dropped there) | The filled-button rule, 13b |

Plan 12 keeps, and plan 13 only uses:

| Plan 12 item | How plan 13 uses it |
| --- | --- |
| E2, the Details pane's half | Same label and predicate as this window's button |
| F8, `size3` follows `size` | This window calls `model::size` |
| G2, the `etched_h` body swap | Nothing to do here: this window has no etched line left |
| B4, Copy address in the row menu | The From row's copy button runs the same copy |
| 12c, "Reannounce" renamed "Ask trackers again" | The Trackers tab's button uses that name |
| Help pictures retaken after 12d | They skip these two windows; 13 retakes them after 13d |

Order: 13a can land any time. 13b-13d land after 12c (words), so every string is written once.

## Decisions (owner, 2026-11-21)

- Two levels: compact, and Details that widens the window to 760 to the right. No middle level.
- No XP-style block progress bar: the strip shows real connections or pieces.
- "Stop sharing" for one torrent lives only in the main window's right-click Torrent submenu, for a
  finished torrent that is sharing. Not in these windows.
- Add trackers, Ask trackers again, Recheck files, Download in order, First and last pieces first and
  Copy magnet link are reachable from the expanded torrent view.
- The "Hide to the notification area" title-bar button stays.

## The design (boards 1-14)

- **Compact (board 1, 6).** 440 wide. Name (13/500); the reading (28/500, percent or a word) with time
  left at the right; a 6 px strip; a detail line (size so far, speed); footer: Show details at the
  left, Stop and Cancel at the right. Two edges only: x 20 and the right edge.
- **The strip.** A file download: the engine's segments (`Snapshot::segments`, byte ranges `start`,
  `cursor`, `end`), each as wide as its share of the file, as today's `segment_bar` draws them;
  each filled from its start to its cursor, with a 2 px tick at the cursor of a segment still being
  written. Segments are not connections: there can be more of them than connections (up to
  `MAX_CONNECTIONS`, 32; 128 when unlocked, `download.rs:84-87`). When a segment is narrower than
  3 px, drop the gaps and ticks: one plain fill. A torrent: pieces bucketed into columns, lit by
  how much of each column is in. A video or a gallery (one progress for the whole job,
  `tool_snapshot`): one plain fill. One progress value for screen readers; segments hidden.
- **Expanded (boards 2, 3, 10-12).** The compact block stays in place at the left (400); the right
  column (288) holds the detail.
  - File download: Saved to (folder button), Server, Can resume, From (copy address), Limit
    (with "Remember for this file" and its note), When done; right: speed graph for 60 s with the
    limit as a dashed line, Connections (fixed height, scrolls; receiving first, then retrying,
    connecting, done).
  - Torrent: Saved to, From (copy magnet link; its tooltip shows the info hash), Peers ("12
    connected, 48 known": `peers_live`, `peers_seen`), Sent, Ratio, **Pieces ("98 of 1,470, 512 KB
    each": the exact count the strip can't show, so a torrent stuck on its last piece is visible)**,
    Limit (Down, Up), When done with "Then keeps sharing"; right: Files / Peers / Trackers tabs.
    Files has Download in order, First and last pieces first, Recheck files; Peers lists address,
    client, received, sent; Trackers lists the trackers (no status: the engine gives only their
    addresses) with Add trackers... and Ask trackers again.
  - Video: Format and "Downloaded by" rows; the right column has the speed graph only (one progress
    for the whole video, no connections or parts).
- **When done** (every download, torrents too): Show the Done window, Do nothing, Exit Grabbit, Shut
  down, Sleep, Hibernate; "Force apps to close" under the last three. Hang up modem is dropped.
- **States (boards 4, 8, 9).** Stopped (grey strip, Resume filled), error (hatched missing part, the
  reason in a line; an expired link offers Refresh download address..., other errors Retry), joining
  picture and sound (percent when known, no time left), size not known (the reading is the amount
  received, empty strip), waiting ("Waiting in Main queue", Stop greyed), finishing, torrent checking
  files (bytes checked as a plain fill: piece-by-piece results aren't known until the check ends),
  sharing ("Sent 308 MB, 6 peers connected", "shared for 1 h 12 min"; Close only), the site sent a
  web page (Open in browser filled), at a debrid service (named, as `at_service` gives it: "62% at
  Real-Debrid"), and the server can't resume (after Stop: Keep downloading filled, Stop anyway).
- **Done window (boards 5, 7, 13, 14).** Same layout as the progress window: name, "Done" with the
  size, a full strip, the file count, Saved to (opens the folder), Main file, "Don't show this
  window again", footer. No "took 5 min": the app doesn't keep when a download started.
  - A main file exists when the largest file is at least half the total **and** at least 4 times the
    second largest (size only). Then: [Open ▾] filled, [Open folder], [Close]; the menu lists the 6
    largest files (the main file checked), then Open with...
  - No main file: [Open folder] filled, [Open file ▾], [Close]; the menu lists the 6 largest and
    "N more in the folder".
  - One file: [Open ▾] (Open with... under the arrow), [Open folder], [Close].
  - Open folder opens the download's folder with the main file selected; for a single file, its
    folder with the file selected (today's `show_in_folder_now`).
  - The name row can be dragged out (today's drag strip), takes focus, and Ctrl+C copies the file.
  - A file moved or deleted: the reason under Saved to, Open greyed, Open folder filled. A file
    dragged out to an unknown place keeps today's message ("Look for it where you dropped it").
  - The notice line (such as subtitles the site refused) stays, under Saved to. The byte count goes.
- **Keys** (unchanged where they exist). Progress window: Esc closes it and the download goes on
  (`on_escape`), Enter does nothing, only the Cancel button cancels. Done window: Enter runs the filled
  button, Esc closes, Alt+O and Alt+F open. Tabs: arrows move, only the selected tab is a Tab stop.
- **Filled button rule.** A state's one main action is filled; Stop and Cancel never are.
- **Dropped on purpose:** the three top tabs and Hide tab / Show hidden tabs (#326); the Pieces and
  General tabs (the strip is the piece map, the Pieces row keeps the exact count, the From row's
  tooltip shows the info hash; the rest of General is in the facts rows and the Sharing state); the
  "Start positions and download progress by connections" caption. Kept: the title bar's percent,
  rounded down as today (`window_title`: 6.7% reads 6%).
- **Data.** Every value shown comes from what the app already has (`Snapshot`, `ConnView`,
  `torrents::Details`, `mdm_torrent::Stats`, the item), with one thing it must start keeping: the
  last 60 s of speed, sampled once a second per open window, for the graph (13c). It answers
  whether the speed limit is holding a download back, costs 60 numbers per open window, and is
  never saved.

  Not shown, because the app doesn't have it: a waiting download's place in its queue (a queue runs
  several downloads at once and the Scheduler can start or stop it, so "3rd" could mean next or in
  an hour; the window names the queue instead), a torrent's seeds, each tracker's status, a video's
  separate parts, how long a finished download took. Each could come later with its data.
- **Tokens.** Hover `#222227` (exists), error track `#3a1b1b`/`#5a2a28`, accent track `#15302a`, plus
  their light values; numbers in the `numbers` family (plan 05).

## 13a · The Done window opens the right thing (behaviour, about 500 lines)

The bug: for a download saved as a folder (a torrent with several files), Open opens the folder and
Open folder opens its parent, so both "open a folder" (`dialogs.rs`, the Done window's footer;
`show_in_folder_now`).

- Compute the main file from the torrent's file list (`torrents::details_of` or the spec's files) with
  the rule above; one bool and one path. Single-file downloads: the file is the main file.
- Footer and menu per the three cases above; Open with... moves into the menu.
- Open folder selects the main file inside the download's folder.
- Drop the "({bytes} Bytes)" part of the Done window's size line (`dialogs.rs:1672`).
- Tests: the main-file rule (one big file, an album of equal files, a 60 MB + 50 MB pair, one file);
  Open and Open folder targets for a folder download.
- Boards 5, 7, 13, 14 (behaviour only here; their look lands in 13d).

## 13b · Progress window structure (behaviour, about 900 lines)

- Remove the three top tabs, Hide tab and the tab-row menu; the window has the compact view and
  Details (expanded to 760, to the right). Keep the window on screen when it widens (it moves left
  when 760 would cross the screen edge); below that, today's scrolling (#262) still applies.
- Move Speed Limiter (with Remember for this file and a torrent's upload limit) and the on-completion
  choices into the expanded view; one When done list; drop Hang up modem (and its `queue::WhenDone`
  field if nothing else uses it).
- Stop/Stopped (A4); Refresh download address... on an expired link (E2); Show/Hide details (F9);
  the percent and "KB/s" in this window (F8's lines here); the filled-button rule (C3).
- The can't-resume confirm when Stop is pressed on a download whose server can't resume.- The torrent commands (Add trackers..., Ask trackers again, Recheck files, the two order options,
  Copy magnet link) call the existing `Cmd::Torrent*` commands.
- Main window, Torrent submenu: add "Stop sharing" / "Start sharing" for a finished torrent that is
  sharing, kept per torrent (a list of excluded hashes that `set_sharing` honours, saved with the
  settings). The rename of Reannounce is plan 12's (12c).
- `/show progress:<state>` names: keep status, error, merging, helper, files, peers, trackers; drop
  menu, limit, done; add stopped, unknown-size, waiting, finishing, checking, sharing, webpage,
  service, cant-resume, details.
- Tests: Esc never cancels (`on_escape`); the When done mapping; Stop sharing persists across a
  restart; the window stays on screen when expanded.

## 13c · Progress window look (look, about 1,200 lines)

- The compact layout, the reading, the strip (connections, pieces, the plain fill above 48), the
  detail line, the footer, the two edges and the 4 px spacing scale (boards 1, 2, 3, 6, 10-12).
- The speed graph (last 60 s, limit line), fed by a new sampler: 60 readings of the item's rate,
  one a second, kept per open window and dropped when it closes. The tables (fixed height, sorted
  rows). The torrent facts with the Pieces row and the From row's info-hash tooltip.
- Tests: the sampler keeps 60 readings and starts empty; segment widths follow byte ranges and
  fall back to one fill below 3 px; the Pieces row counts `pieces`.
- The new tokens in every theme; numbers in the `numbers` family.
- Accessibility: the strip as one progress value with a spoken text (`winui::expose_value` with a
  progress role), the graph's summary, tabs with their selected state, tooltips on keyboard focus
  as well as hover, 24 px icon buttons.

## 13d · States and the Done window's look (look, about 900 lines)

- Every state on boards 4, 8, 9 drawn from the existing state data (no new states in the engine).
  The error's reason is its own line, replacing "Error: {error}" (E5's lines here).
- The Done window's layout (boards 5, 7, 13, 14) on top of 13a's behaviour; menus through
  `chrome::popup`, since they run past the window.
- The Done window's name row (board 13): it takes the keys (Tab) and Ctrl+C copies the file to the
  clipboard, as a file is copied in Explorer. Today's drag strip only drags (pointer, no focus, no
  Ctrl+C), so 13a left this for the name row that replaces it.

## Checks (every part)

00-conventions' checks 1-8, the external audit loop until APPROVED, screenshots of every board's
state before and after, light and dark. After each part, `python tools/upstream-sync/sync.py status
--fetch`: any newly conflicting upstream commit is noted in the PR. Help pictures that show these
two windows are retaken once, after 13d (plan 12's retake skips them).

## Definition of done

- [ ] 13a-13d merged into `dev`, each audited until APPROVED
- [ ] Every row of "Done in plan 13" above is done, and plan 12's matching items say so
- [ ] Every board on the canvas matches the app at 100%, light and dark
- [ ] ROADMAP and CHANGELOG lines per part; docs and help pictures updated for the changed names
