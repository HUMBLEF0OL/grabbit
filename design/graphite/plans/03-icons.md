# 03 · The Graphite icon set (Tabler)

Gives the Graphite theme its own icons: the 53 icon slots, from Tabler Icons 3.31 (MIT), outline at
stroke 1.75, tinted with the theme's ink.

- Branch: `ui/03-icons`
- Needs: 01 (the 106 exported SVGs, SOURCES.txt, LICENSE), 02 (the theme file)
- Unblocks: 03b, 05b, 07a
- Status: complete, merged into the local `dev` on 2026-11-11 (not on origin yet, no PR); the checks
  are under "Checks done" below. The facts below describe the code before it; the set now lives at
  `themeicons.rs` `SETS` (Graphite at about 573-604) and the tests at `:937`, `:972`.

## Facts this plan relies on (paths under `crates/app/`)

- Every slot an icon set can fill is in `src/themeicons.rs` `NEEDS`; a file whose name is not a need is skipped.
- A built-in set is carried with `builtin_files!("<dir>"; 16: [...]; 24: [...])` (`themeicons.rs:281-293`),
  which `include_bytes!` each `assets/themes/<dir>/icons/<px>/<file>`, and registered in `SETS`
  (`themeicons.rs:297-570`) as `sets.insert("<id>", Arc::new(Set::from_files(files, mono)))`.
- `mono = true` (as Metro, `:330`) tints every SVG with the theme's text colour, greyed when disabled.
- Third-party sets keep a `LICENSE` beside `SOURCES.txt` in `assets/themes/<set>/icons/` (Metro: Fluent, MIT).
- Tests that list sets by name: `every_built_in_icon_file_is_a_need_and_carried` (`:841`, the folder must
  match the macro list exactly) and `pictures_come_in_every_size_the_loader_asks_for` (`:873`).
- The theme points at its set with `icon-set = "<id>"`.

## Steps

1. Copy `design/graphite/assets/icons/{16,24}/*.svg` to `crates/app/assets/themes/graphite/icons/{16,24}/`,
   with `SOURCES.txt` and `LICENSE`.
2. In `themeicons.rs` add `let graphite = builtin_files!("graphite"; 16: [ …53 names… ]; 24: [ …53 names… ]);`
   and `sets.insert("graphite".to_owned(), Arc::new(Set::from_files(graphite, true)));`, with a comment
   naming the source and licence like Metro's.
3. Add `"graphite"` to the name lists of the two tests above.
4. In `graphite.toml`: `icon-set = "graphite"` and `icons = "mono"`. The file is written by
   `design/graphite/source/make_theme.py`: change its `HEADER` and run it, not the file.
5. `ATTRIBUTION.md`: Tabler Icons 3.31.0, Paweł Kuna, MIT, used for the Graphite theme's icons.
6. Check the title-bar glyphs (`ui-minimize`, `ui-maximize`, `ui-restore`, `ui-close`, `ui-to-tray`)
   in the drawn caption, and the file-type icons in the list (`ft-*`), at 100% and 200%.
7. Added in review: the categories tree draws no badge over the Graphite set's icons, as the mock.
   The set says so (`Set::no_badges`, true for `graphite` only), `themeicons::tree_badges()` reads
   the active theme's set, and `app.rs` `tree_rows` then leaves out Finished's tick and Queues' page
   stack and draws each queue as `cat-queues`; a synchronized queue keeps its tick (03b's
   `gx-refresh` replaces it in 05b). Classic, the other sets and a folder's own set keep the badges.

## Mock states to compare

Icons and colour section (the sheet of 53), main window toolbar and categories, list file icons,
right-click menu, a few dialogs with icons (Download complete, Scheduler queue list): light and dark.
The Scheduler's queue list keeps Silk's folders in every theme: left to 07e (07-dialogs.md, "Left
from 03"), whose icons are 03b slots.

## Done when

- [x] All 53 needs come from the Graphite set when the theme is chosen; other themes unchanged
- [x] Icon tests pass with "graphite" listed
- [x] Screenshots before and after, light and dark
- [x] Left from 02: the help's View → Theme picture (`view-theme` in `tools/shots/shots.toml`)
      still lists nine themes without Graphite. Retake it with the Docs screenshots workflow (a local
      run on the maintainer's machine gave one-colour pictures for every submenu shot), and update
      its alt text in `shots.toml` and in `docs/getting-started.md` with it
- [x] ATTRIBUTION, ROADMAP, CHANGELOG updated
- [ ] On origin: the PR into `dev` (00, Paperwork), its screenshot folder renamed to the PR's number

## Checks done (2026-11-11, on `dev` at 15141cf)

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo test --workspace` | 61 suites, 825 tests, all pass; new: `graphite_has_every_need_in_one_colour` (every file one colour as carried, sizes, which sets keep the tree's badges) |
| `node tools/extension/test.mjs` | 69 checks pass (the extension is not touched) |
| `cargo build --release`, the app run | built; every shot below is that build |
| Mock: the sheet of 53 | every shape of all 53 carried icons is in `grabbit-design.html` (the same Tabler files, `export_icons.py` writes both copies) |
| Mock states, light and dark, 100 % | main window (toolbar, categories, file icons), right-click menu, Download complete, progress window title bar, Scheduler: `pr-shots` `03-icons/` |
| Before-shots | Graphite at the branch point ecc0166: `02-theme/` and `03-icons/before-*` |
| Step 6 at 200 % | title-bar glyphs (Minimize, Maximize, Close, Hide to the notification area) and every `ft-*` icon sharp, in the ink, light and dark: `03-icons/*-200*` |
| Interface size 150 % | not separately shot; drawings scale (16 px up to 20 px on screen, 24 above, tested in `graphite_has_every_need_in_one_colour`) |
| Classic unchanged | the first-start main window, Classic, pixel-identical to the phase 01 baseline (0 of 1862 × 998 pixels differ, light and dark): `03-icons/classic-*` |
| Screen reader | the accessibility tree of the main window, right-click menu and Download complete identical before and after (every node's name, type and place; 25, 42 and 7 nodes, light and dark) |
| Keyboard | not driven live (an off-screen test copy takes no keys, and Windows' foreground lock refused an on-screen one); the phase changes no key or focus code (`git diff ecc0166 dev -- crates/app/src`), and the tree's keys walk `tree_rows`' views and expanders, which are unchanged (the accessibility tree above) |
| External audit | five rounds (`claude -p`, read-only); the last found nothing in the code, and its paperwork points are fixed here |
