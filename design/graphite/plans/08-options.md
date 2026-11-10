# 08 · Options as a side list

Replaces the two rows of tabs (which swap front and back when a back-row tab is clicked) with the same
twelve pages listed down the side, in the same order, under the Graphite chrome. Every page keeps its
controls and wording; Classic keeps the tabs.

- Branch: `ui/08-options`
- Needs: 07a (the kit: footer, form)
- Unblocks: 09

## Facts this plan relies on (paths under `crates/app/src/options.rs`)

- Window 430 x 506 (`:310`); footer OK* (`:336`) and Help (`:344`), no Cancel; Escape closes, Enter is
  OK (`:358-360`).
- `fn tabs` (`:4067` to about `:4240`): `const G` first row (General 0, File types 1, Save to 2, Downloads 3,
  Connection 4, Remote 11), `const P` second row (Proxy 5, Sites Logins 6, Accounts 7, Sounds 8,
  Regional 9, Archives 10); `first_row = page < 5 || page == REMOTE_PAGE` (`:4110`) picks the front row;
  rows at y 9 and 27; the page frame spans x 6..421, y 45..468.
- Keyboard order `ORDER = [0,1,2,3,4,11,5,6,7,8,9,10]` appears twice (Ctrl+Tab `:314`, arrows about `:4146`).
- Page dispatch `:320`; pages: `general` 421, `file_types` 655, `save_to` 783, `downloads` 987,
  `connection` 1128, `proxy` 1371, `accounts` 1699, `logins` 1875, `archives` 2088, `regional` 2224,
  `sounds` 2352, `remote_page` 2579.
- Every page uses absolute English x values inside the 6..421 frame (for example `buttons_to_right(&p,
  392.0, …)` at `:591, 875, 1002`, 389 at `:682, 744`, 406 at `:3891`).
- Sub-dialogs are separate windows (`:2842-3423`, `sub_dialog` `:2996`).

## Steps

1. **Side list**: `fn side_list(ui, o, page) -> Option<usize>` draws the twelve entries (icon + label,
   190 px wide, selected row filled, accent icon) in `ORDER`; Up/Down move, Enter or click selects,
   Ctrl+Tab keeps working; exposed as `Role::List` / `ListItem` (the tabs' accessibility roles move with it).
   Keep one `ORDER` constant used by both places.
2. **Page origin**: pages draw at an origin; under Graphite pass `o + (196, 0)` and a page width of 415
   (the tab frame's inner width), so every page's absolute x values still land inside its area without
   rewriting the pages. A page heading (icon tile + title) is drawn above, and the page body starts below it.
3. **Window size** under Graphite: 820 x 520 (mock), via `dialog_size`; footer via `footer` with Help, OK*
   (Cancel only if decided in 01; today there is none).
4. **Pages**: check each page at the new origin; fix any control that assumed the old frame edges
   (the `buttons_to_right(&p, 392.0…)` calls take the page's right edge from the page width instead).
   Page-specific polish from the mock: Connection's spin box, Remote's status banner, Accounts' warning
   banner. Keep wording.
5. **Sub-dialogs** (DNS settings, Add an account, Addresses left to the browser, Keys, Test browser
   integration, Download panels, Pair a device, New/Edit login, New/Edit exception) move to `footer` and
   `form` like the 07 waves.
6. **Not built**: Advanced / Socks... and Virus checking settings stay greyed, as today; the mock's two
   proposal dialogs are not part of this phase (decision in 01).

## Tests

- Selecting each page by click, Up/Down and Ctrl+Tab shows the right page; order matches `ORDER`.
- Each page's controls are hit where they are drawn (one click test per page on its first control).
- Classic: tabs unchanged (screenshot diff).

## Done when

- [ ] All twelve pages and nine sub-dialogs match their mock states in Graphite, light and dark
- [ ] Classic unchanged
- [ ] Screenshots, ROADMAP, CHANGELOG
