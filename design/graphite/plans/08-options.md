# 08 · Options as a side list

> **Adjusted after 05.5** (Graphite only, merged 2026-11-13): there is no Classic layout, no old theme and no Silk, Tango or Lucide icon left, so this plan's steps are Graphite's alone (`00-conventions.md`).

Replaces the two rows of tabs (which swap front and back when a back-row tab is clicked) with the same
twelve pages listed down the side, in the same order. Every page keeps its controls and wording.

- Branches: `ui/08a-options-list` (side list, pages unchanged inside), then `ui/08b1-…`, `ui/08b2-…`,
  `ui/08b3-…` in parallel (pages on `form`; see **Fast track** below)
- **Time box: 3 hours of agent time for the whole plan**, from the first commit to the last PR ready.
- Needs: 07a (the kit: footer, form, icon_tile, status_banner), 02b, 03b (page icons); 04 before final sizes (04's plan lists the Options labels Geist overflows)
- Unblocks: 09

## Facts this plan relies on (paths under `crates/app/src/options.rs`, checked on `ui/03-icons` c3541d0)

Line numbers drift as phases land: find each item by its name first.

- Window 430 x 506 (`:309`); footer OK* (`:336`) and Help (`:344`), no Cancel; Escape closes, Enter is
  OK (`:357-359`).
- `fn tabs` (`:4047` to about `:4220`): `const G` first row (General 0, File types 1, Save to 2,
  Downloads 3, Connection 4, Remote 11), `const P` second row (Proxy 5, Sites Logins 6, Accounts 7,
  Sounds 8, Regional 9, Archives 10); `first_row = page < 5 || page == REMOTE_PAGE` (`:4090`) picks the
  front row; rows at y 9 and 27; the page frame spans x 6..421, y 45..468.
- Keyboard order `ORDER = [0,1,2,3,4,11,5,6,7,8,9,10]` appears twice (Ctrl+Tab `:314`, arrows about `:4131`).
- Page dispatch about `:320-331`; pages: `general` 420, `file_types` 646, `save_to` 774, `downloads`
  978, `connection` 1119, `proxy` 1362, `accounts` 1690, `logins` 1866, `archives` 2079, `regional`
  2210, `sounds` 2338, `remote_page` 2565.
- Every page uses absolute English x values inside the 6..421 frame (for example
  `buttons_to_right(&p, 392.0, …)` at `:582, 866, 993`, 389 at `:673`, `button_row_from_left` at
  `:735`, 406 at `:3871`).
- Sub-dialogs are separate windows (`pair_window` `:2823` to about `:3410`, `sub_dialog` `:2982`).

## Open decision owned by this phase

A Cancel button: Options has none today; the close box and Escape already discard the changes
(`options.rs:3`). Decide before 08a starts and record it in `../PLAN.md`; the recommendation is no
Cancel (no behaviour change: Escape is the cancel).

## Fast track (3-hour cap)

Run the same way as 07's fast track (`07-dialogs.md`); what differs here is that everything lives in
one file, `options.rs`.

| Slot | Work | Budget |
| --- | --- | --- |
| 0:00 | The Cancel decision above, recorded in `../PLAN.md` (default if no answer: no Cancel) | — |
| 0:00–0:50 | **PR 1 = 08a**: the side list, the page origin, the footer | 50 min |
| 0:50–1:05 | Maintainer merges PR 1 into `dev` | 15 min |
| 1:05–2:35 | **08b in three parallel parts**, one agent and one worktree each, all branched from `dev` with PR 1 in it | 90 min |
| 2:35–3:00 | Maintainer merges them one after another; the agents rebase and fix ROADMAP/CHANGELOG conflicts | 25 min |

08b's three parts (each its own PR):

| Part | Pages and sub-dialogs |
| --- | --- |
| 08b1 | General, File types, Save to, Downloads, Connection (its spin box) |
| 08b2 | Remote (`status_banner` Info), Proxy, Sites Logins, Accounts (`status_banner` Warning), Sounds, Regional, Archives |
| 08b3 | The nine sub-dialogs (`pair_window`, `sub_dialog` and their callers), the window's 820 x 520 size, the Options help page in `docs/` |

Rules that keep it inside the box:

1. **Own functions only.** Each part edits only its own page and sub-dialog functions in `options.rs`, so
   git merges the three cleanly. The shared parts (`fn tabs`/`side_list`, `ORDER`, the page dispatch,
   the window size) belong to 08a and 08b3. A kit gap goes to `winui` in a small PR of its own, not
   into a part.
2. **Start early.** 08a needs only 07a (in 07's PR 1), so it can run beside 07c, 07d and 07e; plans 07
   and 08 then fit in about 4½ hours together instead of 7.
3. **Worktrees, builds, screenshots, audit, merges, running over:** as 07's fast track rules 2 to 7:
   own `CARGO_TARGET_DIR` per worktree; `cargo check -p mdm-app` while iterating, the full workspace once
   per PR; screenshots in one batch per part with `baseline.ps1 -Theme graphite -Shots <its pages>`
   (`options:<page>` shows a page), light and dark, `-InterfaceSize 100` and `200`, `-Work` inside the
   worktree, an exe copy per runner; one audit round against the fixed checklist, here also: every
   page reachable by click, Up/Down and Ctrl+Tab in `ORDER`, each page's first control hit where
   it's drawn; what isn't ready at 2:35 ships as remaining under the ROADMAP item.

## 08a · The side list

1. **Side list**: `fn side_list(ui, o, page) -> Option<usize>` draws the twelve entries (16 px icon +
   label, 190 px wide, selected row filled with `selection`, its icon in `accent`) in `ORDER`; today's keys
   keep working exactly (Left/Right/Home/End and Ctrl+Tab move between pages, `options.rs:4130-4136`;
   Enter stays OK; a click selects), and Up/Down are added as 00's keys exception; exposed as `Role::List` / `ListItem` (the tabs'
   accessibility roles move with it). Keep one `ORDER` constant used by both places.
   Icons: General `gx-adjust`, File types `ft-generic`, Save to `cat-folder`, Downloads `gx-download`,
   Connection `gx-network`, Remote `gx-device`, Proxy / Socks `gx-shield`, Sites Logins `gx-key`,
   Accounts `gx-user`, Sounds `gx-volume`, Regional `gx-world`, Archives `ft-archive`.
2. **Page origin**: pages draw at an origin; pass `o + (206, 56)` and a page width of
   415 (the tab frame's inner width), so every page's absolute x values land inside its area without
   rewriting the pages. Above the page: its heading (an `icon_tile` with the page icon + the page's
   title, as the mock).
3. **Window size**: 206 + 415 + 16 = about 640 x 560; footer via
   `footer` with Help (Ghost), OK* (Cancel per the decision above). Keys as today.
4. **Pages**: check each page at the new origin; fix any control that assumed the old frame edges (the
   `buttons_to_right(&p, 392.0…)` calls take the page's right edge from the page width instead).

## 08b · Pages on the form

1. Move each page's labels and fields onto `form` (112 / 240 label column), grouping options with
   hairline rules instead of group boxes, so they match the mock; widen the window to the mock's 820 x 520.
2. Page-specific parts from the mock: Connection's spin box, Remote's status as an `status_banner`
   (Info), Accounts' warning as a `status_banner` (Warning).
3. **Sub-dialogs** (DNS settings, Add an account, Addresses left to the browser, Keys, Test browser
   integration, Download panels, Pair a device, New/Edit login, New/Edit exception) move to `footer`
   and `form` like the 07 waves.
4. **Not built**: Advanced / Socks... and Virus checking settings stay greyed, as today (they open
   nothing in the app); the mock's two proposal dialogs are not part of this phase (decision in PLAN.md).
5. **Help**: the `docs/` page about Options describes the side list.

## Tests

- Selecting each page by click, Up/Down and Ctrl+Tab shows the right page; order matches `ORDER`.
- Each page's controls are hit where they are drawn (one click test per page on its first control).

## Done when

- [ ] 08a: every page reachable from the side list and working
- [ ] 08b: all twelve pages and nine sub-dialogs match their mock states in Graphite, light and dark
- [ ] Help updated; screenshots, ROADMAP, CHANGELOG
- [ ] Done within the 3-hour box, or what's left listed under its part's ROADMAP item
