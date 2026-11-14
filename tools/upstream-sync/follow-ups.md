# Upstream ports: follow-ups

The ledger records what was decided about each of upstream's commits. This file holds what the
hand ports of 2026-11-14 (PRs 36, 37, 39, 40, 41, 42) left to do: the decisions waiting for
the owner, and what the independent reviews of each port found but didn't block the merge on.
Strike an item through (or remove it) in the PR that settles it.

## Decisions for the owner

1. **Upstream's video-through-engine rework.** Videos in one file, streams in pieces (HLS,
   DASH) and image galleries come down through MDM's own engine, with yt-dlp and gallery-dl only
   listing them: `8e1bf1d0afbd`, `b35b42d4cab5`, `1e4f8e17179a`, with the lab server's H.264
   tracks for its tests (`9b03e225e093`). Three fixes wait on it: verifying and merging that say
   how far they are, and a gap-free map (`7248768eda7c`, `d2c30b88d92c`, upstream issue 736), and
   a failed merge that keeps its parts (`6196f9adeb71`). Also related: the video button's
   qualities menu on the page (`97e31ad5810e`) and video without ffmpeg (`bbbeb4a08cfd`). Take
   them (a large port, several batches), take them later, or skip them.
2. **Connecting exactly as a browser does.** `0067cd0a7a1f` and `d8147ab28c1b` (upstream issues
   668 and 771: sites and Akamai that let only browsers through) add a TLS stack that copies a
   browser's (the `wreq` crate, BoringSSL). It is a new native dependency with its own build
   script. The part without it is ported (PR 42: the default browser's User-Agent, asked again
   with the former one when refused).
3. **Smaller ones, deferred:** the tray icon's menu in MDM's look (`19296802b99c`, with
   Graphite's tray work); CI permissions and pinned actions (`714f76b48408`) and web-ext 10.8.0
   (`02eafc94b5a8`), with the next change to the release process.

## Fixes the reviews found (worth a follow-up PR)

- **Check for updates and the proxy.** Help → Check for updates builds its client with
  `mdm_engine::net::client_builder` (`help.rs`, `latest()`), so it doesn't use the proxy set in
  Options → Proxy / Socks; PR 40 moved the helpers' release lists to `mdm_tools::client()`, which
  does. Give the update check the same client.
- **Open on a type with no program.** Since PR 40, Open goes through plain `ShellExecuteW`
  (`win32::open_file_shell`): for a type with no program it returns `SE_ERR_NOASSOC` and MDM
  shows "could not be opened", where Windows used to offer "How do you want to open this file?".
  Fall back to the Open with chooser (`SHOpenWithDialog`, as Open with does) on `SE_ERR_NOASSOC`.
- **Show in folder off the window's thread.** PR 39 starts the update installer, Open in
  browser and the power options beside the window; Show in folder (`explorer /select,`, in
  `dialogs.rs`) is still started from it, so a slow antivirus check can still freeze the window.
- **Import when no thread can start.** `transfer::Importing::start` keeps the file pending when
  `Asked::get` can't spawn its thread, so the status strip says "Reading" for good. Drop it with
  a notice instead (PR 39).
- **The default browser's User-Agent at start.** `defaultbrowser::start()` runs beside the
  window and nothing waits for it: downloads resumed right at start go with the fallback
  User-Agent; and unticking **Automatic** before the look-up ends fills the box with the
  fallback, which then stays as the user's own (PR 42).
- **A stream's Referer and Origin.** `VideoDialog::for_page` clears the context's Referer when an
  embedded player is taken to the video's own page but leaves the new Origin, which can make the
  same 403; and `push_item` saves the player's Referer as the download's web page in File
  Properties (PR 42).
- **The file-host page's buttons.** Upstream makes **Open in browser** the default button after
  that failure and shows the reason as a wrapped sentence; the port keeps Resume as the accent
  and "Error: …" (PR 39).
- **yt-dlp's connection row while ffmpeg merges.** The fork's `video::State` has no merging
  state, so the row says "Connecting" while ffmpeg joins the streams (PR 41).
- **Tests that depend on the PC.** The Mark of the Web tests (`mark_finished_tests`, the
  torrents' Zone.Identifier test) fail where Windows' "do not preserve zone information" policy
  is on: give them a policy override (PR 41).

## Tests to add (the fixes work; their proof is thinner than upstream's)

- End-to-end cases upstream has and the ports left out: Firefox Alt+click with
  `browser.altClickSave` off (the fork's e2e profile turns it on, so its `alt` case never meets
  the bug), the Firefox `fullscreen` update case, Chromium `folders`, and the engine's shared-link
  cases with the lab server's `link_rate` fault.
- Unit tests that reach the bug, not only the new helpers: Open with's old rundll32 path, the
  tray's `show_for_dialog`, the drag-out follow-up in the app (`poll_dragged_out`, `moved_away`),
  `--no-warnings` gone from yt-dlp's look-up, and the `.mp4.mp4` title through `set_video`.

## Wording and tidying

- CHANGELOG: the helpers line says 7-Zip runs in the temporary folder (it runs in the folder it
  extracts to); the Open line says "the folder was shown" before (the fork only posted a notice).
- **Draw with**: two strings for Vulkan and Metal that Windows never shows.
- Doc comments past 100 columns (`download.rs` `too_slow`, `chrome.rs` `FOCUS_GRACE`), a stale
  `animation_caption` comment (`win32.rs`), `panes.rs` `connections_label`'s comment (a running
  yt-dlp now has a connection), and one way of naming upstream issues in `renderer.rs`.
- `open_with_dialog` initialises COM itself where `com_on_this_thread()` exists.
