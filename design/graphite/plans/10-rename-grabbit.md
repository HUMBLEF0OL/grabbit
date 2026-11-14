# 10 · Rename to Grabbit

References re-checked on dev e9d327d3 on 2026-11-14. Line numbers drift: find each by the name
given next to it first.

Renames Mister Download Manager (MDM) to Grabbit everywhere users see it, and moves the identifiers
that carry users' data and integrations with a migration, so no one loses settings, saved logins,
autostart or the browser extension. Independent of the look phases; done last, after the Graphite
look ships, so screenshots are taken once.

Scale on dev e9d327d3: `python tools/rename/check.py --summary` finds 245 user-visible "Mister
Download Manager" and 957 "MDM" hits in 123 files (app 77 + 72 in 47 files, plus 21 in test code;
remote page 4 + 12; installer 1 + 0; extension 39 + 10; website 57 + 241; docs 58 + 604, of which
135 are the benchmark data in `docs/bench/`; README 9 + 18). It counts string literals in
`crates/*/src` (not comments) and every non-comment line of the other areas, minus the kept
identifiers. Across the whole repo (`git grep -c`, lines): "Mister Download Manager" 445 lines in
154 files, the word "MDM" 2,211 in 253, `misterdownloadmanager` 774 in 114, `mdm-` names 485 in
121 plus 1,067 `mdm_*::` paths in 101 Rust files.

- Branches: `rename/10a-display`, `rename/10b-assets`, `rename/10c-migration`, `rename/10d-distribution`
  (pushed to origin = `HUMBLEF0OL/grabbit` only, per 00)
- Needs: 09b; a Grabbit logo before 10b (owner: the maintainer; done, `art/grabbit.svg`, PR 26)
- Release: 10a to 10c ship together in one release; 10d follows the store and site moves

## Decisions owned by this phase (record them in `../PLAN.md` before 10a)

| Decision | Options | Recommendation |
| --- | --- | --- |
| Which repository the app points at (update checker `help.rs:16 REPO`, `help.rs:21 LATEST`, crash reports `crates/crash/src/github.rs:6 REPO`, issue templates) | the upstream `anefzaoui/misterdownloadmanager`; the fork `HUMBLEF0OL/grabbit` | The repository that will publish Grabbit releases; until it publishes them, leave both constants as they are |
| Executable names | keep `misterdownloadmanager*.exe`; rename to `grabbit*.exe` | Keep (least risk); revisit later |
| Crate names `mdm-*` | keep; rename | Keep |

### To decide before 10a

Open for the maintainer; nothing here is decided. Counts are lines (`git grep -c`) on dev
e9d327d3.

1. **Which repository the app points at.** Since e53297fa ("Reports, crash reports and updates go
   to this fork") the app's constants already name the fork: `help.rs:16 REPO`, `help.rs:21
   LATEST`, `crates/crash/src/github.rs:6 REPO`, the three issue templates, and the installer's
   `AppSupportURL` and `AppUpdatesURL` (`mdm.iss:28-29`). So the open part is the rest.
   - a. Everything to the fork (the recommendation's end state once the fork publishes Grabbit
     releases). Costs: 41 lines in 17 files still link the original's repository: README 14,
     docs 19 (9 files), website 5 (`privacy.md`, `features/server.md`, `download.html`,
     `llms.txt`), `tools/website/build.py:45 REPO`, `tools/contributors.py:46`, `mdm.iss:27`
     `AppPublisherURL`. Separately, who the Sponsors links name (`help.rs:19`, `extension/popup.js:8`,
     `tools/website/build.py:54`, `.github/FUNDING.yml:4`, README:345): 5 places.
   - b. Back to the original. Costs: the 4 constants and 3 templates above, and updates would come
     from builds this fork doesn't make.
   - c. Leave as it is (code at the fork, docs and site at the original). Costs nothing now; users
     who follow a docs link to report a problem land on the original's issues.
   The ROADMAP (206) and CHANGELOG (184) links to the original's issues stay either way (history).
2. **Executable names.**
   - a. Keep `misterdownloadmanager*.exe` (recommended). Costs nothing; the names stay visible in
     Task Manager, the install folder, the Run value's command and the CLI/MCP docs.
   - b. Rename to `grabbit*.exe`. Costs: 149 lines in 65 files name an exe: `[[bin]]` in 4
     `Cargo.toml`s (app, cli, mcp, nmhost), `release.yml:107`, the installer, `extension.rs:356`
     host path, `update.rs:293`, docs (`command-line.md` and `ai-assistants.md` 12 lines each),
     the website and the lab and shot scripts in `tools/`. And migrations on top of 10c: the Run value's command,
     the ProgID's open command, the native host manifest's path, the updater replacing a
     differently named running exe, the installer deleting the old exes, Windows asking again
     for the firewall (remote and server modes), and users' scripts and AI assistants'
     MCP settings that call `misterdownloadmanager-cli` / `-mcp` by name.
3. **Crate names `mdm-*`.**
   - a. Keep (recommended). Costs nothing; never shown to users (the checker allows them).
   - b. Rename. Costs: 151 lines in 73 files (`Cargo.toml`s, CI, `cargo -p` in Rust and
     workflows), 1,067 `mdm_*::` paths in 101 Rust files (or none with
     `package = "grabbit-…"` renames in `Cargo.toml`, keeping the `mdm_` paths), plus the docs
     and `CLAUDE.md` commands (`cargo run -p mdm-app` …); every open branch then conflicts.

## Never change

- The Windows installer `AppId` (`tools/installer/mdm.iss:22`): upgrades depend on it.
- The Chrome extension `key` (`extension/manifest.json:8`), the store and unpacked IDs
  (`crates/ipc/src/extension.rs:6 UNPACKED_ID`, `:10 STORE_ID`; the Edge and Opera placeholders at
  `:13 OTHER_STORE_IDS`), the Firefox gecko ID `integration@misterdownloadmanager.com`
  (`manifest.firefox.json:9`, `crates/ipc/src/extension.rs:18 FIREFOX_ID`): changing any of them
  makes a new extension and cuts existing users off.

## 10a · Display text (safe)

`python tools/rename/check.py` lists every remaining user-visible name with file:line; 10a is done
when it exits 0.

1. `MAIN_TITLE` (`crates/app/src/main.rs:72`) becomes "Grabbit <ver>". It is also used to find windows
   by title: `drawn_frame` (`app.rs:6909`), `drops.poll` (`app.rs:8139`), `taskbar_progress`
   (`app.rs:9409`), `caption_dark` (`app.rs:9495`), `find_own_window` (`win32.rs:82, 282`), the
   window's title (`main.rs:351`), the title bars (`app.rs:6946, 6948`) and the tray tooltip
   (`app.rs:3466-3467`, `tray.rs:185`). They all read the constant, so change it in one place and
   test each. `droptarget.rs:14` `TITLE` likewise.
2. Every user-visible string with the name in `crates/*/src` (menus `app.rs:6058, 6241, 6250, 6253,
   7705, 8206`; About and updates `help.rs:345, 364, 641, 662, 722, 888, 988`; tray `tray.rs:136`;
   crash `crashui.rs`; Options title `options.rs:308`; `dialogs.rs`, `extension.rs`, `torrentdlg.rs`,
   `wake.rs`, `headless.rs`, `helpers.rs`, `settings.rs` and the rest the checker lists, including
   `crates/cli`, `crates/crash/src/report.rs`, `crates/nmhost`), "MDM" in sentences → "Grabbit".
3. Exe metadata: `crates/app/build.rs:19-20` ProductName, FileDescription; crate descriptions.
4. User-Agent strings (`crashui.rs:249`, `help.rs:150`, `crates/debrid/src/lib.rs:333 USER_AGENT`,
   `crates/crash/src/github.rs:76`) → `Grabbit/<ver>`. They read `MisterDownloadManager`, which the
   checker allows as an identifier, so it doesn't list them.
5. The remote web page (`crates/remote/web/`), the engine's User-Agent fallback notice
   (`crates/engine/src/download.rs:2484`), the MCP server name (`crates/mcp/src/lib.rs:249`, in
   `get_info`).
6. Extension display texts in `extension/_locales/en/messages.json` (name, description, menu) and the
   translator notes; the store listing names (manual, at release).
7. Tests that assert texts: update them (the checker marks them `[test]`).

## 10b · Assets

1. Done (PR 26): the Grabbit logo, `art/grabbit.svg`.
2. Done (PR 26): `tools/icon/build_icon.py` regenerates `crates/app/assets/icons/app/*.png`, the
   `.ico` used by the installer (`mdm.iss:41`), and the extension icons.
3. Covered by 09a2 (the title-bar and About marks).
4. In PR 27: website logos, favicons, `docs/images/social-preview.png` (`tools/readme/social.py`).

## 10c · Identifiers with users' data (migration)

| Identifier | Where | Migration |
| --- | --- | --- |
| Data folder `…/misterdownloadmanager` | `crates/ipc/src/lib.rs:228-249` (`data_dir`) | Reuse the existing one-time rename (the `idmnext` move at 242-245): if the new folder is missing and the old one exists, rename it once. Portable mode (a `data` folder next to the exe) unchanged |
| `MDM_DATA_DIR` env var | `crates/ipc/src/lib.rs:234` and 60 lines in 31 files | Accept `GRABBIT_DATA_DIR`, keep reading `MDM_DATA_DIR` |
| Saved logins `SERVICE = "Mister Download Manager"` | `crates/app/src/secrets.rs:27` | Read new then old; on finding an old entry, write it under the new name and delete the old one |
| Autostart Run value `MisterDownloadManager` | `win32.rs:3093-3100` (`run_value`), `mdm.iss:104` | On start, if the old value exists, write the new one and delete the old |
| Torrent ProgID `MisterDownloadManager.Torrent` | `win32.rs:3004` (`set_torrent_handlers`), `mdm.iss:106-110, 194` (the installer writes `.torrent`; registry keys ignore case) | Register the new ProgID; repoint `.torrent` if it pointed at the old one; remove the old |
| Single-instance mutex `MisterDownloadManager.Running` | `win32.rs:3992` (`mark_running`), `mdm.iss:133` | App and installer change in the same release; the installer checks both names |
| Native messaging host `com.misterdownloadmanager.host` | `crates/nmhost/src/main.rs:28` (`HOST_NAME`), `extension/background.js:14` (`HOST`), registry keys `win32.rs:4363-4364` (`host_registration`), `mdm.iss:129` | Register **both** host names for at least two releases; the extension tries the new name, then the old |
| Binaries `misterdownloadmanager*.exe` | crate `Cargo.toml`s, Run key, ProgIDs, host manifest path (`crates/app/src/extension.rs:356`) | Per the decision above (recommended: keep) |
| Window-property marker `MDM.popup` | `win32.rs:2191` (`popup_window`) | Internal; rename freely |

`dev` is Windows only: there are no Linux or macOS identifiers to migrate (their code lives on
`archive/multiplatform-i18n`).

Tests: each migration has a test with an old-state fixture (old folder, old secret, old Run value, old
ProgID) that ends in the new state with nothing lost; the existing startup tests in `win32.rs`
(`mod startup_tests`, 3204-3260, including `the_installer_writes_the_same_entry` at 3255, which reads
the `.iss`) updated; `node tools/extension/test.mjs` and `node tools/extension/e2e.mjs` pass with
both host names.

## 10d · Distribution

1. Repository constants per the decision above (no repository is renamed: the fork is already
   `HUMBLEF0OL/grabbit`; the app's constants already name it, see "To decide before 10a"); issue
   templates; 390 links to the original in ROADMAP (206) and CHANGELOG (184) (history entries keep
   their URLs).
2. Update feed: installed versions look for `MisterDownloadManager-{version}-{tail}` assets
   (`crates/tools/src/update.rs:76`; extension assets `.github/workflows/release.yml:144-145`, the
   store zip at 149; setup and zip names from `mdm.iss:45` and `tools/installer/build.ps1`). Publish
   the new asset names **and** the old ones for at least two releases, or until the old builds can
   no longer update.
3. Website and domain (`misterdownloadmanager.com`, 70 lines in 36 files, `help.rs:31 SITE`, the
   schema `$id` in `crates/ipc/src/api.rs:166-173`): new domain with redirects from the old;
   regenerate `docs/automation-api.schema.json`.
4. Docs, README, CONTRIBUTING, PRODUCT, DESIGN, CLAUDE.md, `.impeccable/design.json`; CHANGELOG gets a
   rename entry and keeps old entries as written; ROADMAP gets its own item for the rename (added
   with the groundwork).
5. Crate names `mdm-*`: per the decision above.

## Left from 05.5

- `tools/shots/shots.toml`'s `theme:*` takes (the help's View, Theme pictures) and the docs pages
  that show them still name the removed themes: rewrite them with the docs here. Until then,
  don't run `make.py shoot` for them (they would show Graphite under the old names).
- The website's themes and licence pages, README, PRODUCT.md and CLAUDE.md wording about the old
  themes and "IDM's look" (KNOWN-GAPS G3).
- The control styles of the removed themes (`[controls]` `check`, `button`, `progress`,
  `caption` = `luna`, `aero`, `aqua`, `adwaita`, `breeze`, `metro`) stay as theme-file vocabulary
  with their drawing code: 05.5c kept them, to stay under its size limit. Decide here whether a
  theme file still offers them, and remove them with their drawing if not.
- The docs' theme guide (`docs/themes.md`), the website's theme sample (`tools/website/build.py`,
  `inherits = "classic"`, `icons = "color"`) and its AI prompt (`website/content/theme-prompt.txt`)
  still show the removed themes, the `icons` key and the palette roles 05.5d removed (the classic
  toolbar's and tree's, `tree`, `header.sorted`, `font.ui.bold`): rewrite them here.
- Plans 06 to 09 were adjusted after 05.5 (their Classic steps removed, 09b's default already done).

## Done when

- [ ] No user-visible "Mister Download Manager" or "MDM" left in the app, installer, extension, website,
      docs: `python tools/rename/check.py` exits 0
- [ ] An upgrade from the last MDM release keeps downloads, settings, saved logins, autostart, `.torrent`
      association and the browser extension working (tested on a clean VM with the old version installed first)
- [ ] The never-change list is untouched
- [ ] Old update assets still published; old host name still registered
