# 03 · The Graphite icon set (Tabler)

Gives the Graphite theme its own icons: the 53 icon slots, from Tabler Icons 3.31 (MIT), outline at
stroke 1.75, tinted with the theme's ink.

- Branch: `ui/03-icons`
- Needs: 01 (the 106 exported SVGs, SOURCES.txt, LICENSE), 02 (the theme file)
- Unblocks: 03b, 05b, 07a
- Status: built on `ui/03-icons` (in review). The facts below describe the code before it; the set now
  lives at `themeicons.rs` `SETS` (Graphite at about 573-604) and the tests at `:937`, `:972`.

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

## Mock states to compare

Icons and colour section (the sheet of 53), main window toolbar and categories, list file icons,
right-click menu, a few dialogs with icons (Download complete, Scheduler queue list): light and dark.

## Done when

- [ ] All 53 needs come from the Graphite set when the theme is chosen; other themes unchanged
- [ ] Icon tests pass with "graphite" listed
- [ ] Screenshots before and after, light and dark
- [ ] Left from 02: the help's View → Theme picture (`view-theme` in `tools/shots/shots.toml`)
      still lists nine themes without Graphite. Retake it with the Docs screenshots workflow (a local
      run on the maintainer's machine gave one-colour pictures for every submenu shot), and update
      its alt text in `shots.toml` and in `docs/getting-started.md` with it
- [ ] ATTRIBUTION, ROADMAP, CHANGELOG updated
