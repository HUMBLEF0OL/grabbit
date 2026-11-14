# Known gaps

What the Graphite work does not cover yet, why, what stands in for it, and what closes it. One
entry per gap. When a gap is closed, its entry moves to "Closed" with the date and how.
Plans and PRs link here instead of explaining a missing check again.

## Open

### G2 · Windows high contrast (plan 05.5)

- **What:** with Native and High contrast removed in 05.5c, turning on Windows' high contrast no
  longer changes the app's colours.
- **Accepted by the owner (2026-11-12):** high contrast is deferred, to come back as a Graphite
  variant.
- **What closes it:** a high-contrast Graphite variant that follows Windows' setting.

### G3 · Docs, help pictures and website show the old looks (plan 05.5 → 10)

- **What:** after 05.5c, the docs, the help pictures (`tools/shots/shots.toml`'s `theme:*` takes),
  the website's themes and licence pages, README, PRODUCT.md and CLAUDE.md still describe the
  old themes and "IDM's look".
- **Accepted:** rewritten with the rename (plan 10).
- **Until then:** no release (`dev` → `main`), since the website is published from `main`.
- **What closes it:** plan 10.

### G4 · Screenshots owed by the upstream-port PRs (2026-11-14)

- **What:** CLAUDE.md rule 6 asks each changed window in pictures, before and after, light and
  dark. The upstream-port batches were merged with their checks as proof and these pictures
  still owed:
  - PR 39: **View → Draw with**'s reason line; the progress window's error for a file host's
    page (**Open in browser**).
  - PR 41: **Options → Save to**'s new **Mark downloaded files as coming from the internet**
    row (the temporary-folder row below it moved down 22 px); **Download video** with a
    sound-only choice (Music's folder) and with a title that has line breaks; the progress
    window's connections row while yt-dlp works; the notice when a file dragged out of
    **Download complete** was moved elsewhere.
  - PR 42: **Options → Downloads**' User-Agent row with its new **Automatic** box;
    **Download video**'s JavaScript-runtime note for YouTube.
- **Until then:** the PRs' test output stands in for them (each window's logic is tested).
- **What closes it:** one shot run (`plans/00-conventions.md`, Screenshots) for these windows,
  put on pr-shots under each PR's number and linked from its description. Plan 09's review
  (R1–R3) passes through every one of these windows and can take them there.

## Closed

### G1 · No screenshots on this PC (2026-11-12 to 2026-11-13)

- **What:** offscreen captures (`MDM_OFFSCREEN=1`, `tools/lab/cap-own.ps1`) come out completely
  black on the maintainer's PC. This happened with the debug and release builds and with every
  renderer (`warp`, `opengl`, the default). The app needs no installing for shots: they run a
  build from the repository. The capture is what fails.
- **What it blocks:**
  - CLAUDE.md rule 6, the before and after pictures in each PR (light and dark, each changed
    window);
  - plan 05.5c step 9, the pixel-identical proof that Graphite is unchanged.
- **Accepted by the owner (2026-11-12):** PRs go without pictures and link this entry instead.
- **What stands in for it:**
  - **Which assets are drawn:** the logged run (plan 05.5, "The final check"). It is a scratch
    build that writes one line per old asset it reaches, across about 56 windows, light and dark,
    100 to 200 %. It is text, so it works here.
  - **Graphite unchanged (05.5c):** the logged run's list of every icon reached, before and after,
    must be identical. Every test passes, with the test-count change explained. These show the
    same icons are reached; unlike pixels, they don't show their places, sizes or colours.
  - **Each PR says so:** "No screenshots: see KNOWN-GAPS G1", with the logs in place of the
    pictures (rule 6 allows "proof (tables, logs) instead" for a change with nothing to see; this
    one has something to see, so the gap is named, not hidden).
- **What closes it:**
  - a capture that works again (check: `docs-shots.ps1 -Show about=about` gives a picture that
    isn't black, e.g. with the display on and unlocked);
  - or shots on screen (`-At "x,y"`) or on GitHub's runner, if the owner agrees.
  
  Then take the missing pictures for every PR listed below, and the 05.5c pixel comparison.
- **PRs merged under this gap** (pictures owed):
  - PR 13, 05.5a: the Options page headers, the helpers window, Download complete's drag
    handle, the Scheduler's queues, Download File Info's Server row (light and dark, 100 to
    200 %); and `gx-computer`'s look check in the app beside its neighbours.
  - PRs 14 and 15, 05.5c: the pixel pairs that show Graphite unchanged, light and dark, for
    every window in the plan's final check (the logged runs stand in for them: the same icons
    reached before and after).
- **Closed 2026-11-13, with the owner's agreement:** shots on screen
  (`docs-shots.ps1 -At "40,40"`), with `cap-own.ps1` made per-monitor DPI aware while it captures:
  at 200 % its `PrintWindow` gave only the top-left quarter of an on-screen window otherwise.
  Offscreen captures still come out black on this PC; how to shoot is in `plans/00-conventions.md`
  (Screenshots). Neutral work folder (`E:\shots`), the pointer parked off the windows.
  - PR 13 (05.5a): the 18 windows it changed, before (6303cfe5) and after (06835db6), light and
    dark at 100 %, light at 150 % and dark at 200 %, on pr-shots `13/`. Every change is on an
    icon; the rest are live values (the progress window, Regional's sample clock).
  - PRs 14 and 15 (05.5c) and 16 (05.5d): all 56 windows of the final check, before (06835db6)
    and after (57fa360f), light and dark, on pr-shots `14/`. 39 of 56 identical in each mode;
    the others differ only by live values (a running download's numbers, the progress window,
    Regional's sample clock), About's build commit, and View → Theme's submenu (intended).
