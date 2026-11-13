# 00 · How every phase is built and shipped

Every plan in this folder follows these rules. They come from `CLAUDE.md`, `CONTRIBUTING.md` and
the decisions in `../PLAN.md`; when those files change, they win, with one standing exception: the
push target below, which is the maintainer's rule and overrides the repository named in CLAUDE.md.

## Where work goes

- Push branches, open PRs and push `pr-shots` only to `origin` = `github.com/HUMBLEF0OL/grabbit`.
  Never push to or open PRs on `anefzaoui/misterdownloadmanager`, though CLAUDE.md rule 6 names it.

## Branch and workspace

- One phase = one branch from `dev` = one PR into `dev` on origin. Name: `ui/<nn>-<slug>` (for example
  `ui/02-theme`); the rename uses `rename/10<x>-<slug>`.
- Work in a git worktree, not the main checkout: `git worktree add ../grabbit-ui-<nn> -b ui/<nn>-<slug> dev`.
  Other sessions use the main checkout.
- Rebase on `dev` before opening the PR and again before merging. `dev` is English only and Windows
  only; the multi-language, multi-platform state is kept on `archive/multiplatform-i18n`. No plan step
  may touch Linux or macOS code: it no longer exists.
- A phase that grows past about 1,500 changed lines splits into numbered parts (`ui/05a-…`, `ui/05b-…`).

## The spec

- The mockup `../grabbit-design.html` is the reference. Each plan names the mock states it builds;
  build each one, then compare it side by side with the mock state at 100% interface size.
- Wording in windows stays the app's own (`crate::i18n::t` etc.). The mock shows the app's real
  strings; where the mock and the code disagree, the code's wording wins unless the plan says otherwise.
- Behaviour never changes in a look phase: same commands, order, shortcuts, keys (Enter, Escape),
  defaults and enabled rules. Accepted exceptions, each named by the plan that adds it:
  - a **new route to an existing command** that the mock shows (the title-bar Find button, the
    toolbar's Details toggle, the paste bar);
  - **keys a new layout needs**, when every old key keeps working (Up/Down in the Options side list;
    the paste bar's Tab stop, Escape and Ctrl+V);
  - **accessibility additions** in every theme (exposing a control that was not exposed, such as the
    toolbar's buttons): invisible, and the tree only gains nodes.
- **Graphite is the only layout** (since 05.5c). A look phase changes Graphite's windows
  directly: there is no Classic side to gate on or to keep pixel-identical.

## Checks before every PR (the same commands CI runs)

1. `cargo fmt --all --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace` (all green; new tests listed in the plan)
4. `node tools/extension/test.mjs` (CI runs it; required when the extension or host is touched)
5. `cargo build --release` and run the app: `cargo run -p mdm-app`
6. Keyboard pass: Tab, arrows, Alt and menu letters, Enter and Escape in every changed window
7. Screen-reader pass where the plan touches layout (the accessibility tree in `winui.rs` must match what is painted)
8. Interface size 100%, 150%, 200% (View, Interface size) for every changed window

## Screenshots (rule 6 of CLAUDE.md, to the fork)

- Each changed window and state, before and after, light and dark.
- Stored on the `pr-shots` branch of origin as `<pr>/<name>.png`, embedded from
  `raw.githubusercontent.com/HUMBLEF0OL/grabbit/pr-shots/<pr>/<name>.png`, never merged.
- Neutral test folders and files (as in the mockup: `D:\Downloads`, example.org addresses), no IDM artwork.
- **Before** = `dev` at the branch point, in the same theme as the after-shot, taken with the same
  script. Graphite light and dark only: no Classic pairs.
- Take every shot at Windows display scale 200% and View, Interface size 100% (the baseline's `INDEX.md`
  records this), or the pairs don't match pixel for pixel.
- On the maintainer's PC offscreen captures come out black: shoot on screen (`docs-shots.ps1 -At
  "40,40"`), with the display on and unlocked, the pointer parked off the windows, nobody using the
  PC, and a neutral `-Work` folder (KNOWN-GAPS G1, closed).

## Paperwork in every PR

- `ROADMAP.md` (rule 5): ROADMAP has one item for this work, 🔄 "A new look, 'Graphite'". Each phase
  adds what it delivered to that item's "Done:" clause; 09b marks it ✅. The rename (10) adds its own item.
- `ATTRIBUTION.md`: any new third-party asset (Tabler Icons MIT, Geist OFL) with its licence.
- `CHANGELOG.md`: one user-facing line when the phase changes what users see; refactors with nothing
  to see (05a, 07a) say "no CHANGELOG line" in the PR.
- `../PLAN.md` Progress: tick the phase in its own PR; it counts as done once that PR is merged into
  `dev` on origin.
- PR description: what changed, the screenshots, the checks run, what was left for a later phase
  (and which plan now carries it, as a "Left from NN" note in that plan).

## Clean room (rule 1)

Icons are Tabler (MIT), type is Geist (SIL OFL), colours and layout are our own design. Nothing from
IDM: no artwork, icons, sounds or text. Behaviour stays what is learned from public material.

## Definition of done (every phase)

- [ ] Every mock state the plan lists matches at 100%, light and dark
- [ ] All checks above pass
- [ ] Screenshots in the PR, from the fork
- [ ] ROADMAP, ATTRIBUTION and CHANGELOG updated where they apply; PLAN.md Progress ticked
- [ ] The next phase's prerequisites (listed in its plan) are met
