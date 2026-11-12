#!/usr/bin/env python3
"""Bring the original MDM's fixes into this fork, one upstream commit at a time, one way only.

The original (anefzaoui/misterdownloadmanager) is the `upstream` remote: fetched, never pushed to
(its push address is DISABLED). Its fixes reach this fork's `dev` only through a pull request on
this fork. Nothing here merges upstream wholesale: the fork has removed parts of it (the other
platforms, the other languages) and rewrites others (Graphite), so each commit is sorted:

  removed  every file it touches is gone here (or lies in a folder that is): nothing to take
  clean    applies as it is (after the fork's own files below are left out)
  prune    applies once the removed files and the fork's own files are left out
  port     conflicts in code the fork kept and changed: read the fix and write it again by hand

The fork's own files (OWN) always keep the fork's version: upstream's lines in them are left out
and the sync pull request writes the fork's own entries. A sorted commit is a starting point, not
a verdict: a "clean" one can still break the fork's logic, which is what the build and tests and
the review are for (.claude/skills/upstream-sync/SKILL.md).

What was taken is read from this fork's history: a pick's `(cherry picked from commit <sha>)`
line and a port's `Upstream-Port: <sha>` trailer. What was decided without a commit (skipped,
covered, deferred) is in LEDGER.

    python tools/upstream-sync/sync.py doctor [--fix]   # the remotes and gh, checked (and set)
    python tools/upstream-sync/sync.py status [--fetch] [--json] [--all] [--onto REF]
    python tools/upstream-sync/sync.py show <sha>       # one commit: why it sorts as it does
    python tools/upstream-sync/sync.py pick <sha>...    # onto the current sync branch
    python tools/upstream-sync/sync.py record <sha> <skipped|covered|deferred|ported> --note "…"
    python tools/upstream-sync/sync.py pr --title "…" --body-file F   # push, PR to the fork's dev

Python 3.10 or later; git 2.40 or later (merge-tree --merge-base).
"""
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
from dataclasses import asdict, dataclass, field
from pathlib import Path

UPSTREAM_REMOTE = "upstream"
UPSTREAM_SLUG = "anefzaoui/misterdownloadmanager"
UPSTREAM_BRANCH = "main"
FORK_REMOTE = "origin"
FORK_BASE = "dev"
PROTECTED = {"dev", "main"}
SYNC_PREFIX = "upstream-sync/"
LEDGER = "tools/upstream-sync/ledger.md"
# The fork's own record of what changed and what's planned: upstream's lines never land here.
OWN = ("CHANGELOG.md", "ROADMAP.md")
STATUSES = ("skipped", "covered", "deferred", "ported")
# A hint for the reviewer, never a decision: words upstream's subjects use for fixes that matter.
CRITICAL = re.compile(
    r"secur|vulnerab|cve-|crash|panic|freez|hangs?|deadlock|corrupt|data loss|heap|"
    r"overflow|leak|injection|traversal|signature|integrity|checksum|one piece short|"
    r"can't stop|permissions",
    re.I,
)
# Upstream still builds for these; this fork doesn't (a hint: such a commit is usually skipped).
OTHER_PLATFORMS = re.compile(r"^(linux|macos|mac|ubuntu|flatpak|safari)|^linux and macos", re.I)
PICKED = re.compile(r"\(cherry picked from commit ([0-9a-f]{40})\)")
PORTED = re.compile(r"^Upstream-Port:\s*([0-9a-f]{7,40})\s*$", re.M)
ROW = re.compile(r"^\|\s*([0-9a-f]{7,40})\s*\|\s*(\w+)\s*\|(.*)\|\s*$")


class Fail(Exception):
    """A stop with a reason for the person: printed, exit code 1."""


def git(*args: str, check: bool = True, env: dict | None = None) -> subprocess.CompletedProcess:
    r = subprocess.run(
        ["git", *args],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        env={**os.environ, **(env or {})},
    )
    if check and r.returncode != 0:
        raise Fail(f"git {' '.join(args)}: {r.stderr.strip() or r.stdout.strip()}")
    return r


def out(*args: str) -> str:
    return git(*args).stdout.strip()


def root() -> Path:
    return Path(out("rev-parse", "--show-toplevel"))


def slug(url: str) -> str:
    """owner/name of a GitHub remote address (https or ssh), lower case."""
    m = re.search(r"github\.com[:/]+([^/]+/[^/]+?)(?:\.git)?/?$", url.strip())
    return m.group(1).lower() if m else ""


def remote_url(name: str, push: bool = False) -> str:
    r = git("remote", "get-url", *(["--push"] if push else []), name, check=False)
    return r.stdout.strip() if r.returncode == 0 else ""


def upstream_ref() -> str:
    return f"{UPSTREAM_REMOTE}/{UPSTREAM_BRANCH}"


def fork_ref() -> str:
    return f"{FORK_REMOTE}/{FORK_BASE}"


def has_ref(ref: str) -> bool:
    return git("rev-parse", "--verify", "--quiet", f"{ref}^{{commit}}", check=False).returncode == 0


def current_branch() -> str:
    return git("symbolic-ref", "--short", "-q", "HEAD", check=False).stdout.strip()


# ---------------------------------------------------------------------------------------------
# What the fork removed, and what it has already taken


@dataclass
class Removed:
    files: set[str]
    dirs: set[str]

    def __contains__(self, path: str) -> bool:
        if path in self.files:
            return True
        parts = path.split("/")
        return any("/".join(parts[:i]) in self.dirs for i in range(1, len(parts)))


def removed(base: str, head: str) -> Removed:
    """Files the fork deleted since the fork point (a rename is not a deletion: git follows it),
    and folders that existed then and are gone now (a file upstream adds in one is removed too)."""
    files = set(out("diff", "-M", "--diff-filter=D", "--name-only", base, head).splitlines())
    then = set(out("ls-tree", "-r", "-d", "--name-only", base).splitlines())
    now = set(out("ls-tree", "-r", "-d", "--name-only", head).splitlines())
    return Removed(files, then - now)


def fork_changed(base: str, head: str) -> set[str]:
    return set(out("diff", "-M", "--name-only", base, head).splitlines())


def read_ledger(path: Path) -> dict[str, tuple[str, str]]:
    """sha (as written) -> (status, rest of the row)."""
    rows = {}
    if path.exists():
        for line in path.read_text(encoding="utf-8").splitlines():
            m = ROW.match(line)
            if m:
                rows[m.group(1)] = (m.group(2), m.group(3).strip())
    return rows


def full_sha(sha: str) -> str | None:
    r = git("rev-parse", "--verify", "--quiet", f"{sha}^{{commit}}", check=False)
    return r.stdout.strip() if r.returncode == 0 else None


@dataclass
class Handled:
    picked: set[str] = field(default_factory=set)
    ported: set[str] = field(default_factory=set)
    equivalent: set[str] = field(default_factory=set)
    ledger: dict[str, str] = field(default_factory=dict)
    unknown: list[str] = field(default_factory=list)  # ledger rows upstream no longer has

    def status(self, sha: str) -> str | None:
        if sha in self.picked:
            return "picked"
        if sha in self.ported or any(sha.startswith(p) for p in self.ported if len(p) < 40):
            return "ported"
        if sha in self.ledger:
            return self.ledger[sha]
        if sha in self.equivalent:
            return "equivalent"
        return None


def handled(refs: list[str], fork_point: str, up: str, ledger_path: Path) -> Handled:
    h = Handled()
    refs = [r for r in dict.fromkeys(refs) if has_ref(r)]
    log = out("log", "--format=%B%x00", f"^{fork_point}", *refs) if refs else ""
    h.picked = set(PICKED.findall(log))
    h.ported = set(PORTED.findall(log))
    for ref in refs:  # same change already here, by patch-id (a pick made without -x, a rebase)
        for line in out("cherry", ref, up, fork_point).splitlines():
            if line.startswith("- "):
                h.equivalent.add(line[2:].strip())
    for sha, (status, _) in read_ledger(ledger_path).items():
        full = full_sha(sha)
        if full:
            h.ledger[full] = status
        else:
            h.unknown.append(sha)
    return h


# ---------------------------------------------------------------------------------------------
# Sorting one upstream commit


@dataclass
class Commit:
    sha: str
    date: str
    subject: str
    kind: str = ""
    critical: bool = False
    touches_fork: list[str] = field(default_factory=list)
    left_out: list[str] = field(default_factory=list)
    conflicts: list[str] = field(default_factory=list)
    after: list[str] = field(default_factory=list)  # earlier upstream commits not taken, same files
    notes: list[str] = field(default_factory=list)
    status: str | None = None


def touched(sha: str) -> list[str]:
    """Paths the commit changes, both sides of a rename."""
    paths = []
    for line in out("diff-tree", "-r", "-M", "--no-commit-id", "--name-status", sha).splitlines():
        paths += line.split("\t")[1:]
    return list(dict.fromkeys(paths))


def simulate(sha: str, onto: str) -> tuple[bool, list[str]]:
    """Cherry-pick `sha` onto `onto` in memory (nothing on disk moves): clean?, conflicted paths."""
    r = git(
        "merge-tree", "--write-tree", "--name-only", "--no-messages",
        f"--merge-base={sha}^", onto, sha, check=False,
    )
    if r.returncode not in (0, 1):
        raise Fail(f"merge-tree on {sha[:12]}: {r.stderr.strip()}")
    lines = r.stdout.splitlines()
    conflicts = []
    for line in lines[1:]:
        if not line.strip():
            break
        conflicts.append(line.strip())
    return r.returncode == 0, list(dict.fromkeys(conflicts))


def not_taken_before(c: Commit, paths: list[str], fork_point: str, h: Handled) -> list[str]:
    """Earlier upstream commits, not taken here yet, that changed these paths: a conflict in them
    is often upstream's own earlier change missing here, not the fork's. Take or port those first."""
    if not paths:
        return []
    shas = out("rev-list", "--reverse", "--no-merges", f"{fork_point}..{c.sha}^", "--", *paths)
    return [s for s in shas.splitlines() if h.status(s) in (None, "deferred")]


def sort_commit(c: Commit, onto: str, gone: Removed, changed: set[str], fork_point: str,
                h: Handled) -> Commit:
    full = out("log", "-1", "--format=%B", c.sha)
    c.critical = bool(CRITICAL.search(full))
    paths = touched(c.sha)
    own = [p for p in paths if p in OWN]
    gone_here = [p for p in paths if p in gone]
    rest = [p for p in paths if p not in OWN and p not in gone]
    c.left_out = gone_here + own
    c.touches_fork = [p for p in rest if p in changed]
    if not rest:
        c.kind = "removed"
        return c
    if len(out("rev-list", "--parents", "-n", "1", c.sha).split()) > 2:
        c.notes.append("a merge commit: its own resolution is not carried")
    _, conflicts = simulate(c.sha, onto)
    c.conflicts = [p for p in conflicts if p not in OWN and p not in gone]
    if c.conflicts:
        c.kind = "port"
        c.after = not_taken_before(c, c.conflicts, fork_point, h)
        mine = [p for p in c.conflicts if p in changed]
        if c.after:
            c.notes.append("after " + ", ".join(s[:12] for s in c.after)
                           + ": upstream changed these files before; take those first, then sort again")
        if not mine:
            c.notes.append("the fork never changed the conflicting files: only upstream's earlier "
                           "commits are missing")
    elif gone_here or (set(conflicts) & set(own)):
        c.kind = "prune"
    else:
        c.kind = "clean"
    if "Cargo.lock" in c.conflicts:
        c.notes.append("Cargo.lock: keep the fork's, then `cargo update -p <crate>` for its bump")
    if any(p.endswith(".rs") for p in c.left_out if p in gone):
        c.notes.append("Rust code left out: check nothing kept here needed it")
    if OTHER_PLATFORMS.search(c.subject):
        c.notes.append("about another platform: likely skipped, unless it touches shared code")
    if c.touches_fork and c.kind != "port":
        c.notes.append("applies, but in files the fork changed: review the logic, not just the text")
    return c


def pending(fork_point: str, up: str) -> list[Commit]:
    commits = []
    fmt = "%H%x1f%ad%x1f%s"
    for line in out(
        "log", "--reverse", "--topo-order", "--no-merges", "--date=short", f"--format={fmt}",
        f"{fork_point}..{up}",
    ).splitlines():
        sha, date, subject = line.split("\x1f", 2)
        commits.append(Commit(sha, date, subject))
    return commits


# ---------------------------------------------------------------------------------------------
# Commands


def context(onto: str | None) -> tuple[str, str, str, Removed, set[str], Handled]:
    up, fork = upstream_ref(), fork_ref()
    for ref in (up, fork):
        if not has_ref(ref):
            raise Fail(f"{ref} is missing: run `doctor --fix`, then `status --fetch`")
    onto = onto or "HEAD"
    fork_point = git("merge-base", fork, up, check=False).stdout.strip()
    if not fork_point:
        raise Fail(f"{fork} and {up} share no history")
    gone = removed(fork_point, onto)
    changed = fork_changed(fork_point, onto)
    h = handled([fork, "HEAD", onto], fork_point, up, root() / LEDGER)
    return up, fork_point, onto, gone, changed, h


def cmd_status(a) -> int:
    if a.fetch:
        git("fetch", UPSTREAM_REMOTE, "--no-tags", "--prune")
        git("fetch", FORK_REMOTE, "--prune")
    up, fork_point, onto, gone, changed, h = context(a.onto)
    commits = pending(fork_point, up)
    for c in commits:
        c.status = h.status(c.sha)
    todo = [c for c in commits if a.all or c.status in (None, "deferred")]
    for c in todo:
        sort_commit(c, onto, gone, changed, fork_point, h)
    open_syncs = unmerged_sync_branches()
    if a.json:
        print(json.dumps({
            "upstream": up, "fork_point": fork_point, "onto": onto,
            "open_sync_branches": open_syncs, "ledger_unknown": h.unknown,
            "commits": [asdict(c) for c in todo],
        }, indent=1))
        return 0
    done = len(commits) - len([c for c in commits if c.status in (None, "deferred")])
    print(f"Upstream {up} since the fork point {fork_point[:12]}: {len(commits)} commits, "
          f"{done} handled, {len(commits) - done} to look at. Sorted against {onto}.\n")
    if open_syncs:
        print("! Sync branches on origin not yet in dev (their picks may show again here): "
              + ", ".join(open_syncs) + "\n")
    for sha in h.unknown:
        print(f"! Ledger row {sha}: upstream no longer has that commit (rewritten history?)")
    order = {"port": 0, "prune": 1, "clean": 2, "removed": 3}
    todo.sort(key=lambda c: (not c.critical, order.get(c.kind, 9)))
    print("| sha | date | kind | critical? | status | subject |")
    print("|---|---|---|---|---|---|")
    for c in todo:
        print(f"| {c.sha[:12]} | {c.date} | {c.kind} | {'yes' if c.critical else ''} | "
              f"{c.status or ''} | {c.subject.replace('|', '/')} |")
    for c in todo:
        if c.conflicts or c.notes:
            print(f"\n{c.sha[:12]} {c.kind}: " + "; ".join(
                ([f"conflicts in {', '.join(c.conflicts)}"] if c.conflicts else []) + c.notes))
    return 0


def cmd_show(a) -> int:
    up, fork_point, onto, gone, changed, h = context(a.onto)
    sha = full_sha(a.sha)
    if not sha:
        raise Fail(f"no commit {a.sha}")
    c = Commit(sha, out("log", "-1", "--date=short", "--format=%ad", sha),
               out("log", "-1", "--format=%s", sha))
    c.status = h.status(sha)
    sort_commit(c, onto, gone, changed, fork_point, h)
    print(json.dumps(asdict(c), indent=1))
    return 0


def tracked_dirty(ignore: set[str]) -> list[str]:
    paths = []
    for line in out("status", "--porcelain", "--untracked-files=no").splitlines():
        p = line[3:].split(" -> ")[-1].strip('"')
        if p not in ignore:
            paths.append(p)
    return paths


def git_dir_file(name: str) -> Path:
    return Path(out("rev-parse", "--git-path", name))


def undo_pick() -> None:
    if git_dir_file("CHERRY_PICK_HEAD").exists():
        git("cherry-pick", "--abort", check=False)
    git("reset", "--merge", "HEAD", check=False)


def pick_one(sha: str, gone: Removed, h: Handled) -> str:
    """One upstream commit onto HEAD, with the fork's rules. Returns what happened."""
    status = h.status(sha)
    if status and status != "deferred":
        return f"{sha[:12]} already {status}: not picked again"
    msg = out("log", "-1", "--format=%B", sha)
    author = out("log", "-1", "--format=%an <%ae>", sha)
    date = out("log", "-1", "--format=%aI", sha)
    git("cherry-pick", "--no-commit", sha, check=False)
    left_out = []
    staged = set(out("diff", "--cached", "--name-only", "HEAD").splitlines())
    conflicted = set(out("diff", "--name-only", "--diff-filter=U").splitlines())
    for path in sorted(staged | conflicted):
        if path in OWN:
            if git("cat-file", "-e", f"HEAD:{path}", check=False).returncode == 0:
                git("checkout", "HEAD", "--", path)
            else:
                git("rm", "-f", "--quiet", "--", path, check=False)
            left_out.append(path)
        elif path in gone:
            git("rm", "-f", "--quiet", "--", path, check=False)
            left_out.append(path)
    still = out("diff", "--name-only", "--diff-filter=U").splitlines()
    if still:
        undo_pick()
        return f"{sha[:12]} needs a port: conflicts in {', '.join(still)} (nothing changed)"
    if git("diff", "--cached", "--quiet", "HEAD", check=False).returncode == 0:
        undo_pick()
        return f"{sha[:12]} has nothing left once removed and own files are out: record it skipped"
    body = msg.rstrip() + f"\n\n(cherry picked from commit {sha})\n"
    if left_out:
        body += "\nLeft out here (removed in this fork, or the fork's own record):\n"
        body += "".join(f"- {p}\n" for p in left_out)
    with tempfile.NamedTemporaryFile("w", encoding="utf-8", delete=False, suffix=".txt") as f:
        f.write(body)
        msg_file = f.name
    try:
        r = git("commit", "--quiet", "-F", msg_file, f"--author={author}", f"--date={date}",
                check=False)
    finally:
        os.unlink(msg_file)
    if r.returncode != 0:
        undo_pick()
        return f"{sha[:12]} could not be committed ({r.stderr.strip()}): nothing changed"
    h.picked.add(sha)
    new = out("rev-parse", "--short=12", "HEAD")
    extra = f", left out {len(left_out)} file(s)" if left_out else ""
    return f"{sha[:12]} picked as {new}{extra}"


def guard_sync_branch() -> str:
    branch = current_branch()
    if not branch:
        raise Fail("HEAD is detached: work on a sync branch (upstream-sync/<date>)")
    if branch in PROTECTED:
        raise Fail(f"on {branch}: picks go on a sync branch, and reach {FORK_BASE} by a PR")
    return branch


def cmd_pick(a) -> int:
    guard_sync_branch()
    if git_dir_file("CHERRY_PICK_HEAD").exists() or git_dir_file("MERGE_HEAD").exists():
        raise Fail("a cherry-pick or merge is in progress: finish or abort it first")
    dirty = tracked_dirty({LEDGER})
    if dirty:
        raise Fail("uncommitted changes (commit or stash them first): " + ", ".join(dirty))
    up, fork_point, onto, gone, changed, h = context(None)
    in_range = set(out("rev-list", "--no-merges", f"{fork_point}..{up}").splitlines())
    shas = []
    for s in a.sha:
        full = full_sha(s)
        if not full:
            raise Fail(f"no commit {s}")
        if full not in in_range:
            raise Fail(f"{s} is not an upstream commit since the fork point")
        shas.append(full)
    # Upstream's order, whatever order they were given in: a fix can build on the one before.
    order = out("rev-list", "--reverse", "--topo-order", f"{fork_point}..{up}").splitlines()
    shas.sort(key=order.index)
    failed = 0
    for sha in shas:
        line = pick_one(sha, gone, h)
        print(line)
        if "picked as" not in line and "already" not in line:
            failed += 1
            if not a.keep_going:
                print("Stopped: later picks may build on this one (--keep-going to go on).")
                return 1
    return 1 if failed else 0


def cmd_record(a) -> int:
    sha = full_sha(a.sha)
    if not sha:
        raise Fail(f"no commit {a.sha}")
    if not a.note.strip():
        raise Fail("--note is required: say why, for whoever reads the ledger next")
    path = root() / LEDGER
    lines = path.read_text(encoding="utf-8").splitlines() if path.exists() else []
    subject = out("log", "-1", "--format=%s", sha).replace("|", "/")
    note = a.note.replace("|", "/").replace("\n", " ")
    row = f"| {sha[:12]} | {a.status} | {a.ref or ''} | {subject} | {note} |"
    for i, line in enumerate(lines):
        m = ROW.match(line)
        if m and full_sha(m.group(1)) == sha:
            lines[i] = row
            break
    else:
        lines.append(row)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
    print(row)
    return 0


def unmerged_sync_branches() -> list[str]:
    fork = fork_ref()
    if not has_ref(fork):
        return []
    names = out("for-each-ref", "--format=%(refname:short)",
                f"refs/remotes/{FORK_REMOTE}/{SYNC_PREFIX}").splitlines()
    return [n for n in names
            if git("merge-base", "--is-ancestor", n, fork, check=False).returncode != 0]


def fork_slug() -> str:
    s = slug(remote_url(FORK_REMOTE))
    if not s:
        raise Fail(f"{FORK_REMOTE} is not a GitHub address")
    if s == UPSTREAM_SLUG:
        raise Fail(f"{FORK_REMOTE} points at the original ({UPSTREAM_SLUG}): never push there")
    return s


def cmd_pr(a) -> int:
    branch = guard_sync_branch()
    target = fork_slug()
    if tracked_dirty(set()):
        raise Fail("uncommitted changes: commit them (the ledger too) before the PR")
    git("push", "-u", FORK_REMOTE, f"HEAD:refs/heads/{branch}")
    # -R and --base always: in a fork, gh's default for a new PR is the parent repository.
    args = ["gh", "pr", "create", "-R", target, "--base", FORK_BASE, "--head", branch,
            "--title", a.title, "--body-file", a.body_file]
    if a.draft:
        args.append("--draft")
    r = subprocess.run(args, text=True, encoding="utf-8")
    return r.returncode


def checks() -> list[tuple[bool, str, list[str] | None]]:
    """(ok, what, the git config command that fixes it, if one does)."""
    c = []
    up_url = remote_url(UPSTREAM_REMOTE)
    want = f"https://github.com/{UPSTREAM_SLUG}.git"
    c.append((slug(up_url) == UPSTREAM_SLUG, f"{UPSTREAM_REMOTE} fetches {UPSTREAM_SLUG}",
              ["remote", "add" if not up_url else "set-url", UPSTREAM_REMOTE, want]))
    push = remote_url(UPSTREAM_REMOTE, push=True)
    c.append((bool(up_url) and push == "DISABLED", f"{UPSTREAM_REMOTE} cannot be pushed to",
              ["remote", "set-url", "--push", UPSTREAM_REMOTE, "DISABLED"]))
    tag = git("config", "--get", f"remote.{UPSTREAM_REMOTE}.tagOpt", check=False).stdout.strip()
    c.append((tag == "--no-tags", f"{UPSTREAM_REMOTE}'s tags stay out (they'd clash with ours)",
              ["config", f"remote.{UPSTREAM_REMOTE}.tagOpt", "--no-tags"]))
    origin = slug(remote_url(FORK_REMOTE))
    c.append((bool(origin) and origin != UPSTREAM_SLUG,
              f"{FORK_REMOTE} is this fork ({origin or 'missing'}), not the original", None))
    gh = git("config", "--get", f"remote.{FORK_REMOTE}.gh-resolved", check=False).stdout.strip()
    c.append((gh == "base", f"gh's default repository is {FORK_REMOTE} (not the parent)",
              ["config", f"remote.{FORK_REMOTE}.gh-resolved", "base"]))
    pd = git("config", "--get", "remote.pushDefault", check=False).stdout.strip()
    c.append((pd != UPSTREAM_REMOTE, f"remote.pushDefault is not {UPSTREAM_REMOTE}",
              ["config", "--unset", "remote.pushDefault"]))
    v = re.search(r"(\d+)\.(\d+)", out("--version"))
    c.append((bool(v) and (int(v[1]), int(v[2])) >= (2, 40), "git 2.40 or later", None))
    c.append((has_ref(upstream_ref()), f"{upstream_ref()} fetched",
              ["fetch", UPSTREAM_REMOTE, "--no-tags"]))
    return c


def cmd_doctor(a) -> int:
    bad = 0
    for ok, what, fix in checks():
        if not ok and a.fix and fix:
            git(*fix)
            ok = True
            what += " (fixed)"
        print(("ok    " if ok else "FAIL  ") + what)
        bad += not ok
    for n in unmerged_sync_branches():
        print(f"note  {n} is not in {fork_ref()} yet: one sync at a time, merge or close it first")
    return 1 if bad else 0


def main(argv: list[str] | None = None) -> int:
    for stream in (sys.stdout, sys.stderr):  # the Windows console's code page can't print every
        stream.reconfigure(encoding="utf-8", errors="replace")  # upstream subject
    p = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = p.add_subparsers(dest="cmd", required=True)
    d = sub.add_parser("doctor")
    d.add_argument("--fix", action="store_true")
    s = sub.add_parser("status")
    s.add_argument("--fetch", action="store_true")
    s.add_argument("--json", action="store_true")
    s.add_argument("--all", action="store_true", help="the handled commits too")
    s.add_argument("--onto", help="sort against this ref (default HEAD)")
    sh = sub.add_parser("show")
    sh.add_argument("sha")
    sh.add_argument("--onto")
    k = sub.add_parser("pick")
    k.add_argument("sha", nargs="+")
    k.add_argument("--keep-going", action="store_true")
    r = sub.add_parser("record")
    r.add_argument("sha")
    r.add_argument("status", choices=STATUSES)
    r.add_argument("--note", required=True)
    r.add_argument("--ref", help="the fork's commit or PR, for ported and covered")
    pr = sub.add_parser("pr")
    pr.add_argument("--title", required=True)
    pr.add_argument("--body-file", required=True)
    pr.add_argument("--draft", action="store_true")
    a = p.parse_args(argv)
    try:
        return {"doctor": cmd_doctor, "status": cmd_status, "show": cmd_show, "pick": cmd_pick,
                "record": cmd_record, "pr": cmd_pr}[a.cmd](a)
    except Fail as e:
        print(f"error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
