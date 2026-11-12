# Upstream ledger

What this fork decided about each of the original MDM's commits (anefzaoui/misterdownloadmanager,
`upstream/main`) since the fork point. `tools/upstream-sync/sync.py` writes the rows (`pick`
and `record`): one per upstream commit, a later decision replacing the earlier one. Edit by hand
only to fix a mistake; `sync.py doctor` reports a bad or duplicated row.

- **picked**: taken as it is (minus what the fork removed); the ref is the fork's commit, whose
  `Upstream-Pick: <sha>` trailer must reach `dev` (a squash merge drops it: `status` then shows
  `picked?`)
- **ported**: written again by hand in the fork's code; the ref is the PR, whose commit carries
  `Upstream-Port: <sha>` (until it merges, `status` shows `ported?`)
- **skipped**: not wanted here (a removed platform or language, an upstream feature)
- **covered**: the fork already fixed it its own way; the ref says where
- **deferred**: looked at, not now (it shows again); the note says what it waits for

| upstream | status | date | ref | upstream subject | note |
|---|---|---|---|---|---|
| f3c11394f3d1260b589d6675777b5af2331c9ec3 | picked | 2026-11-12 | 998ad574c83b | The data folder is worked out once per run, not on every tick (#565) (#582) | left out 2 file(s) |
| 390189f7470f85b38be278f6f08881ce679e4c01 | picked | 2026-11-12 | 64dd27035b4d | Menus: another item closes the submenu at once, one item lit (#579) (#595) | left out 2 file(s) |
| 43a529d91e5f66ec29a93788acd90cec604d1cfe | picked | 2026-11-12 | 19bcf7dd0afb | Download complete: dragging the file out no longer holds up the window (#561) (#592) | left out 2 file(s) |
| feeb7133217d777817b226093d8ff68ce90b7cb2 | picked | 2026-11-12 | 3774fce29116 | Categories without their own folder follow General's folder; start Add torrent with Torrents' folder (#556, #550) (#554) | left out 2 file(s) |
| 39be735d19dfb7649a05b7e8d88951ff3a25cde1 | picked | 2026-11-12 | 48e5868dee9e | Speed limiter: a raised or removed limit applies to the waits already started (#621) (#637) | left out 2 file(s) |
| e7237345530e2a94203de915bc5b1d90da586a23 | picked | 2026-11-12 | 84448dc006f6 | Debrid sign-in by code: only the service's own page is opened (#635) | left out 2 file(s) |
| a255a8b1f185e7be888eb92fbe1527c332fcece9 | picked | 2026-11-12 | ccff44ca91f3 | Cancel in Download File Info asks whether to cancel the waiting downloads too (#394) (#610) | left out 2 file(s) |
| 0198cc4f74e48142838b96348fddbffde1e0a9e0 | picked | 2026-11-12 | 049886da62ff | A link refused before any byte goes on where the browser's download ended (#630) (#642) | left out 2 file(s) |
| a59a2e2cb89ca8719e632fbd6006c57575918816 | picked | 2026-11-12 | f2597059a285 | Test: a passed-over gallery-dl, on a machine that has its own (#648) |  |
| ca7bedbfd3970c229c56cb6df04e49f833344bae | picked | 2026-11-12 | d42b0e48246e | Test: the slow-flush bound grows with the yardstick (#649) |  |
| d45f3f5ba28cb1b69239370fbd34142acc584a2a | picked | 2026-11-12 | dd3070b457e7 | Crash reports: Linux stacks read from the program's start, a C-runtime stall is a busy computer (#661, #663) (#669) | left out 2 file(s) |
| 817a7b4c818d63c8dbfead7e9b46e45c85feddfc | picked | 2026-11-12 | 2251534c83e2 | Torrents: the window never waits on the torrent engine (#693) (#702) | left out 2 file(s) |
| 34ac3595267745a5a07581b07348d8874ac1bd92 | picked | 2026-11-12 | 31a49e25d595 | Videos and galleries: the speed limits reach yt-dlp and gallery-dl, and a change starts them again (#708) (#775) | left out 3 file(s) |
| 87cf289f8ded68ccdb651732a273c7359d977b2b | picked | 2026-11-12 | 69486b183817 | Icons: the file-type icons no longer fill the log with SVG warnings, so reports keep MDM's own lines (#767) (#774) | left out 2 file(s) |
| 68ba617edb3d29af691d7410079c5ab773c766a5 | picked | 2026-11-12 | 2ea1059d6cb5 | Windows 10: the progress window without a black band above its title bar (#620) (#652) | left out 2 file(s) |
| ff0560fd399dcee9b210b141120c4dff9d358298 | picked | 2026-11-12 | 47c9fd217cd7 | Windows: the C runtime is inside MDM's programs, so a missing or 32-bit VCRUNTIME140.dll can't stop them (#785) (#786) | left out 2 file(s) |
| 3d11d7ba7c5d5bdfe4092b43916e32c3390eff48 | picked | 2026-11-12 | f6dfb78a70a7 | Video button: one place over YouTube's thumbnail previews (#560) (#591) | left out 2 file(s) |
| 0612aa8a444cc121431407be702ceddbf6bdce65 | picked | 2026-11-12 | 0dc183703e71 | Fix/603 colab runtime proxy only (#729) | left out 1 file(s) |
| ba2cbabd68d10434ba4ac5d99791c6501fc2f6c3 | picked | 2026-11-12 | d12fa344273d | Windows: the clipboard read and written one at a time, no more heap corruption (#629) (#639) | left out 2 file(s) |
| b54f8b7b56cb278bd3f97f051054fbe7d4518ede | skipped | 2026-11-12 |  | funding.json: the open funding manifest (#596) | upstream's funding manifest: its identity, not this fork's |
| b936414ecb7b29f7e6dbfd63a601cfdbad3223f4 | skipped | 2026-11-12 |  | Linux and macOS: the system's gallery-dl, as yt-dlp's (#571) (#594) | Linux and macOS only (gallery-dl from the system): removed platforms |
| 5bfb9bb0211e8c8fa3569abbed0bbb3e4043c4e6 | skipped | 2026-11-12 |  | macOS: downloads to a network volume no longer refused for 0 bytes free (#623) (#638) | macOS only (network volumes): removed platform |
| 9fa1e58a28df5b53ee99af7aed42124442c1d43e | skipped | 2026-11-12 |  | Search: point the README, the add-on and the site's data to misterdownloadmanager.com (#658) | points README, add-on and site data at upstream's site: identity, the rename's work |
| 8200e91208a49cc8bcf82eb04be74bc139066ad5 | skipped | 2026-11-12 |  | macOS: helpers get Homebrew's folders on their PATH (#659) (#662) | macOS only (Homebrew folders on PATH): removed platform |
| 4843a8b7fd6d12c0304b2d3c87deaa617b44cb78 | skipped | 2026-11-12 |  | Test: the running helper copy works on Ubuntu 26.04 (its sleep is a multi-call program) (#681) | a test for Ubuntu: removed platform |
| 460ab099c093ed8540caaaae7fec864daac20cef | skipped | 2026-11-12 |  | Helium: MDM offers it the extension on Linux and macOS, by Helium's own name for the store (#685) (#725) | Helium on Linux and macOS: removed platforms |
| cc3922efc4b1b714d923a27779a395d1eb20c938 | skipped | 2026-11-12 |  | Extension: the video button's corners and divider follow the text direction (#744) | the video button's right-to-left layout: right-to-left removed |
| 8742fcd833eae2a241d64da98eff79b5ec7dbfa6 | skipped | 2026-11-12 |  | Themes: Emerald, a white canvas with one emerald fill and near-black ink (from #45) (#522) | feature (the Emerald theme); revisit on request |
| 42e0a83b3e4f1b539e97a521dcc2abf9e419752f | skipped | 2026-11-12 |  | Smooth Windows text (View → Text) and 110 / 115 / 120 % interface sizes (#478) | feature (smooth text, 110-120 % sizes) in Graphite's files; revisit on request |
| 26f5b00f8943d732bbd6725b7eb8d66cc7029526 | skipped | 2026-11-12 |  | feat(debrid): add TorBox for file-host links and torrents (#566) | feature (TorBox debrid service); revisit on request |
| 7adb2f9fa191d22e762ffb7e00a5a7dc0001515d | skipped | 2026-11-12 |  | feat(progress): add safe deferred Rename row on Download status tab (#655) | feature (a Rename row on the status tab); revisit on request |
| 714f76b4840843bae2841c1c1ab0ee78ea4c6bef | deferred | 2026-11-12 |  | CI: each job gets only the permissions it needs; actions and release tools pinned (#580) | CI hardening (job permissions, pinned actions): its own PR for this fork's main-only workflows |
| 02eafc94b5a810a91e443b1073ae9898cce32bf4 | deferred | 2026-11-12 |  | Release tools: web-ext 10.8.0, with the dependencies it brings (#598) | release tools' web-ext bump: with the next change to the release process |
| bbbeb4a08cfd13d067a7d374f8d74a792d8ace61 | deferred | 2026-11-12 |  | Video without ffmpeg: a sound that comes as a file, and HLS pieces saved as what they are (#569) (#593) | wanted fix (video without ffmpeg): port, conflicts in the video code |
| a5a75803718884cbdfe7593ac6e70ebfeae2d133 | deferred | 2026-11-12 |  | Extension: downloads it leaves keep another extension's name and folder (#578) (#584) | wanted fix (extension keeps another extension's name and folder): port |
| 70599ffc0a2ff2703bab7d1d52b749c91582b471 | deferred | 2026-11-12 |  | fix(tray+handoff): tray click toggles window; browser downloads open only their dialog (#520) | wanted fix (tray click, browser downloads' dialog): port with the other tray fixes |
| b488f2b539d9063ade0fa8b4904665c5f2447ebb | deferred | 2026-11-12 |  | Tray click on Windows: in front is what covers the window, not the focus (#640) | wanted fix (tray click brings the window in front): port with the other tray fixes |
| 7dc0e3a9567b26ca210c6afcff68227011fee769 | deferred | 2026-11-12 |  | Download video: a sound-only choice saves in Music (#611) (#614) | wanted fix (sound-only video saves in Music): port |
| 69aeb6db14846713ea5c3004e1ef3907af98eb3f | deferred | 2026-11-12 |  | Torrents: the DHT joins through four public routers, and the port picked is kept (#650) (#664) | wanted fix (torrent DHT routers, kept port): port |
| de60fb70ace3a827d800ddf63617cc728076950a | deferred | 2026-11-12 |  | Links added by hand go with the default browser's User-Agent (part of #668) (#691) | wanted fix (browser User-Agent for links added by hand, upstream issue 668): port with its group |
| 0067cd0a7a1f3d3dc8e3a3ea9a1ce476490eef3e | deferred | 2026-11-12 |  | Sites that let only browsers through: one more try the way a browser connects (part of #668) (#748) | wanted fix (one more try as a browser connects, upstream issue 668): port with its group |
| d8147ab28c1b910e8342ef336ba3f2e879f44921 | deferred | 2026-11-12 |  | Downloads: Akamai's refusal of programs is a bot check, so MDM tries again as a browser connects (#771) (#776) | wanted fix (Akamai bot check, upstream issue 771; builds on upstream issue 668's group): port with it |
| 380cdb3ed029caff849465b1e01c54d8312c2f03 | deferred | 2026-11-12 |  | Extension: a video in a feed sends its post, not the feed (#689, #643) (#707) | wanted fix (extension: a video in a feed sends its post): port |
| 19296802b99cb96aae4f382ea3d08cef91d11445 | deferred | 2026-11-12 |  | Windows: the tray icon's menu follows MDM's look, dark or light (#682) (#722) | tray menu's look: decide with Graphite's tray work |
| 50a3985607b73e2fa943d48fbf69b23be838eadc | deferred | 2026-11-12 |  | Tests: the slow-flush case measures the transfer, not the runner's disk (part of #518) (#728) | test robustness (slow-flush case): port with the engine's tests |
| 97e31ad5810e4e4e059683e19cd4d766493b7697 | deferred | 2026-11-12 |  | Video button: qualities in a menu on the page, with Video/Audio tabs and fast look-ups; HLS sizes, duplicate videos, a smooth progress bar (#449) | feature (video button qualities menu) carrying fixes (HLS sizes, duplicates): read its fixes in a later run |
| 8e1bf1d0afbd3994b437d4c84ef3e052d9f35975 | deferred | 2026-11-12 |  | Videos in one file come down through MDM's engine; yt-dlp only finds them (part of #769) (#781) | feature (videos through MDM's engine, upstream issue 769): a decision of its own |
| b35b42d4cab545fb63cba9765681ad4c7f67c4f2 | deferred | 2026-11-12 |  | Streams in pieces (HLS, DASH) come down through MDM's engine (part of #769, #531) (#783) | feature (streams through MDM's engine, upstream issue 769): with 8e1bf1d0af |
| 1e4f8e17179a6ab3199f953b30d91800c0e707c9 | deferred | 2026-11-12 |  | Image galleries come down through MDM's engine; gallery-dl lists the files (fixes #769) (#784) | feature (galleries through MDM's engine, upstream issue 769): with 8e1bf1d0af |
| 98418eee0c264a9f58370f9d8ce9ea7901eea642 | ported | 2026-11-12 | PR 5 | Clipboard: every read off the window's thread (follows #418) (#688) | ported by hand: clipboard read off the window's thread |
| 7ec5bf056a4a6f81cfd5cdb9303b8e57caea9c39 | ported | 2026-11-12 | PR 5 | Tests: the clipboard tests one at a time (#644) | ported by hand: clipboard read off the window's thread |
| 32881b9d7f2066c37b85abdeda1c7a100272298c | ported | 2026-11-12 | PR 6 | Torrents: a pause during a piece's hash check no longer leaves the torrent one piece short (#706) (#717) | rqbit pin moved to 694291b (brings ac7fa6e, the library half of 650) |
| 5213ad1f036f0814d20b9b3399ff22204ba74200 | ported | 2026-11-12 | PR 7 | Windows: the title bar's light or dark set once, not checked with DWM every 2 s (#633, #583) (#641) | cfg(windows) branches dropped; caption set once per change |
| 6b4a76255e3cb943bbaf791ca27fcb3483c00aa0 | ported | 2026-11-12 | PR 8 | Windows: Close hides MDM in the notification area again, and the tray icon's questions leave the window's thread (#750, #747) (#764) | watcher thread for the tray icon; macOS/Linux/relabel/Toggle left out |
