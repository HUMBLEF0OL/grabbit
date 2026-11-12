# 08 · Options as a side list

> **Since 05.5c Graphite is the only look** (`00-conventions.md`): a step here that gates on Graphite, keeps Classic unchanged, uses Silk icons, sizes a window for both layouts or falls back to Classic is read as Graphite's alone; there is no Classic side left.

Replaces the two rows of tabs (which swap front and back when a back-row tab is clicked) with the same
twelve pages listed down the side, in the same order, under the Graphite chrome. Every page keeps its
controls and wording; Classic keeps the tabs, pixel for pixel.

- Branches: `ui/08a-options-list` (side list, pages unchanged inside), `ui/08b-options-pages` (pages on `form`)
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

## 08a · The side list

1. **Side list**: `fn side_list(ui, o, page) -> Option<usize>` draws the twelve entries (16 px icon +
   label, 190 px wide, selected row filled with `selection`, its icon in `accent`) in `ORDER`; today's keys
   keep working exactly (Left/Right/Home/End and Ctrl+Tab move between pages, `options.rs:4130-4136`;
   Enter stays OK; a click selects), and Up/Down are added as 00's keys exception; exposed as `Role::List` / `ListItem` (the tabs'
   accessibility roles move with it). Keep one `ORDER` constant used by both places.
   Icons: General `gx-adjust`, File types `ft-generic`, Save to `cat-folder`, Downloads `gx-download`,
   Connection `gx-network`, Remote `gx-device`, Proxy / Socks `gx-shield`, Sites Logins `gx-key`,
   Accounts `gx-user`, Sounds `gx-volume`, Regional `gx-world`, Archives `ft-archive`.
2. **Page origin**: pages draw at an origin; under Graphite pass `o + (206, 56)` and a page width of
   415 (the tab frame's inner width), so every page's absolute x values land inside its area without
   rewriting the pages. Above the page: its heading (an `icon_tile` with the page icon + the page's
   title, as the mock).
3. **Window size** under Graphite: 206 + 415 + 16 = about 640 x 560, via `dialog_size`; footer via
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
- Classic: tabs and pages unchanged (screenshot diff).

## Done when

- [ ] 08a: every page reachable from the side list and working; Classic unchanged
- [ ] 08b: all twelve pages and nine sub-dialogs match their mock states in Graphite, light and dark
- [ ] Help updated; screenshots, ROADMAP, CHANGELOG
