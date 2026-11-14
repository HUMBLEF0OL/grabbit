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

- Branches: `rename/10a-display`, `rename/10b-assets`, `rename/10c-migration`, `rename/10d-distribution`,
  `rename/10e-internal` (last, when no other branch is open)
  (pushed to origin = `HUMBLEF0OL/grabbit` only, per 00)
- Needs: 09b; a Grabbit logo before 10b (owner: the maintainer; done, `art/grabbit.svg`, PR 26)
- Release: 10a to 10c ship together in one release; 10d follows the store and site moves

## Decisions (decided by the maintainer, 2026-11-14)

Recorded in `../PLAN.md`. Counts are lines (`git grep -c`) on dev e9d327d3.

1. **Everything points to the fork, `HUMBLEF0OL/grabbit`, only.** The app's constants already do
   since e53297fa (`help.rs:16 REPO`, `help.rs:21 LATEST`, `crates/crash/src/github.rs:6 REPO`, the
   three issue templates, `mdm.iss:28-29` `AppSupportURL`/`AppUpdatesURL`). Left for 10d step 1:
   41 lines in 17 files that still link the original's repository: README 14, docs 19 (9 files),
   website 5 (`privacy.md`, `features/server.md`, `download.html`, `llms.txt`),
   `tools/website/build.py:45 REPO`, `tools/contributors.py:46`, `mdm.iss:27` `AppPublisherURL`.
   **Sponsors links point to the fork too** (`help.rs:19`, `extension/popup.js:8`,
   `tools/website/build.py:54`, `.github/FUNDING.yml:4`, README:345): the fork owner's GitHub
   Sponsors once it is enabled on that account, the fork's repository page until then. The ROADMAP
   (206) and CHANGELOG (184) links to the original's issues are history and stay.
2. **Executables are renamed to `grabbit*.exe`** (app, cli, mcp, nmhost), in 10c with their
   migrations, after 08 and 09 are merged so no running branch breaks. Costs: 149 lines in 65 files
   name an exe: `[[bin]]` in 4 `Cargo.toml`s, `release.yml:107`, the installer, `extension.rs:356`
   host path, `update.rs:293`, docs (`command-line.md` and `ai-assistants.md` 12 lines each), the
   website and the lab and shot scripts in `tools/` (the `docs-shots`/`baseline` scripts launch the
   exe by name). Migrations: see 10c's "Binaries" row.
3. **Crates are renamed to `grabbit-*`, their code paths stay `mdm_*::`**: each package is named
   `grabbit-…`, and every dependency on it is declared under its old key with
   `mdm-engine = { package = "grabbit-engine", path = … }`, so the Rust code keeps
   `use mdm_engine::…`. 10e, the very last step, when no other branch is open. Costs: 151 lines in
   73 files (`Cargo.toml`s, CI, `cargo -p` in Rust and workflows, the docs and `CLAUDE.md` commands
   such as `cargo run -p mdm-app`); none of the 1,067 `mdm_*::` paths in 101 Rust files.
   **Why the paths stay (the upstream sync):** `tools/upstream-sync/sync.py` cherry-picks upstream's
   commits. Over the 3 months to 2026-11-14, 124 of upstream's 367 commits (34 %) changed lines
   with `mdm_*::` paths: renamed paths would make about a third of upstream's fixes conflict, and
   even clean picks would bring code naming the old crates. Only 7 (2 %) touched the `Cargo.toml`
   name and dependency lines this rename changes. The crate folders (`crates/app`, `crates/engine`…)
   have no `mdm` in them and don't move. Check `sync.py pick --lock` against the renamed packages
   in Cargo.lock as part of 10e.
   **Exes (decision 2), for comparison:** 30 of the 367 (8 %) touched exe names, mostly the
   installer, `release.yml` and the CLI/MCP docs: those parts become small hand fixes in a port.
   **Before 10e:** clear the upstream-sync backlog (`sync.py status`; 36 commits waiting for a port
   on 2026-11-14), so the rename doesn't land on a long queue.
4. **The removed themes' control styles go** (`luna`, `aero`, `aqua`, `adwaita`, `breeze`,
   `metro`), with their drawing code: 10e, with the crate rename. A theme file naming one gets
   Graphite's control (the fork has no users relying on them).

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
| Binaries `misterdownloadmanager*.exe` → `grabbit*.exe` (decision 2) | `[[bin]]` in 4 crate `Cargo.toml`s, Run key, ProgIDs, host manifest path (`crates/app/src/extension.rs:356`), `update.rs:293`, `release.yml:107`, installer | The installer removes the old exes; the Run value's command, the ProgID's open command and the native host manifest's path are rewritten on first start; the updater replaces the running old-named exe; Windows asks again for the firewall (remote and server modes: say so in the release notes); users' scripts and AI assistants' MCP settings that call `misterdownloadmanager-cli` / `-mcp` by name: say so in the release notes and the CLI/MCP docs |
| Window-property marker `MDM.popup` | `win32.rs:2191` (`popup_window`) | Internal; rename freely |

`dev` is Windows only: there are no Linux or macOS identifiers to migrate (their code lives on
`archive/multiplatform-i18n`).

Tests: each migration has a test with an old-state fixture (old folder, old secret, old Run value, old
ProgID) that ends in the new state with nothing lost; the existing startup tests in `win32.rs`
(`mod startup_tests`, 3204-3260, including `the_installer_writes_the_same_entry` at 3255, which reads
the `.iss`) updated; `node tools/extension/test.mjs` and `node tools/extension/e2e.mjs` pass with
both host names.

## 10d · Distribution

1. Everything to the fork (decision 1): the 41 links to the original's repository and the 5
   Sponsors links listed there (no repository is renamed: the fork is already
   `HUMBLEF0OL/grabbit`, and the app's constants already name it); 390 links to the original in ROADMAP (206) and CHANGELOG (184) (history entries keep
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
## 10e · Internal names and old styles (last, when no other branch is open)

1. Packages `mdm-*` → `grabbit-*`, dependencies declared `mdm-x = { package = "grabbit-x", … }`
   so the code's `mdm_*::` paths stay (decision 3), in one mechanical commit with nothing else in
   it; then `CLAUDE.md`, the docs and the workflows' `cargo -p` commands. Then one upstream pick
   with `--lock` as a check that `sync.py` still works.
2. The removed themes' control styles and their drawing code (decision 4; "Left from 05.5" below).
3. Full `cargo build`, `cargo test`, `cargo clippy --all-targets`, `node tools/extension/test.mjs`.

## Left from 05.5

- `tools/shots/shots.toml`'s `theme:*` takes (the help's View, Theme pictures) and the docs pages
  that show them still name the removed themes: rewrite them with the docs here. Until then,
  don't run `make.py shoot` for them (they would show Graphite under the old names).
- The website's themes and licence pages, README, PRODUCT.md and CLAUDE.md wording about the old
  themes and "IDM's look" (KNOWN-GAPS G3).
- The control styles of the removed themes (`[controls]` `check`, `button`, `progress`,
  `caption` = `luna`, `aero`, `aqua`, `adwaita`, `breeze`, `metro`) stay as theme-file vocabulary
  with their drawing code: 05.5c kept them, to stay under its size limit. Decided: they go, in
  10e (decision 4).
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
