# 01 · Prepare: decisions, assets and the baseline

Nothing in the app changes in this phase. It turns the mockup into the files the build phases
consume, and records the decisions they depend on.

- Branch: `ui/01-prepare` (only adds files under `design/graphite/`; may be merged as docs)
- Needs: nothing
- Unblocks: every later phase

## 1. Decisions to record (in `../PLAN.md`, Goal and decisions table)

| Decision | Options | Recommendation | Blocks |
| --- | --- | --- | --- |
| Colour direction | Graphite green; Obsidian, ice, racing red; Jet black, royal amethyst, electric cyan; Navy, seal blue, crimson pink | Jet black, royal amethyst, electric cyan | 02 |
| Default light or dark for the new theme | Follow the system (today's behaviour) or dark first | Follow the system; the palette is designed dark-first | 02 |
| Is the new look the default theme? | Opt-in under View, Theme; or default for new installs | Opt-in in 02 to 08; default after 08 ships | 02, 09 |
| Typeface | Geist; keep the system font | Geist | 04 |
| Paste bar and details pane default | On or off for existing users | Paste bar on, details pane off | 06 |
| The two proposal dialogs | Drop; keep as planned features | Drop from the build; revisit with the proxy work | 07 |
| The rename to Grabbit | Now (before 02) or last (10) | Last: it touches data paths and the extension | 10 |

## 2. Export the icon set

Output: `design/graphite/assets/icons/{16,24}/<need>.svg` for all 53 needs listed in
`crates/app/src/themeicons.rs` `NEEDS`.

1. Unpack `vendor/tabler-icons-3.31.0.tgz`; source = `package/icons/outline/<name>.svg`.
2. Take the names from `source/iconmap.py` (first column) with the review changes in
   `source/gen3.py` `REFINE` applied (stop-all, start/stop queue, scheduler, delete-completed,
   restore, queued, to-tray).
3. For each file: drop the empty `<path stroke="none" d="M0 0h24v24H0z" fill="none"/>`, set
   `stroke="#ffffff"` (the app tints white), set `stroke-width="1.75"`, keep `viewBox="0 0 24 24"`,
   write `width`/`height` 16 and 24.
4. Check: 106 files; every need present at both sizes; each opens in a browser.
5. Write `assets/icons/SOURCES.txt` (need → Tabler name, version 3.31.0, commit or npm tarball hash)
   and copy `vendor/TABLER-LICENSE` to `assets/icons/LICENSE`.

Keep it a script (`source/export_icons.py`) so the set can be rebuilt after any mapping change.

## 3. Map the palette to the app's theme keys

Output: `design/graphite/assets/graphite.toml`, a draft theme file for the chosen palette.

1. Take the light and dark tokens from `source/pal.py` for the chosen palette.
2. Map each mock token to the app's theme keys (the full key list is in plan 02):
   - `bg` → `seeds.background`; `surface` → `seeds.surface` and `seeds.header`; `ink` → `seeds.text`
   - `acc` → `seeds.accent`; `sel` → `seeds.selection`; `err` → `seeds.error`; `okt` → `seeds.success`
   - `line` → `button.border`, `field.border`; `track` → `progress.track`, `meter.track`
   - `out` → `progress.bar`, `meter.fill`, `button.accent`, `button.default.border`; `onout` → `button.accent.text`; `out2` → `button.accent.hot`
   - `dis` → `text.disabled`; title bar roles (`caption`, `caption.text`, `caption.text.inactive`) from `surface`, `ink`, `ink3`
   - The full role list and the rest of the mapping are in plan 02, step 3
3. Re-run the contrast check on the mapped values (`python source/pal.py`).

## 4. Fonts

Output: `design/graphite/assets/fonts/` with Geist Regular, Medium and SemiBold (TTF or OTF) and
`OFL.txt`, from the official Vercel Geist release; record the version and source URL in `SOURCES.txt`.
Note each file's size for the installer budget.

## 5. Design spec

Output: `design/graphite/DESIGN.md`, the rules the build phases follow (taken from the mockup):

- Corner radii 6 (items, menu rows), 8 (buttons, fields, boxes), 10 (windows)
- Heights: buttons and fields 30, list rows 32, toolbar buttons 62 wide, title bar 36
- Label column 112 px, 240 px for sentence-length labels; 12 px between label and field
- Default button last in every footer, filled with the outlaw colour
- Greyed: one colour (`dis`) for text, icons and checkbox borders; no opacity tricks
- Status always icon + word; colour never alone
- Icon size 20 in the toolbar, 16 in lists and menus, stroke 1.75
- Type sizes 11.5 (toolbar labels, hints), 12 (column heads), 13 (body), 15 to 16 (dialog headings)

## 6. Baseline screenshots

Output: `design/graphite/baseline/` (not committed to `dev`; pushed to `pr-shots` as `baseline/`).
Every window and state the mockup shows, from today's app, in the Classic theme, light and dark, at
100%. Name each file after its mock state (`main-details.png`, `progress-status.png`, …) so PRs pair
them with the after-shots.

## Done when

- [ ] Every decision in section 1 recorded
- [ ] 106 icon SVGs, SOURCES.txt, LICENSE
- [ ] Draft `graphite.toml` whose contrast check passes
- [ ] Geist files and OFL.txt
- [ ] DESIGN.md
- [ ] Baseline screenshots
