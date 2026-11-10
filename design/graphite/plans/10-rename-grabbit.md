# 10 · Rename to Grabbit

Renames Mister Download Manager (MDM) to Grabbit everywhere users see it, and moves the identifiers
that carry users' data and integrations with a migration, so no one loses settings, saved logins,
autostart or the browser extension. Independent of the look phases; best done last, after the
Graphite look ships, so screenshots are taken once.

Scale: about 550 "Mister Download Manager", 2,600 "MDM", 1,065 `misterdownloadmanager`, 1,410
`mdm-*` crate references across the repo.

- Branches: `rename/10a-display`, `rename/10b-assets`, `rename/10c-migration`, `rename/10d-distribution`
- Needs: decision in 01; a Grabbit logo (10b)
- Release: 10a to 10c ship together in one release; 10d follows the repository and store moves

## Never change

- The Windows installer `AppId` (`tools/installer/mdm.iss:22`): upgrades depend on it.
- The Chrome extension `key` (`extension/manifest.json:8`), the store and unpacked IDs
  (`crates/ipc/src/extension.rs:6-10`), the Firefox gecko ID `integration@misterdownloadmanager.com`
  (`manifest.firefox.json:9`): changing any of them makes a new extension and cuts existing users off.

## 10a · Display text (safe)

1. `MAIN_TITLE` (`crates/app/src/main.rs:75`) becomes "Grabbit <ver>". It is also used to find windows
   by title: `drawn_frame` (`app.rs:6429`), `drops.poll` (`app.rs:7360`), `taskbar_progress`
   (`app.rs:8676`), `caption_dark` (`app.rs:8768`). They all read the constant, so change it in one place
   and test each. `droptarget.rs:14` `TITLE` likewise.
2. Every user-visible string with the name in `crates/app/src` (menus `app.rs:5580, 5791, 5800, 6955,
   7436`; About and updates `help.rs:370, 692, 708, 1046`; tray `tray.rs:195, 244`; crash `crashui.rs`;
   Options title `options.rs:309`; `dialogs.rs`, `extension.rs`, `torrentdlg.rs`, `wake.rs`, `headless.rs`,
   `helpers.rs`, `settings.rs`), "MDM" in sentences → "Grabbit".
3. Exe metadata: `crates/app/build.rs:28-29` ProductName, FileDescription; crate descriptions.
4. User-Agent strings (`crashui.rs:249`, `help.rs:163`, `crates/debrid/src/lib.rs:333`,
   `crates/crash/src/github.rs:76`) → `Grabbit/<ver>`.
5. The remote web page (`crates/remote/web/`), the engine notice (`crates/engine/src/download.rs:1578`),
   the MCP server name (`crates/mcp/src/lib.rs:249`).
6. Extension display texts in `extension/_locales/en/messages.json` (name, description, menu) and the
   translator notes; the store listing names (manual, at release).
7. Tests that assert texts: update them.

## 10b · Assets

1. A Grabbit logo (to be designed; the mock's "G" mark is a placeholder). Source image in `art/`.
2. `tools/icon/build_icon.py` regenerates `crates/app/assets/icons/app/*.png`, the `.ico` used by the
   installer, and the extension icons; keep output file names or update every reference.
3. Website logos, favicons, `docs/images/social-preview.png` (`tools/readme/social.py`).

## 10c · Identifiers with users' data (migration)

| Identifier | Where | Migration |
| --- | --- | --- |
| Data folder `…/misterdownloadmanager` | `crates/ipc/src/lib.rs:219-241` | Reuse the existing one-time rename (the `idmnext` move at 236-239): if the new folder is missing and the old one exists, rename it once. Portable mode (a `data` folder next to the exe) unchanged |
| `MDM_DATA_DIR` env var | ipc and docs, 67 uses | Accept `GRABBIT_DATA_DIR`, keep reading `MDM_DATA_DIR` |
| Saved logins `SERVICE = "Mister Download Manager"` | `crates/app/src/secrets.rs:32` | Read new then old; on finding an old entry, write it under the new name and delete the old one |
| Autostart Run value `MisterDownloadManager` | `win32.rs:2675` | On start, if the old value exists, write the new one and delete the old |
| Torrent ProgID `MisterDownloadManager.Torrent` | `win32.rs:2580`, `mdm.iss:106-110` | Register the new ProgID; repoint `.torrent` if it pointed at the old one; remove the old |
| Single-instance mutex `MisterDownloadManager.Running` | `win32.rs:3587`, `mdm.iss:133` | App and installer change in the same release; the installer checks both names |
| Native messaging host `com.misterdownloadmanager.host` | `crates/nmhost/src/main.rs:38`, `extension/background.js:14`, `browsers.rs:382`, `mdm.iss:129`, `unix.rs:173-175` | Register **both** host names for at least two releases; the extension tries the new name, then the old |
| Binaries `misterdownloadmanager*.exe` | crate `Cargo.toml`s, Run key, ProgIDs, host manifest path (`extension.rs:573`) | Decide: keep the old exe names (least risk) or rename with the installer removing the old files and every path above updated in the same release |
| Window-class marker `MDM.popup` | `win32.rs:2002` | Internal; rename freely |

Linux and macOS identifiers (LaunchAgent, `.desktop`, secret schema, bundle IDs, Debian package) are
out of scope: `dev` is Windows only (their code is on `archive/multiplatform-i18n`).

Tests: each migration has a test with an old-state fixture (old folder, old secret, old Run value, old
ProgID) that ends in the new state with nothing lost; the existing `win32.rs` test that reads the `.iss`
text (`2785-2834`) updated.

## 10d · Distribution

1. Repository rename (GitHub keeps redirects): `help.rs:15 REPO`, `crates/crash/src/github.rs:6`, issue
   templates, about 470 URLs in ROADMAP and CHANGELOG (history entries may keep the old URLs).
2. Update feed: installed versions look for `MisterDownloadManager-{version}-{tail}` assets
   (`crates/tools/src/update.rs:131`, `.github/workflows/release.yml:144-149`). Publish the new asset names
   **and** the old ones for at least two releases, or until the old builds can no longer update.
3. Website and domain (`misterdownloadmanager.com`, 70 references, `help.rs:29`, the schema `$id` in
   `crates/ipc/src/api.rs:166-173`): new domain with redirects from the old; regenerate
   `docs/automation-api.schema.json`.
4. Docs, README, CONTRIBUTING, PRODUCT, DESIGN, CLAUDE.md, `.impeccable/design.json`; CHANGELOG gets a rename
   entry and keeps old entries as written.
5. Crate names `mdm-*`: internal; rename only if wanted, in a PR of its own.

## Done when

- [ ] No user-visible "Mister Download Manager" or "MDM" left in the app, installer, extension, website, docs
- [ ] An upgrade from the last MDM release keeps downloads, settings, saved logins, autostart, `.torrent`
      association and the browser extension working (tested on a clean VM with the old version installed first)
- [ ] The never-change list is untouched
- [ ] Old update assets still published; old host name still registered
