---
name: upstream-sync
description: "Bring the original MDM's fixes (anefzaoui/misterdownloadmanager, the `upstream` remote) into this fork: sort upstream's new commits, pick the ones that apply, port the critical ones that touch the fork's own code, record the rest, and open a PR to the fork's dev. Use when the user says /upstream-sync, \"bring upstream's fixes\", \"sync with the original\", \"what did upstream fix\", or asks whether an upstream fix is in the fork."
user-invocable: true
---

# Upstream sync

This fork is its own project. The original (anefzaoui/misterdownloadmanager) is a source of
fixes, read from and never written to. Fixes flow one way: upstream → a sync branch → a PR on
this fork (HUMBLEF0OL/grabbit) → its `dev`. The fork has removed the other platforms and the
other languages and rewrites the interface (Graphite), so upstream is never merged wholesale:
each upstream commit is sorted and decided on its own.

The tool is `tools/upstream-sync/sync.py` (its docstring has every command); the decisions made
without a commit are in `tools/upstream-sync/ledger.md`.

## Never

- Push to, open a PR, issue or comment on, or otherwise write to anefzaoui/misterdownloadmanager.
  Reading it (`git fetch upstream`, `gh pr view … -R anefzaoui/misterdownloadmanager`,
  `gh issue view …`) is fine. `gh pr create` always goes through `sync.py pr`, which names this
  fork and its `dev`: in a fork, gh's default for a new PR is the parent.
- Commit to `dev` or `main`, merge a sync PR, or force-push anything but your own sync branch.
  The person merges.
- Work in the main checkout or in another session's worktree: other sessions work there. Every
  run gets its own worktree.
- Copy upstream's CHANGELOG.md or ROADMAP.md lines: the fork writes its own (below).
- Take code for Linux or macOS, or for other languages, back in: `#[cfg(target_os = "linux")]` /
  `"macos"` branches, translation catalogs, the language picker, right-to-left layout.

## A run

1. **Check and fetch.** In any checkout of this repository:
   `python tools/upstream-sync/sync.py doctor` (on a fresh clone: `doctor --fix`; it adds the
   `upstream` remote with pushing disabled and its tags left out, and makes gh default to this
   fork). A `note` about an unmerged `upstream-sync/…` branch means a previous run's PR is still
   open: finish that one first, or its picks show again as to do.

2. **A worktree for the run**, from the fork's latest `dev`:
   ```
   git fetch origin
   git worktree add -b upstream-sync/<YYYY-MM-DD> "../grabbit-sync-<YYYY-MM-DD>" origin/dev
   ```
   Builds there: `CARGO_TARGET_DIR` = the main checkout's `target` (no dependency rebuild), and
   `-j 2` on this machine.

3. **Sort.** `python tools/upstream-sync/sync.py status --fetch`. Critical-looking ones first,
   then by kind. The "critical?" column is a word match on the commit message, a hint only: read
   each commit's subject and decide. Critical means: security; downloads lost, corrupted, cut
   short or not resumed; crashes, freezes and hangs; the app not starting. Those are taken (picked
   or ported) even when they touch the fork's code. Everything else may wait, or be skipped.

4. **Decide each commit**, in upstream's order (`status` prints it), by kind:

   | kind | what to do |
   |---|---|
   | `removed` | `record <sha> skipped --note "<what it was for>"` |
   | `clean`, `prune` | read it (`git show <sha>`). A `prune` lists what is left out: if Rust code is among it, check nothing the fork keeps needed it. A subject about Linux or macOS whose code is all `cfg`'d to them: skip it. Otherwise `pick` it. |
   | `port` | first its `after` list: earlier upstream commits not taken here that changed the same files. Decide those first (a pick there often turns this one `clean`: `status` again). What still conflicts is ported by hand (below), or skipped if not wanted. |
   | already fixed here another way | `record <sha> covered --ref <fork commit or PR> --note "…"` |
   | not now | `record <sha> deferred --note "<what it waits for>"`: it shows again next run |

   `pick` takes several shas and applies them in upstream's order; it stops at the first one
   that won't apply and leaves the branch as it was before that one. It never picks a commit
   twice: a pick's `(cherry picked from commit …)` line in this fork's history is its record.

5. **Port by hand** (a fix the fork needs, in code the fork changed):
   - Read the fix: `git show <sha>`, and its PR (`gh pr view <n> -R anefzaoui/misterdownloadmanager`)
     for the bug report. Write down in one or two sentences what the bug was and what the fix
     changes.
   - Check the fork has the bug: Graphite or an earlier fix of the fork's may have removed it
     (then `record … covered`).
   - Write the same fix in the fork's code, in the fork's style and files, keeping its rules
     (CLAUDE.md: `crate::i18n` for every text the user sees, Windows only, Windows calls in
     `win32.rs`, clean room). Bring upstream's test along when it has one, adapted; otherwise add
     one that fails without the fix.
   - Commit with one trailer per upstream commit it carries:
     `Upstream-Port: <full sha>`. That trailer is the record; `record <sha> ported --ref <PR>`
     adds the PR and a note to the ledger.
   - A port that changes behaviour goes in its own PR (its own branch,
     `upstream-sync/<date>-<issue>`), so it can be reviewed, tested and refused on its own.

6. **The fork's own record.** Upstream's CHANGELOG.md and ROADMAP.md lines were left out; for
   each user-visible fix taken, write the fork's: a CHANGELOG.md line in the fork's next-version
   section, in the fork's words, naming upstream's issue; and ROADMAP.md (CLAUDE.md rule 5: the
   item done, statuses true).

7. **Verify**, in the run's worktree, everything CI runs:
   ```
   cargo fmt --all --check
   cargo clippy --workspace --all-targets -j 2 -- -D warnings
   cargo test --workspace -j 2
   node tools/extension/test.mjs        # when the extension changed
   python tools/upstream-sync/test_sync.py   # when the tool changed
   ```
   A failure is found and fixed before the PR, never shipped as is. When one pick causes it and
   the fix isn't obvious: make the branch again from `origin/dev` without it (the picks are
   repeatable) and `record <sha> deferred --note "breaks <test>: <why>"`. Tests that fail only in
   workspace runs, with paths into a removed worktree, are stale test binaries in the shared
   target: touch that crate's `src/lib.rs` and run again.

8. **The PR.** Commit the ledger and the fork's CHANGELOG.md/ROADMAP.md lines, then
   `python tools/upstream-sync/sync.py pr --title "Upstream fixes, <date>" --body-file <file>`.
   The body (CLAUDE.md rule 6) has: a table of every upstream commit looked at (sha, subject,
   picked / ported / skipped / covered / deferred, and why); the verify step's output as the
   proof; and screenshots, before and after, light and dark, for any fix that changes what a
   window shows (on this fork's `pr-shots` branch, `<pr>/<name>.png`, embedded from
   `raw.githubusercontent.com/HUMBLEF0OL/grabbit/pr-shots/`). Batches of at most about 20
   picks per PR, so each review stays readable.

   Ask that it be merged with **Create a merge commit** or **Rebase and merge**, not
   **Squash and merge**: the picks' `(cherry picked from commit …)` lines and the `Upstream-Port`
   trailers must reach `dev` as written, or the next run offers those commits again.

9. **After the merge**, remove the run's worktree (`git worktree remove …`) and its local branch.

## When something looks wrong

- `status` shows a commit you know was taken: its record didn't reach `dev` (a squash merge
  that dropped the line, a pick made by hand without `-x`). Record it `covered`, with the
  fork's commit as its ref.
- `! Ledger row … upstream no longer has that commit`: upstream rewrote its history. Find the
  commit's new sha by its subject (`git log upstream/main --grep=…`) and record it again.
- Upstream reverts a commit: the revert is a commit like any other. Taken the original? Take
  the revert. Skipped it? Skip the revert.
- A conflict in `Cargo.lock` alone: keep the fork's, then `cargo update -p <crate> --precise
  <version>` for the bump the commit made, and commit that as the port.
- Upstream changes its main branch's name or the repository moves: update `UPSTREAM_BRANCH` /
  `UPSTREAM_SLUG` in `sync.py` and run `doctor --fix`.
