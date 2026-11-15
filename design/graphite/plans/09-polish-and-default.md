# 09 · Polish, then make Graphite the default

> **Adjusted after 05.5** (Graphite only, merged 2026-11-13): there is no Classic layout, no old theme and no Silk, Tango or Lucide icon left, so this plan's steps are Graphite's alone (`00-conventions.md`).

Adds the finishing touches the theme shipped without (the two-tone hot colour and the dark-mode glow),
does a full review against the mockup, and finishes Graphite as the default (05.5c already made it the
default and only look).

- Branches: `ui/09a1-finish`, `ui/09a2-mark`, `ui/09b-text`, then `ui/09r1-…`, `ui/09r2-…`,
  `ui/09r3-…` in parallel, then `ui/09b-shots` (see **Fast track** below)
- **Time box: 3½ hours of agent time for the whole plan**, from the first commit to the last PR ready.
- Needs: 06, 07e, 08b (and through them 05, 07a…07d, 08a; 02b's roles `button.accent.top`,
  `progress.bar.end`, `glow`). 09a1 and 09a2 need only 02b's roles (see rule 2 below).
- Unblocks: 10 (the rename's screenshots are taken once, on the final look)

## Fast track (3½-hour cap)

Run the same way as 07's and 08's fast tracks (`07-dialogs.md`, `08-options.md`). It differs from 08's
in having two merge windows: the review needs the finish in, and the screenshots need the review's
fixes in.

| Slot | Work | Budget |
| --- | --- | --- |
| 0:00–0:45 | **Three parallel parts**, one agent and one worktree each: 09a1 (PR 1), 09a2, 09b-text | 45 min |
| 0:45–1:00 | Maintainer merges 09a1 and 09a2 into `dev` | 15 min |
| 1:00–2:30 | **The full review (09a step 5) in three parallel parts**, R1, R2, R3, all branched from `dev` with 09a1 and 09a2 in it; each reviews and fixes its own windows | 90 min |
| 2:30–2:55 | Maintainer merges R1, R2, R3 one after another; the agents rebase and fix ROADMAP/CHANGELOG conflicts | 25 min |
| 2:55–3:30 | **09b-shots**: website and docs screenshots retaken on the final look, then merged | 35 min |

The parts (each its own PR):

| Part | Work |
| --- | --- |
| 09a1 | 09a steps 1, 2 and 4: two-tone buttons and progress, the dark glow, the frame-time check (over budget: the glow comes out in this same PR) |
| 09a2 | 09a step 3: the app mark (About's at 56 drawn sharp; the title bar's already is), the two offers without `winui::banner`'s logo band |
| 09b-text | 09b steps 2 and 4 to 5 without the screenshots: `website/content/themes.toml`, `features/themes.md`, `theme-prompt.txt`, `llms.txt`, `tools/shots/shots.toml` entry, the `docs/` theme help, CHANGELOG, ROADMAP |
| R1 | The main window, the details pane, the paste bar, and **Left from 06** |
| R2 | 07's dialogs, and the contrast pass (the WCAG tests in `palette.rs`, greyed text by hand) |
| R3 | Options and its nine sub-dialogs, About, the extension and video-helpers offers |
| 09b-shots | `tools/shots/` and `docs/` screenshots in Graphite, the `site-theme-graphite` shot; R1's check of the paste bar and details pane at 150 and 200 (Left from 06) |

Each review part covers its windows in light and dark at interface sizes 100, 150 and 200, side by side
with the mock, plus the keyboard pass (Tab order, Alt and menu letters, Enter and Escape) and the
accessibility-tree pass (names, roles, disabled states, bounds).

Rules that keep it inside the box:

1. **Own functions only.** 09a1 edits only the painting code (`paint_button`, `button_colors_in`,
   `progress_bar`, `era_bar`, `fill_shaded`, the `[controls]` looks); 09a2 only the About window and
   the two offers (the title bar's mark needs nothing since 10b's logo). Each review part fixes only its own windows' functions. A gap in
   the shared kit goes to `winui` in a small PR of its own, not into a part.
2. **Start early.** 09a1 and 09a2 touch no dialog or Options code, so they can run beside 08b (or
   earlier); then only the review and the shots are left after 08b, about 2¾ hours. The review cannot
   start early: it must see 06, 07 and 08 finished.
3. **Worktrees, builds, screenshots, audit, merges:** as 07's fast track rules 2 to 7: own
   `CARGO_TARGET_DIR` per worktree; `cargo check -p mdm-app` while iterating, the full workspace once per
   PR; screenshots in one batch per part with `baseline.ps1 -Theme graphite`, light and dark,
   `-Work` inside the worktree, an exe copy per runner; one audit round against the fixed checklist.
4. **Running over.** The review is the open-ended part: what a review part has not fixed at 2:30 ships
   as a follow-up listed under the ROADMAP item (09a step 5 already allows "fixed or recorded as
   follow-ups"). The frame-time budget is not waived: over budget means flat, not late.

## Facts this plan relies on (paths under `crates/app/src/`, checked on `ui/03-icons` c3541d0)

Line numbers drift as phases land: find each item by its name first.

- Gradients already exist: `winui::fill_shaded(p, rc, radius, stops)` (`winui.rs:1583`) and
  `fill_shaded_corners` (`:1589-1661`) build an `egui::Mesh` of vertical bands with rounded corners;
  `enum Shine { Luna, Aero, Aqua }` (`:1667`) with `stops(face, gloss)`.
- Buttons are painted in `paint_button` (`winui.rs:3639`, `fill_shaded` at `:3682`), colours from
  `button_colors_in` (`:3741`; the accent roles `button.accent` and `button.accent.bottom` at
  `palette.rs:210/213` are read at `winui.rs:3769`); progress bars in `progress_bar` (`:638`) and
  `era_bar` (`:1826`); the title bar in `winui::caption` (`style()` `:5485`, `title_bar` `:5954`).
- Looks are chosen by `ButtonLook` / `ProgressLook` / `CaptionLook` (`palette.rs:469/491/502`) through
  `[controls]` (`Controls::set`, `:546-582`). `ButtonLook::Graphite` and `ProgressLook::Graphite` do not
  exist yet. `button.glow` (`palette.rs:216`) is Luna's glow under the pointer, not a shadow: leave it.
- The theme's id is saved in `settings.theme`, default "graphite" since 05.5c; an id MDM no longer
  has (the old themes) is Graphite, and saved so at the next save.

## 09a · Finish

1. **Two-tone hot colour**: a new `ButtonLook::Graphite` and `ProgressLook::Graphite` (`[controls]`
   values `button = "graphite"`, `progress = "graphite"`, written by `make_theme.py`). For the Accent and
   Default buttons, `paint_button` fills with two stops (`button.accent.top` at the top,
   `button.accent` at the bottom) through `fill_shaded`; the progress fill runs left to right
   `progress.bar` → `progress.bar.end` (a horizontal variant of `fill_shaded`, or two meshes).
   As built (09a1): only filled buttons (`ButtonKind::Accent`, which a footer's Default becomes)
   are shaded; a Default button outside a footer is outlined in Graphite, so it has no face to
   shade. The glow follows the mock rather than one shadow for both: under a filled button
   `0 6px 18px -8px` (narrower than the face, 6 lower, blur 12); round a progress fill
   `0 0 10px` (grown by 4, blur 10, since egui caps a blur at the rect's smaller side).
2. **Glow (dark only)**: a soft shadow under the default button and the progress fill in the `glow`
   role (about 35% alpha), drawn with `epaint::Shadow` (blur 12, offset y 4). Graphite light sets `glow`
   transparent, so nothing is drawn.
3. **App mark**: the Grabbit logo exists (10b steps 1 and 2, PR 26: `art/grabbit.svg`, Graphite's
   cyan into amethyst, no tile behind it), so there is no placeholder gradient tile to build: the
   marks are the app icon's own PNGs, which `tools/icon/build_icon.py` writes.
   - Title bar: done. `winui::caption`'s two title bars (the drawn one and `title_bar`) already paint
     `mdm-32.png` at 16, sharp up to 200 %.
   - About (`help.rs`, the "about" window): it paints `mdm-48.png` stretched to 56, soft at 100 % and
     blurry at 150 and 200 %. Paint `mdm-128.png` at 56 instead (scaled down, sharp to 200 %).
   - The extension offer and the video-helpers offer drop `winui::banner`'s logo band under Graphite,
     as the mock draws them.
   - Nothing here needs the theme's colours any more: no `accent` → `button.accent` tile, no radius.
4. **Cost check**: frame time of the main window with 1,000 rows and 10 running downloads, before and
   after, in Graphite dark. Budget: no more than 5% slower; otherwise ship flat (remove the glow first).
   **Measured (09a1)**, as 04 did: a release build with a temporary probe (not kept), the main
   window off the screens with 1,000 downloads (990 queued, 10 running from the lab server at
   about 200 KB/s each), continuous repaints, Graphite dark switched live through `/show theme:`
   between Graphite and a theme file inheriting it with `button = "fluent"`, `progress = "line"`
   (today's flat look), eight rounds of 25 s each; the mean of every 600 frames' `unstable_dt` and
   of the main window's `ui` CPU time:

   | Graphite | `unstable_dt` mean | Frame CPU mean |
   | --- | --- | --- |
   | flat (before) | 17.482 ms | 15.638 ms |
   | two-tone and glow | 17.234 ms (−1.4 %) | 15.414 ms (−1.4 %) |

   Within the run's noise (each mean drifts by about ±10 % over the run): the budget is met and
   the glow stays.
5. **Full review**: every mock state, light and dark, at 100% and 150%, side by side with the mock; a
   list of differences fixed or recorded as follow-ups. It includes:
   - a contrast pass with the real colours (the WCAG tests in `palette.rs` plus a manual check of greyed text);
     **done in 09r2**: `built_in_themes_text_and_meaning_roles_read` (and `make_theme.py`'s
     check) covers the rest of the text and meaning roles, the filled button's label on its
     lighter top included (white on `#00829a` 4.51:1 light, 14.12:1 dark). Greyed text is exempt
     from WCAG 1.4.3 and held at 2:1 on every face it is drawn on, below the secondary text's
     ratio: measured 2.14 (light, a menu item on the pointer's face) to 2.97 (dark, on the
     window), 2.24 to 2.53 light and 2.44 to 2.97 dark elsewhere. No colour failed, none changed;
   - a full keyboard pass (Tab order, Alt and menu letters, Enter and Escape) over every changed window;
   - an accessibility-tree pass over every changed window (names, roles, disabled states, bounds);
   - interface size 200%.

   Of step 5's passes, 09r2 ran only the contrast pass (besides the "Left from 07" items
   below, which it did). Over 07's dialogs it leaves, as follow-ups under the
   ROADMAP's Graphite item: the side-by-side with the mock in the running app (light and dark,
   100/150/200 %, the Scheduler included), the keyboard pass and the accessibility-tree pass.
   It also keeps `[profile.dev.package.mdm-crash] debug = "full"` in `Cargo.toml`: the checks
   build with `--config profile.dev.debug=0`, and mdm-crash's end-to-end tests read crashy's own
   debug info to name the function, so they fail without it (build setting only, no app change).

## 09b · Graphite by default

1. **Done in 05.5c**: Graphite is the default and the only built-in look; no one-time question is
   needed (nothing else to choose) and no migration (the fork had no users).
2. Left: a user upgrading from the last release (whose looks are gone) starts in Graphite; say so in
   the release's CHANGELOG and notes (05.5c's line already does).
3. **Downgrade**: the last release (0.5.2) has no Graphite: it reads `graphite` as an unknown theme
   and falls back to its classic look. Acceptable; nothing to do.
4. Website and docs screenshots (`tools/shots/`, `docs/`) retaken in Graphite; `docs/` theme help lists Graphite.
   Left from 02, which kept Graphite off the website until a release has it: with the release that
   ships it, add its tile to `website/content/themes.toml` (and the "Nine looks" line of
   `website/content/features/themes.md`), its `site-theme-graphite` shot to `tools/shots/shots.toml`
   (`take = "theme:graphite+@main"`, as the other themes'), `"graphite"` to the `inherits` list of
   `website/content/theme-prompt.txt` (its `[controls]` no longer takes `chrome`: 05.5c ignores it),
   and Graphite to the themes in `website/llms.txt`. The old themes' tiles and shots go (plan 10's
   "Left from 05.5" and KNOWN-GAPS G3 hold the list).
5. CHANGELOG entry for the new look; ROADMAP's "A new look, 'Graphite'" item marked ✅.

## Left from 06

09a's full review (step 5) takes these. Plan 06's "Left from 06" has the details:

- the on-screen keyboard and Narrator pass over the paste bar, the details pane and their toggles
  (skipped in 06; the headless tests cover the logic). Left to a person at the screen (09r1 cannot
  run Narrator or the on-screen keyboard);
- interface sizes 150 % and 200 % for both (06's screenshots are at 100 % only). Left to 09b-shots
  (`baseline.ps1` at 150 and 200 with `/show paste:…` and `/show details`);
- the audit's deferred nits:
  - tests of `route_new_url`'s side effects (done in 09r1: `app::route_new_url_tests` builds an `App`
    with `App::assemble`: the history and `dirty` for every kind of address, `request_parts`'
    login on a .torrent's fetch, the Video and General folders, the duplicate check);
  - a file name without break points staying on one line in the pane's header (done in 09r1:
    the pane's `pane_lines` takes `winui::fit_lines`' break, else, when its first line is too
    wide, the latest space, slash or hyphen break whose first line fits (listed by the pane's
    own `name_breaks`), and breaks inside the name only where none fits, as the mock's
    `overflow-wrap:anywhere`; the shared `winui` kit is unchanged);
  - tests of the splitter and of a short pane (done in 09r1: `panes::dragged_width`, pulled out
    of `App::details`, and a 276 × 150 pane that paints nothing outside itself);
  - `/show details` while a column filter is set (done in 09r1: it picked a download the
    filter hid, which the list then unselected, so the pane was empty; it now keeps the filter
    and picks among the downloads it lets through in All Downloads (without a filter, the first
    stopped download across the list, as before), and when the filters hide every download,
    as `/show filtered` does, it clears them as `/show unfiltered` does and picks among all).
- 09r1's review against the mock (`m-main`, the paste bar, the details pane): the paste band
  (padding 8 × 10, gap 8, the field 32 high, the button 30 high centred on it), the pane
  (padding 14, name SemiBold 13 on 2 lines, the label row 12 muted then 6, the
  cells 8 × 8 high 2 apart, the rows' 96-point label column 12 before the value, 7 between
  rows, the buttons 2 + 1 at 30 high 6 apart) and their roles match (the buttons were 32 high,
  set to the mock's `.btn` 30 in 09r1's audit). Recorded, not changed: the pane's cells take the download's
  status colour (06's choice, as the list's Status cell) where the mock always uses the
  progress colour; the header's icon is the shell's file icon at 32 with no tile, then 10,
  where the mock's `.ficon` is a 40 tile (`--tile`, radius r-m) with a 21 icon inside, so the
  name starts 8 points further left than the mock's.
- Follow-ups of 09r1 (not reviewed in it; the next review part or 09b-shots takes them): the
  rest of `m-main` against the mock, light and dark at 100, 150 and 200: the toolbar, Categories,
  the list and its columns, the menu bar and its menus, the context menu, the column chooser, the
  filter box, the empty and filtered overlays, the status bar; and the keyboard pass and the
  accessibility-tree pass over the whole main window.

## Left from 07

09a's full review (step 5) takes these, from 07c's, 07d's and 07e's audits:

- (done in 09r2) the progress window's **Hide tab** at the page's top right, as the mock has it,
  30 high: x to 521, y 34 to 64 (the Speed Limiter's check boxes start at 64; Options on
  completion's Save To path now stops 8 before it);
- (done in 09r2) the torrent window's **General** tab: the mock's info banner ("Complete,
  sharing", the rate to its peers) in the piece bar's place while a finished torrent is shared,
  the property table 19 lower with 5 rows (the rest scroll), ending at 403 inside the page. One
  difference from the mock, kept on purpose: the rate sits on the title's row, at the banner's
  right, not on a second line under it (a two-line banner, 56 high, would cost the table more
  rows);
- (done in 09r2) `/show` entries for the progress window's error state (`progress:error`, a
  sample reason, the download untouched) and its tab row's menu (`progress:menu`, shot as `@menu:progress:menu`: the menu is a window
  of its own, composed over the progress window, both closed after), and for
  `create-folder` (Yes makes nothing), `archive-password` (its answer extracts nothing),
  `add-category` and `drop-target` (shown this run, the setting untouched), all in
  `baseline.ps1`'s states; the queue-done countdown gets none that runs (it turns the computer
  off after 30 s);
- (done in 09r2, the follow-up after its build) the **Scheduler** at the mockup's 800 and short
  enough for a 1080p screen at 200 %: client 800 x 496 with Graphite's drawn 32-high title bar
  (464 under it; was 651 x 559, which scrolled there; the pages end 4 above IDM's, `PAGE_UP`,
  for Windows 11's 48 px taskbar). The queue list keeps its place; the
  pages' rows end at 776 (`RIGHT`, was 627), the weekdays take four columns of two rows (16
  back), Hang up and Exit share a row (18 back), the file to open sits after its caption on its
  row (25 back), the files list's name column takes the width. A unit test checks the client,
  title bar included, against `scale::room_on` (1920 x 1080, Windows 11's 48 px taskbar, the
  frame, at 200 %: 952 x 496), the rule `dialogs::shown_size` shows windows by. Not yet compared side by
  side with the mock in the running app.

## Left from 08

09a's full review (step 5) takes these, from 08a's and 08b's audits (PRs 25, 32, 33, 34):

- **Options at 200 %**: its 820 x 520 client scrolls on a 1080p screen (as before 08). (Done in
  09r3: shorter where the screen is, `fit_height`, down to the side list's whole height; its
  page scrolls between the list and the footer, the window wider by the bar; 100 % unchanged.)
- **`status_banner`**: no wrapping title (Accounts draws its sentence in the title's place itself)
  and nothing a screen reader can read; Remote's and Accounts' banners need both. (Done in 09r3:
  the title wraps as far as the banner's height has room, `status_banner_lines` and
  `status_banner_height_for` size it; no title puts the sub in its place, which Accounts uses;
  every banner is a `Status` node named by its title and sub.)
- the side list's rows (`ListItem`) have no `List` parent node for screen readers. (Done in 09r3.)
- a greyed **`icon_only_button`** keeps its icon dark (Sites Logins' "..." with cookies Off): the
  greyed tint in `button_face`. (Checked in 09r3: not a fault. The 08b2 shots' dots are
  `text.disabled` exactly, #a5a0b6 light and #5e596c dark, as Edit's greyed label; three solid
  dots only read darker than a thin label. Tests hold a greyed button's ink at `text.disabled`
  and `themeicons::graphite` inking a drawing with the ink given.)
- **Pair a device**: the mock centres its column; the kit has no centred paragraph. (Done in
  09r3: `winui::paragraph_centred`; the sentence, the code's line, the hint, the expired line and
  New code centred.)
- **Add an account**: its waiting hint's box is a fixed 94 px (two lines; a third would overflow).
  (Done in 09r3: sized to its hint; a longer one moves the rest down and the window grows.)
- 09r3's review of Options, its sub-dialogs, About and the two offers against m-options, m-dns,
  m-acct, m-leftb, m-keys, m-testbi, m-panels, m-pair, m-newlogin, m-newexc, m-about, m-ext and
  m-helpers (m-socks and m-virus are proposals, not built): Pair a device's and Add an account's
  codes, painted text, are now read by screen readers. Left, needing a person at the screen:
  the light/dark side-by-side at 100, 150 and 200 % (screenshots), Narrator over the side list,
  the banners and the codes, and Options at 200 % on a real 1080p screen (the page's scroll bar
  and Tab bringing a control into view).

## Done when

- [x] Graphite's default button and progress show the two-tone finish; dark mode shows the glow; light does not (PR 44)
- [x] Frame-time budget met (−1.4 %, 09a step 4)
- [x] Every mock state reviewed in code (visual, keyboard, accessibility, 200%); follow-ups filed (PRs 46 to 48); the passes that need a person at the screen (Narrator, the on-screen keyboard, side by side at 150 and 200 %) are listed under the ROADMAP item
- [x] An upgrade from the last release starts in Graphite, and the release notes say so (05.5c; CHANGELOG)
- [x] Screenshots, docs, website, ROADMAP, CHANGELOG (09b-text PR 29, 09b-shots: the help
  pictures, the README's and the site's (seven crops and four marks set again for the new layout), taken on this PC with `make.py`, `docs-shots.ps1` and
  `progress-shots.ps1`, addresses on `localhost:8099` as no hosts entry could be added; left:
  the video's progress window (`progress-video`, and the help's `video-progress` and
  `vfp-progress` made from it), which needs yt-dlp, and File Properties' type line naming this PC's video
  player, for a run on a stock Windows)
- [x] Done within the 3½-hour box, or what's left listed under its part's ROADMAP item (over the box; what is left is listed)

## Left after 09 (the screen pass, 2026-11-15)

An automated keyboard and accessibility-tree pass (each changed window on the screen, Tab through
it, Shift+Tab, Escape; its UI Automation tree's names, roles, enabled states and bounds) at 100,
150 and 200 %, and Options at 200 % on the 1920 x 1080 monitor: every Tab order cycles, Escape
closes each dialog, the progress window's Escape leaves its download running, and Options at
200 % is 837 x 496 with its page scrolling and every stop reached. What it found unnamed or named
twice is fixed (branch `ui/09-finish`). Its 150 and 200 % pictures, reviewed, left these to fix:

- the main window with the details pane open: toolbar labels cut ("Add Torr…", "Delete Co…",
  "Start Que…", "Stop Que…"), and at 200 % the pane's last row under its buttons;
- the progress window at 150 %: the connection bar shows its first tick only;
- the Scheduler: the Schedule tab's label without side padding, Stop download at's check box 2 px
  right of the others, the retries box a little high at 200 %;
- About's and Helpers' icons at 150 and 200 %, light: a faint square behind them;
- Create folder: the path cut at its end (the folder made is the part hidden);
- Add an account: "Find my token" out of line with the left column;
- Narrator itself, by a person at the screen.
