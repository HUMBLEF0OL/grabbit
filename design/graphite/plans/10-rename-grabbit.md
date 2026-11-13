# 10 · Rename to Grabbit

Renames Mister Download Manager (MDM) to Grabbit everywhere users see it, and moves the identifiers
that carry users' data and integrations with a migration, so no one loses settings, saved logins,
autostart or the browser extension. Independent of the look phases; done last, after the Graphite
look ships, so screenshots are taken once.

Scale: about 550 "Mister Download Manager", 2,600 "MDM", 1,065 `misterdownloadmanager`, 1,410
`mdm-*` crate references across the repo.

- Branches: `rename/10a-display`, `rename/10b-assets`, `rename/10c-migration`, `rename/10d-distribution`
  (pushed to origin = `HUMBLEF0OL/grabbit` only, per 00)
- Needs: 09b; a Grabbit logo before 10b (owner: the maintainer)
- Release: 10a to 10c ship together in one release; 10d follows the store and site moves

## Decisions owned by this phase (record them in `../PLAN.md` before 10a)

| Decision | Options | Recommendation |
| --- | --- | --- |
| Which repository the app points at (update checker `help.rs:15 REPO`, crash reports `crates/crash/src/github.rs:6`, issue templates) | the upstream `anefzaoui/misterdownloadmanager`; the fork `HUMBLEF0OL/grabbit` | The repository that will publish Grabbit releases; until it publishes them, leave both constants as they are |
| Executable names | keep `misterdownloadmanager*.exe`; rename to `grabbit*.exe` | Keep (least risk); revisit later |
| Crate names `mdm-*` | keep; rename | Keep |

## Never change

- The Windows installer `AppId` (`tools/installer/mdm.iss:22`): upgrades depend on it.
- The Chrome extension `key` (`extension/manifest.json:8`), the store and unpacked IDs
  (`crates/ipc/src/extension.rs:6-10`), the Firefox gecko ID `integration@misterdownloadmanager.com`
  (`manifest.firefox.json:9`, `crates/ipc/src/extension.rs:18`): changing any of them makes a new
  extension and cuts existing users off.

## 10a · Display text (safe)

References checked on `ui/03-icons` c3541d0; line numbers drift, find each by its name first.

1. `MAIN_TITLE` (`crates/app/src/main.rs:70`) becomes "Grabbit <ver>". It is also used to find windows
   by title: `drawn_frame` (`app.rs:6344`), `drops.poll` (`app.rs:7260`), `taskbar_progress`
   (`app.rs:8560`), `caption_dark` (`app.rs:8643`), and at `app.rs:2952, 6363` and the tray tooltip
   (`tray.rs:217`). They all read the constant, so change it in one place and test each.
   `droptarget.rs:14` `TITLE` likewise.
2. Every user-visible string with the name in `crates/app/src` (menus `app.rs:5487, 5698, 5707, 6857,
   7327`; About and updates `help.rs:331, 351, 656, 672, 891, 1010`; tray `tray.rs:168`; crash
   `crashui.rs`; Options title `options.rs:308`; `dialogs.rs`, `extension.rs`, `torrentdlg.rs`,
   `wake.rs`, `headless.rs`, `helpers.rs`, `settings.rs`), "MDM" in sentences → "Grabbit".
3. Exe metadata: `crates/app/build.rs:19-20` ProductName, FileDescription; crate descriptions.
4. User-Agent strings (`crashui.rs:243`, `help.rs:148`, `crates/debrid/src/lib.rs:333`,
   `crates/crash/src/github.rs:76`) → `Grabbit/<ver>`.
5. The remote web page (`crates/remote/web/`), the engine notice (`crates/engine/src/download.rs:1570`),
   the MCP server name (`crates/mcp/src/lib.rs:249`).
6. Extension display texts in `extension/_locales/en/messages.json` (name, description, menu) and the
   translator notes; the store listing names (manual, at release).
7. Tests that assert texts: update them.

## 10b · Assets

1. The Grabbit logo (provided by the maintainer; the mock's "G" mark is a placeholder), source image in `art/`.
2. `tools/icon/build_icon.py` regenerates `crates/app/assets/icons/app/*.png`, the `.ico` used by the
   installer (`mdm.iss:41`), and the extension icons; keep output file names or update every reference.
3. The title-bar and About marks of 09a step 3 switch to the logo.
4. Website logos, favicons, `docs/images/social-preview.png` (`tools/readme/social.py`).

## 10c · Identifiers with users' data (migration)

| Identifier | Where | Migration |
| --- | --- | --- |
| Data folder `…/misterdownloadmanager` | `crates/ipc/src/lib.rs:219-241` | Reuse the existing one-time rename (the `idmnext` move at 236-239): if the new folder is missing and the old one exists, rename it once. Portable mode (a `data` folder next to the exe) unchanged |
| `MDM_DATA_DIR` env var | ipc and docs, about 56 uses | Accept `GRABBIT_DATA_DIR`, keep reading `MDM_DATA_DIR` |
| Saved logins `SERVICE = "Mister Download Manager"` | `crates/app/src/secrets.rs:27` | Read new then old; on finding an old entry, write it under the new name and delete the old one |
| Autostart Run value `MisterDownloadManager` | `win32.rs:2672` | On start, if the old value exists, write the new one and delete the old |
| Torrent ProgID `MisterDownloadManager.Torrent` | `win32.rs:2577`, `mdm.iss:106-110` | Register the new ProgID; repoint `.torrent` if it pointed at the old one; remove the old |
| Single-instance mutex `MisterDownloadManager.Running` | `win32.rs:3584`, `mdm.iss:133` | App and installer change in the same release; the installer checks both names |
| Native messaging host `com.misterdownloadmanager.host` | `crates/nmhost/src/main.rs:28` (`HOST_NAME`), `extension/background.js:14`, registry keys `win32.rs:3955-3956`, `mdm.iss:129` | Register **both** host names for at least two releases; the extension tries the new name, then the old |
| Binaries `misterdownloadmanager*.exe` | crate `Cargo.toml`s, Run key, ProgIDs, host manifest path (`crates/app/src/extension.rs:376`) | Per the decision above (recommended: keep) |
| Window-class marker `MDM.popup` | `win32.rs:2001` | Internal; rename freely |

`dev` is Windows only: there are no Linux or macOS identifiers to migrate (their code lives on
`archive/multiplatform-i18n`).

Tests: each migration has a test with an old-state fixture (old folder, old secret, old Run value, old
ProgID) that ends in the new state with nothing lost; the existing startup tests in `win32.rs`
(`2786-2836`, including `the_installer_writes_the_same_entry` at 2828, which reads the `.iss`) updated;
`node tools/extension/test.mjs` and `node tools/extension/e2e.mjs` pass with both host names.

## 10d · Distribution

1. Repository constants per the decision above (no repository is renamed: the fork is already
   `HUMBLEF0OL/grabbit`); issue templates; about 470 URLs in ROADMAP and CHANGELOG (history entries keep
   their URLs).
2. Update feed: installed versions look for `MisterDownloadManager-{version}-{tail}` assets
   (`crates/tools/src/update.rs:76`; extension assets `.github/workflows/release.yml:141-146`; setup and
   zip names from `mdm.iss:45` and the build script). Publish the new asset names **and** the old ones
   for at least two releases, or until the old builds can no longer update.
3. Website and domain (`misterdownloadmanager.com`, 70 references, `help.rs:29`, the schema `$id` in
   `crates/ipc/src/api.rs:166-173`): new domain with redirects from the old; regenerate
   `docs/automation-api.schema.json`.
4. Docs, README, CONTRIBUTING, PRODUCT, DESIGN, CLAUDE.md, `.impeccable/design.json`; CHANGELOG gets a
   rename entry and keeps old entries as written; ROADMAP gets its own item for the rename.
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
- Theme-file vocabulary nothing draws any more, decided with the above: the `icons` key
  (`IconStyle`, which inked the Lucide outlines), and the palette roles of the classic tree, list
  and toolbar (`tree`, `tree.frame`, `tree.lines`, `tree.chevrons`, `toolbar.arrow`,
  `toolbar.split.shadow`/`.light`, `toolbar.line.light`, `icon.disabled.end`, `list.frame`,
  `header.sep`, `sorted`): warn "ignored" or remove them, with the docs' theme guide.
- Plans 06 to 09 carry a note that Graphite is the only look; their Classic steps are gone.

## Done when

- [ ] No user-visible "Mister Download Manager" or "MDM" left in the app, installer, extension, website, docs
- [ ] An upgrade from the last MDM release keeps downloads, settings, saved logins, autostart, `.torrent`
      association and the browser extension working (tested on a clean VM with the old version installed first)
- [ ] The never-change list is untouched
- [ ] Old update assets still published; old host name still registered
