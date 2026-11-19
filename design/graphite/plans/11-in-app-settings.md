# 11 · Options in the main window

Plan written on dev `da63b006`, 2026-11-18 (by a read-only planning agent; verify line numbers, they drift: find each by the name next to it). The mockup is artboard "4 · Options in the main window" of the 2026-11-18 UI proposals, drawn in the "graphite" palette (`source/pal.py`): the page takes its colours from the theme, none hard-coded.

**Goal.** Options and the other settings windows (Scheduler, Speed Limiter settings, Torrent settings, the browser extension) open as a page inside the main window, with sections in a side list and no settings popups. The download flow stays as dialogs: Add URL, Download File Info, progress, batch, video, gallery and torrent adds, confirmations.

**Maintainer's decision, 2026-11-18** (record it in `../PLAN.md` with D1–D7). A behaviour change, not a look phase (00, "Behaviour never changes in a look phase"): its exceptions are listed under each part.

- Branches: `ui/11a-options-page`, `ui/11b1-scheduler-section`, `ui/11b2-small-sections`, `ui/11c-appearance` (optional). Origin only, per 00.
- Needs: 08 (side list, `page_area`), 09, 10e. Before 11a: `sync.py status`, port anything waiting in `options.rs` / `scheduler.rs`.

## Status (2026-11-19)

11a is built on `ui/11a-options-page` (on top of the panels port below, which it needs: both
change `options.rs`): `settingspage.rs`, Options' shell in `options.rs`, the page in `App::paint`,
`gx-arrow-left` and `gx-puzzle`, the docs, and the shot tools' `options:<page>+@main` takes. 11b1,
11b2 and 11c are not started.

The status at the 1.0.1 handover (2026-11-18, dev `9a12944b`): the pre-step ports are done but
not merged; each is a pushed branch with one commit and no pull request yet, 6 commits behind dev:

| Branch | Upstream | State |
| --- | --- | --- |
| `upstream-sync/2026-11-18` | 81afcae2 (upstream issue 1041): Options → Downloads → Edit… opens the Download progress dialog; also `winui::enter_is_not_a_click` clears egui's `FAKE_PRIMARY_CLICKED`, so Enter on a focused check box no longer ticks it and presses OK in every dialog | built, tests green; its ledger also records c0be4d33 covered, e44dfc02 and 34cacf3a skipped |
| `upstream-sync/2026-11-18-1058` | 868f3087 (upstream issues 1058, 1091): a `.torrent` downloaded in the browser stays with the browser while Torrent settings don't open torrents with Grabbit; Grabbit is listed in Default apps | not yet compiled; review its installer (`mdm.iss`) edits |
| `upstream-sync/2026-11-18-818` | a70b2c17: browser integration from a copy run inside a zip no longer leaves browsers looking for a helper that is gone | not yet compiled |

The 1058 and 818 ports change `extension.rs`, `nmhost` and `win32.rs`, which the 1.0.1 feedback
PR (69) rewrote (no store listing, only the unpacked ID, `com.grabbit.host`): merge dev into each
and resolve onto that model, then build, test and open each through `sync.py pr`.

Since this plan was written, dev changed the main window it builds on (verify every line number
by name): PR 69 put the Find field in the title bar (`find.rs` `Bar`, `/show find:<text>`,
`/show find-options`), the light/dark button at the status strip's end (`Cmd::ToggleMode`,
`/show mode[:light|:dark]`; View → Dark Mode support and Always dark are gone, so 11c has no
dark-mode rows), made the first window 1080 × 560 and changed the browser extension window's
rows to "Show me how"; PR 75 switched Graphite to the green palette. The universal Add (PR 76,
draft) changes the toolbar's first item and the paste bar; take its icon-list additions in
`gen3.py` / `themeicons.rs` alongside this plan's `gx-arrow-left` and `gx-puzzle`.

Order: the three ports, then 11a, 11b1, 11b2 (11c only if wanted), each its own pull request
with an external audit until approved. The help pictures are retaken once for the 1.0.1 release,
not per part (PR screenshots per part as usual).

## Decisions (default if no answer)

| # | Question | Default | Why |
| --- | --- | --- | --- |
| D1 | "Options" or "Settings" | **Options** (toolbar, menu, tray, page title unchanged) | ~150 "Options → …" paths in docs/ and website/content stay true |
| D2 | Live apply or draft + save bar | **Draft + save bar** (Discard / Save) | Options' commit parses typed fields (`options.rs:427-440`) and runs side effects (`app.rs:5663 apply_options`: Run key, remote listener, PAC) that must not fire per keystroke; `Settings::merge_changes` (`settings.rs:601`) merges a draft safely |
| D3 | Back/Escape with unsaved changes | **Ask once**: `dialogs::Confirm` "Discard your changes?" | today Escape/close discard silently (`options.rs:3`) |
| D4 | RSS feeds | **Stays a window** | a feed reader that downloads (`rssdlg.rs`) = download flow |
| D5 | Small entry dialogs from a section (New login, DNS settings, Pair a device, Add account, Edit category, queue name, Delete queue) | **Stay small dialogs over the main window** | entry forms; inlining rewrites their bodies (`options.rs:2923-3506`) |
| D6 | Search box | **Not in 11** | |
| D7 | Section order | Today's 12 Options pages, then a hairline and the moved windows | keeps the `ORDER` test (`options.rs:4767`) |

## Facts

- **Options** (`options.rs`, 4,970 lines): `OptionsDialog` (`:180-230`) holds a draft `s` and `from`; `show` (`:313-444`) = one OS window `window(ctx, "options", "Grabbit Configuration", …)` (`:332-339`): side list, page (scrolled when short `:344-362`), footer OK* + Help (`:363-379`); overlays (sub-dialog, category editor, restore question `:394-426`); OK result from typed fields `:427-440`. `page_area(ui, o, downloads)` (`:448-466`) draws heading + dispatches: `general` 470, `file_types` 692, `save_to` 805, `downloads` 1047, `connection` 1208, `proxy` 1420, `accounts` 1735, `logins` 1904, `archives` 2134, `regional` 2264, `sounds` 2390, `remote_page` 2626. `W` 820 × `H` 520 (`:4156`), `SIDE_W` 190, `PAGE_AT`, `PAGE_LEFT…PAGE_BOTTOM` (`:4174-4192`), `PAGES` (`:4197-4284`), `side_list` `:4305`, `heading` `:4416`. Sub-dialogs (OS windows): `opt-pair` 2923, `opt-login` 3110, `opt-exception` 3155, `opt-panels` 3212, `opt-dns` 3357, `opt-account` 3376, `opt-addresses` 3410, `opt-keys` 3461, `opt-test` 3506. Tests from `:4639` (side list a11y `:4687`, `options_fits_a_short_screen` `:4743`, click/arrows/Ctrl+Tab `:4762`, every page inside its area `:4865`).
- **Scheduler** (`scheduler.rs`, 2,315): `window(ctx, "scheduler", …)` (`:346-355`), `W` 800; draft queues + limits (`:148-169`, `state`/`apply` `:282-302`); Close applies (`:380-384`); `body` `:389` → tree `:598`, `schedule_page` 661, `files_page` 1029, `limits_page` 1252; footer Stop/Help/Apply/Close/Start now (`:513-545`, `:580-584`); name prompt `question` `:1321`.
- **Speed Limiter settings**: `dialogs::SpeedSettings` (`dialogs.rs:2094`), `show` `:2127`, window `"speed-settings"` 480 wide; applied `app.rs:9874-9889`.
- **Torrent settings**: `torrentdlg::TorrentSettings` (`:949`), `show` `:1054`, window `:1057`, footer `:1319`, `Done(TorrentChoice)` `:1338`; applied `app.rs:9640-9694`.
- **Browser extension**: `extension::Extension` (`extension.rs`), `show` `:145`, window `:171`, `body(ui, o, h, answer)` `:189`; shown `app.rs:9327-9344`. (PR 69 changes this window's rows to "Show me how" — rebase on it.)
- **App wiring** (`app.rs`): fields `speed_settings`, `scheduler`, `options` (`:587-589`), `torrent_settings` (`:642`), `raise_options` (`:577-579`); commands `Cmd::LimiterSettings` `:6222`, `TorrentSettings` `:6257`, `Options` `:6324`, `Scheduler` `:6338`, `EditQueue`/`ScheduleQueue`/`NewQueue` `:6348`; Options outcome merged/applied `:9838-9856`; openers: Downloads menu, Tasks menu, toolbar, tray, `/show` `show_named` (`:2449`): `scheduler[:files|limits|name]`, `torrent-settings`, `extension`, `speedlimit`, `options:<page>|panels|test|account|pair`.
- **Main window** (`App::paint` `app.rs:7742`, `chrome::Layout` `chrome.rs:20-140`): title bar, toolbar (`:7955-8012`), paste bar (`:8014-8040`), panes `panes_top()`→`strip_top(h)` (tree `:8042-8167`, list `:8172-8265`, details `:8217`), status strip `:8254`; popups `:8266-8286`; shortcuts `:8289-8294` (`shortcut_cmd` `:910`); min window 600×300 (`main.rs`). NOTE: PR 69 changes app.rs/chrome.rs (Find field in the title bar, light/dark button at the status strip's end, first window 1080×560).
- `scale::pane(ui, id, content, body)` (`scale.rs:310`) scrolls a fixed-size layout.
- Shots: `tools/shots/shots.toml` takes `options:*` 14×, `scheduler` (3), `speedlimit`, `torrent-settings`, `options:panels`; `options-integration` has a pixel crop `[195,162,618,199]`; `tools/lab/docs-shots.ps1` waits for a new window after `/show` (`New-Window` `:93`, `:303`), list `:214-215`, two-window special case `options:panels` `:308-318`, "a+b" runs `/show a` then shoots b (`:255`). 36 help pictures to retake: `docs/images/help/options-*` (12 + integration, light/dark), `download-panels`, `scheduler`, `speed-limiter`, `torrent-settings`; website `features/*.md` reuse them; `features/scheduler.md` uses `/assets/screens/scheduler-light.webp`. `docs/ARCHITECTURE.md:63` "one immediate viewport per dialog".

## Inventory

| Window | Fate |
| --- | --- |
| Options | **Moves** (11a) |
| Scheduler | **Moves** (11b1) |
| Speed Limiter settings | **Moves** (11b2); Turn on/off stay menu commands |
| Torrent settings | **Moves** (11b2) |
| Browser extension | **Body reused** as a section (11b2); the offer at start stays a dialog |
| Appearance | optional new section (11c), menu stays |
| Category editor, Options sub-dialogs, RSS, Help windows, Helpers, all flow dialogs | Stay |

## Information architecture

- **Placement**: the page replaces the panes area (`panes_top()`→`strip_top(h)`), full width. Title bar, menus, toolbar, paste bar, status strip stay; downloads keep running; dialog commands open over the page.
- **Header**: a 40-pt band over the side list with a Ghost button "← Downloads" (Back; also Escape and Alt+Left). Sections keep the existing `heading` (icon tile + title).
- **Side list**: today's `side_list`, 190 wide: OPTIONS group (General, File types, Save to, Downloads, Connection, Remote, Proxy / Socks, Sites Logins, Accounts, Sounds, Regional, Archives, today's order), a hairline + caption **Tools**: Scheduler (`tb-scheduler`), Speed limiter (`gx-gauge`), Torrents (`cat-torrents` / the menu's), Browser extension (`gx-puzzle`, new icon). Optional (11c) **Look**: Appearance. Scrolls when short; Up/Down, Home/End, Ctrl+Tab.
- **Body**: each section keeps its fixed English layout at the origin right of the list, scrolled by `scale::pane` when smaller (Options 630 × page height, Scheduler 800, Speed 480, Torrents `SETTINGS_W`).
- **Save bar**: `winui::footer` at the bottom of the section column: Help (Ghost, leading; opens options.md / queues-and-scheduler.md / connections-and-speed.md by section), then **Discard** and **Save*** — greyed while nothing differs; "Unsaved changes" hint when dirty; Enter = Save unless a sub-dialog is open.
- **Quick uses** stay one click: toolbar Scheduler / Downloads → Scheduler open the Scheduler section; queue right-click Edit/Schedule open it on that queue; Create new queue opens it with the name prompt; Speed Limiter → Settings… opens Speed limiter; Turn on/off, Start/Stop Queue need no page. Scheduler's Start now / Stop move to its tabs' row.

## 11a · The page, with Options in it

1. New `crates/app/src/settingspage.rs` (~300 lines): `SettingsPage { section: usize, options: OptionsDialog }` (+ drafts in 11b), `fn show(&mut self, ui, area: Rect, downloads) -> Option<PageEvent>` (`Back`, `Save`, `Discard`); draws header, `options::side_list`, `self.options.page_area(...)` inside `scale::pane`, save bar; AccessKit container `Role::Pane` named "Options" (screenshot crop) and the Back button; `crate::crash::window("options")` while drawn.
2. `options.rs`: only the shell changes, no page body edits. Replace `show` with `overlays(&mut self, ctx, downloads)` (the `:394-426` lines), `result(&self) -> Settings` (`:429-439`), `modal()`, `dirty()` (`result() != from`); make `page_area`, `side_list`, `heading` `pub(crate)`; delete `W`/`H`/`MIN_H`/`fit_height` and the footer; give `side_list` the rows to draw.
3. `app.rs`: field `options` → `settings_page: Option<SettingsPage>`; remove `raise_options`; `Cmd::Options` opens the page and closes popup/menu; in `paint`, right after the paste bar, an early branch (page + status strip + `popup_menu` + pasted + chosen, then return) — the 220-line panes block is NOT re-indented (upstream picks stay clean); only the popup block moves into `fn popup_menu`; `shortcut_cmd` and `panes::tab_keys` are not run on the page (Delete/Ctrl+A can't act on a hidden list). `dialogs()`: Save → `merge_changes` + `apply_options`, page stays open with a fresh draft; Discard → the existing Closed lines (re-publish categories, remote config) + fresh draft; Back → if dirty, Confirm (D3), else close. `show_named`: `options:<page>` (and panels/test/account/pair) open the page; every other name first closes the page (discarding); new `downloads` name does only that.
4. Exceptions to 00's no-behaviour-change rule, named: Options no longer a window; footer gains Discard, OK → Save; Back asks; toolbar's Options doesn't raise a window.
5. Docs: `docs/options.md` intro, `docs/ARCHITECTURE.md:63`, CHANGELOG, ROADMAP "Options in the main window".

## 11b1 · Scheduler as a section

`dialogs::inline(ui, o, size, bg, body)` (~15 lines, same closure shape as `window`); `scheduler.rs` `show(ctx,…)` → `show(ui, o, …)` swapping `window` for `inline`; footer removed, Start now / Stop to the tabs' row; `dirty()`, `save(actions)` = `apply`; Close = Apply rule goes (Back asks, D3); `name_prompt`/`deleting` stay dialogs; tree/pages untouched. `settingspage.rs`: `scheduler: Option<Scheduler>` made on first visit. `app.rs`: Scheduler commands and `/show scheduler:*` open section 12; `scheduler_action` unchanged. Download limits exist in Options → Connection and Scheduler → Download limits: on Save, Scheduler first, then Options' `merge_changes` (only what Options changed); test it.

## 11b2 · Speed limiter, Torrents, Browser extension

Speed limiter (`dialogs.rs:2127`) → `show(ui, o)` via `inline`, footer removed, `pub fn choice() -> SpeedChoice`, `app.rs` `fn apply_speed(c)` called by Save; `Cmd::LimiterSettings` and `/show speedlimit` open section 13. Torrents (`torrentdlg.rs:1054`) same: `choice()`, `fn apply_torrent_settings(c)`, section 14. Browser extension section 15: `section(ui, o)` calls the existing `body`; nothing to save (save bar greyed); the offer at start and `/show extension` stay the window; `/show options:extension` opens the section. Dirty/Save across sections: dirty if any draft is; Save saves Scheduler, Speed, Torrents, Options in that order.

## 11c · Appearance (optional)

A section built from the View menu's data (themes, Add a theme file…, Open themes folder, interface size, draw with); each control emits the menu's own `Cmd`, live; save bar greyed. NOTE: PR 69 removed View → Dark Mode support / Always dark (the mode is the status-strip button now).

## Upstream sync

Upstream changed (since 2026-08-18): options.rs 51, dialogs.rs 65, torrentdlg.rs 25, scheduler.rs 16, chrome.rs 31, app.rs 199 commits. Rules: no edits inside a body (page functions, scheduler's tree/pages, extension::body, the Speed/Torrent closures stay line for line, unindented); only shells change; app.rs: add, don't move (no re-indentation); a pick touching a removed shell is a port; `sync.py status --fetch` must sort the waiting commits the same before and after each part (record counts in the PR).

## Screenshots and tools

shots.toml: `take = "options:<p>"` → `"options:<p>+@main"` with `crop = { around = ["Options"], pad = 0 }`, same for scheduler/speedlimit/torrent-settings; `options-integration`'s pixel crop/mark → `around`/named controls; `options:panels` and `scheduler:name` stay plain takes; alt texts updated. docs-shots.ps1: built-in list gets `+@main` forms; delete the `options:panels` two-window case; run `/show downloads` before the main-window shot. Retake (light+dark) options-* (13), download-panels, scheduler, speed-limiter, torrent-settings, the site's scheduler screen; `make.py check` passes. PR shots per 00: each section, save bar dirty, the Discard question, a sub-dialog over the page, the page in a 600×300 window, light/dark, 100% and 200%.

## Docs and website text

`docs/options.md` intro; `docs/queues-and-scheduler.md` ("The Scheduler window" → "The Scheduler", Apply/Close → Save/Back); `docs/connections-and-speed.md:129`, `docs/guide-keep-speed-for-everything-else.md:18` alt texts; `docs/torrents.md`; website features (speed-limiter, scheduler, torrents, browser-extension) wording where they say "window". i18n: "Downloads" (Back), "Save", "Discard", "Unsaved changes", "Tools", "Look", "Speed limiter", "Torrents", "Browser extension", "Discard your changes to Options?".

## Tests

Keep, re-pointed: side list is a list of its pages (16 after 11b); every page reached by click/arrows/Ctrl+Tab (`ORDER` gains 12–15); every page inside its area (unchanged); sub-dialog Enter/Escape. Replace `options_fits_a_short_screen` with `the_page_scrolls_in_a_small_window` (600×300). New: `save_merges_only_what_options_changed`, `discard_restores_categories_and_remote`, `back_with_changes_asks_back_without_changes_closes`, `the_list_takes_no_keys_while_the_page_is_open` (Delete/Ctrl+A act on nothing), `show_names_open_their_section`; 11b1 `scheduler_save_then_options_save_order`, `start_now_applies_then_starts`; 11b2 `speed_choice_round_trips`, `torrent_choice_round_trips`; `chrome.rs` headless a11y harness: a page frame has the "Options" pane, the "Pages" list, the Back button, and no list/tree nodes. Keyboard pass and screen-reader pass.

## Risks

Hidden list keys (early return skips shortcut_cmd/tab_keys; tested); lost edits (D3); settings written elsewhere while open (merge_changes); fixed-width sections scroll, not reflow, in a narrow window; sub-dialogs now over a large main window (`dialogs::place` centres them); remote pairing changes the live listener before Save → Discard/Back must re-apply `remote::config`; every non-page `/show` closes the page.

## Done when

- [ ] 11a: Options opens in the main window from toolbar, menu, tray, `/show options:*` and the extension's Button settings; Save/Discard/Back per D2/D3; no "Grabbit Configuration" window remains.
- [ ] 11b1: Scheduler is a section; queue right-click, Start now and Stop work; no Scheduler window remains.
- [ ] 11b2: Speed limiter, Torrents and Browser extension are sections; the extension offer at start still opens by itself.
- [ ] Flow dialogs and D5's small dialogs unchanged.
- [ ] Help pictures retaken, `make.py check` passes; docs, website wording, ARCHITECTURE, CHANGELOG, ROADMAP updated.
- [ ] `sync.py status` sorts upstream's waiting commits the same before and after each part.
- [ ] 11c done or recorded as not wanted.
