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
| 09b-shots | `tools/shots/` and `docs/` screenshots in Graphite, the `site-theme-graphite` shot |

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
5. **Full review**: every mock state, light and dark, at 100% and 150%, side by side with the mock; a
   list of differences fixed or recorded as follow-ups. It includes:
   - a contrast pass with the real colours (the WCAG tests in `palette.rs` plus a manual check of greyed text);
   - a full keyboard pass (Tab order, Alt and menu letters, Enter and Escape) over every changed window;
   - an accessibility-tree pass over every changed window (names, roles, disabled states, bounds);
   - interface size 200%.

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
  (skipped in 06; the headless tests cover the logic);
- interface sizes 150 % and 200 % for both (06's screenshots are at 100 % only);
- the audit's deferred nits: tests of `route_new_url`'s side effects (they need an `App` built in a
  test), a file name without break points staying on one line in the pane's header, tests of the
  splitter and of a short pane, and `/show details` while a column filter is set.

## Left from 07

09a's full review (step 5) takes these, from 07c's, 07d's and 07e's audits:

- the progress window's **Hide tab** at the page's top right, as the mock has it, 30 high: the
  Speed Limiter and completion pages need their rows checked for room first (it keeps its old
  place and size until then);
- the torrent window's **General** tab: the mock's info banner ("Complete, sharing", the rate to
  its peers) above the property table. The table starts at a fixed 274 under the piece bar, so
  the banner needs that page's geometry reworked (the frame's buttons are 07c's row already);
- `/show` entries for the progress window's error state and its tab row's menu (07c's
  screenshots), and for Create folder, Archive password, Add category (the script shoots
  Properties) and the drop target (07e's), each without a real side effect; the queue-done
  countdown gets none that runs (it turns the computer off after 30 s);
- the **Scheduler** at the mockup's 800 (it keeps 651 and its panes' places), and short enough for
  a 1080p screen at 200 % (its client is 527 high: at 200 % it scrolls by about 45 px).

## Done when

- [ ] Graphite's default button and progress show the two-tone finish; dark mode shows the glow; light does not
- [ ] Frame-time budget met
- [ ] Every mock state reviewed (visual, keyboard, accessibility, 200%); follow-ups filed
- [ ] An upgrade from the last release starts in Graphite, and the release notes say so
- [ ] Screenshots, docs, website, ROADMAP, CHANGELOG
- [ ] Done within the 3½-hour box, or what's left listed under its part's ROADMAP item
