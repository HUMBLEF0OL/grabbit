# 00 · How every phase is built and shipped

Every plan in this folder follows these rules. They come from `CLAUDE.md`, `CONTRIBUTING.md` and
the decisions in `../PLAN.md`; when those files change, they win.

## Branch and workspace

- One phase = one branch from `dev` = one PR into `dev`. Name: `ui/<nn>-<slug>` (for example `ui/02-theme`).
- Work in a git worktree, not the main checkout: `git worktree add ../grabbit-ui-<nn> -b ui/<nn>-<slug> dev`.
  Other sessions use the main checkout.
- Rebase on `dev` before opening the PR and again before merging. `dev` is English only and Windows
  only; the multi-language, multi-platform state is kept on `archive/multiplatform-i18n`.
- A phase that grows past about 1,500 changed lines splits into numbered parts (`ui/05a-…`, `ui/05b-…`).

## The spec

- The mockup `../grabbit-design.html` is the reference. Each plan names the mock states it builds;
  build each one, then compare it side by side with the mock state at 100% interface size.
- Wording in windows stays the app's own (`crate::i18n::t` etc.). The mock shows the app's real
  strings; where the mock and the code disagree, the code's wording wins unless the plan says otherwise.
- Behaviour never changes in a look phase: same commands, order, shortcuts, defaults, enabled rules.

## Checks before every PR

1. `cargo fmt --all`
2. `cargo clippy --all-targets` (no new warnings)
3. `cargo test` (all green; new tests listed in the plan)
4. `cargo build --release` and run the app: `cargo run -p mdm-app`
5. Keyboard pass: Tab, arrows, Alt and menu letters, Enter and Escape in every changed window
6. Screen-reader pass where the plan touches layout (the accessibility tree in `winui.rs` must match what is painted)
7. Interface size 100%, 150%, 200% (View, Interface size) for every changed window

## Screenshots (rule 6 of CLAUDE.md)

- Each changed window and state, before and after, light and dark.
- Stored on the `pr-shots` branch as `<pr>/<name>.png`, embedded from
  `raw.githubusercontent.com/anefzaoui/misterdownloadmanager/pr-shots/`, never merged.
- Neutral test folders and files (as in the mockup: `D:\Downloads`, example.org addresses), no IDM artwork.
- Before-shots come from the baseline set made in phase 01, so every PR compares against the same starting point.
  Take after-shots with the same scripts at the same Windows display scale (the baseline's `INDEX.md`
  says which: 200 %), or the pairs don't match pixel for pixel.

## Paperwork in every PR

- `ROADMAP.md`: move the phase's item to done, or add it (rule 5). Public file: no private notes.
- `ATTRIBUTION.md`: any new third-party asset (Tabler Icons MIT, Geist OFL) with its licence.
- `CHANGELOG.md`: one user-facing line when the phase changes what users see.
- PR description: what changed, the screenshots, the checks run, what was left for a later phase.

## Clean room (rule 1)

Icons are Tabler (MIT), type is Geist (SIL OFL), colours and layout are our own design. Nothing from
IDM: no artwork, icons, sounds or text. Behaviour stays what is learned from public material.

## Definition of done (every phase)

- [ ] Every mock state the plan lists matches at 100%, light and dark
- [ ] All checks above pass
- [ ] Screenshots in the PR
- [ ] ROADMAP, ATTRIBUTION and CHANGELOG updated where they apply
- [ ] The next phase's prerequisites (listed in its plan) are met
