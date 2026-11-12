#!/usr/bin/env python3
"""Bring the original MDM's fixes into this fork, one upstream commit at a time, one way only.

The original (anefzaoui/misterdownloadmanager) is the `upstream` remote: fetched, never pushed to
(its push address is DISABLED). Its fixes reach this fork's `dev` only through a pull request on
this fork. Nothing here merges upstream wholesale: the fork has removed parts of it (the other
platforms, the other languages) and rewrites others (Graphite), so each commit is sorted:

  nothing  it only touches the fork's own record (OWN): nothing to take
  removed  every other file it touches is gone here (or lies in a folder that is): nothing to take
  clean    applies as it is (the fork's own record left out)
  prune    applies once the removed files and the fork's own record are left out
  lock     applies but for Cargo.lock: `pick --lock` keeps the fork's and makes upstream's bumps
  port     conflicts in code the fork kept: read the fix and write it again by hand
  merge    an upstream merge commit whose result differs from merging its parents: read it

The kind says whether the text applies, never whether the fix is right here: a `clean` commit can
still break the fork's logic, which is what the flags, the build and the review are for
(.claude/skills/upstream-sync/SKILL.md). Flags: C critical-looking, S sensitive (runs code or
instructs agents: never batch-picked), A earlier upstream commits not taken here changed the same
files or crates, I upstream's identity in added lines, P another platform, a language or text
direction in added lines.

The record of what was decided is LEDGER (one row per upstream commit, written by `pick` and
`record`), backed by the trailers `pick` and a port leave in this fork's history
(`Upstream-Pick: <sha>`, `Upstream-Port: <sha>`): a revert in the fork (of a pick, a merged sync
PR, or a revert) and a port PR closed unmerged show the fix as not taken. Sync PRs are merged with
a merge commit or a rebase, never squashed: a squash drops the trailers, so a squashed port stays
`ported?` and a revert of a squashed PR goes unseen. `equivalent`: the same change is already in
the fork (by patch-id), made without the tool.

    python tools/upstream-sync/sync.py doctor [--fix]   # remotes, gh, branches: checked (and set)
    python tools/upstream-sync/sync.py status [--fetch] [--json] [--all]
    python tools/upstream-sync/sync.py show <sha>       # one commit: why it sorts as it does
    python tools/upstream-sync/sync.py watch [--json]   # upstream's open PRs and unmerged branches
    python tools/upstream-sync/sync.py report           # status + watch as Markdown (CI's issue)
    python tools/upstream-sync/sync.py pick <sha>... [--lock] [--allow-sensitive] [--keep-going]
    python tools/upstream-sync/sync.py record <sha> <skipped|covered|deferred|ported> --note "…"
    python tools/upstream-sync/sync.py pr --title "…" --body-file F [--draft] [--force]

Python 3.10 or later; a git whose merge-tree takes --merge-base and -X (`doctor` checks); gh for
`watch`, `pr` and the PR states (UPSTREAM_SYNC_OFFLINE=1: never call gh, as the self-check does).
"""
from __future__ import annotations

import argparse
import codecs
import datetime as dt
import json
import os
import re
import shutil
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from dataclasses import asdict, dataclass, field
from pathlib import Path

UPSTREAM_REMOTE = "upstream"
UPSTREAM_SLUG = "anefzaoui/misterdownloadmanager"
UPSTREAM_BRANCH = "main"
FORK_REMOTE = "origin"
FORK_BASE = "dev"
PROTECTED = ("dev", "main")
SYNC_PREFIX = "upstream-sync/"
LEDGER = "tools/upstream-sync/ledger.md"
# The fork's own record of what changed and what's planned: upstream's lines never land here.
OWN = ("CHANGELOG.md", "ROADMAP.md", LEDGER)
STATUSES = ("picked", "ported", "skipped", "covered", "deferred")
RECORDABLE = ("skipped", "covered", "deferred", "ported")
# A move the fork made counts only when git is this sure of it; below, git's guess (50 %) can pair
# a deleted file with an unrelated new one and carry upstream's fix into the wrong file.
RENAMES = "find-renames=90%"
STALE_DEFERRED_DAYS = 14
REPORT_ROWS = 200
REPORT_CHARS = 60_000  # GitHub's issue body holds 65,536

# Hints for the reviewer, never decisions. Upstream writes plain language ("the window never
# waits", "no longer holds up"), so its phrasings are here as well as the usual words.
CRITICAL = re.compile(
    r"secur|vulnerab|\bcve-|crash|panic|freez|\bhangs?\b|deadlock|corrupt|data loss|heap|"
    r"overflow|\bleak|injection|traversal|signature|integrity|checksum|one piece short|"
    r"can't stop|can't start|won't start|window's thread|beside the window|holds? up|"
    r"never waits|stops? answering|not responding|lost|\bshort\b|untrusted|privacy|referr|"
    r"only the service's own|permissions|\btls\b|certificate|\btokens?\b|password",
    re.I,
)
CRITICAL_AREAS = ("crates/debrid/", "crates/ipc/", "crates/nmhost/", "crates/crash/",
                  "crates/remote/", "crates/app/src/help.rs", "extension/background.js",
                  "tools/installer/", ".github/workflows/")
SENSITIVE = re.compile(
    r"^(\.claude/|\.github/|\.cargo/|\.vscode/|tools/|\.gitattributes$|\.gitmodules$|"
    r"rust-toolchain(\.toml)?$|\.mcp\.json$)|"
    r"(^|/)(claude\.md|agents\.md|build\.rs|package(-lock)?\.json)$",
    re.I,  # Windows' file names ignore case: .Claude/ is .claude/
)
# What cargo reads before it builds anything: never run cargo over it unread.
# A theme's file or folder (crates/app/assets/themes/<id>.toml or <id>/): this fork keeps
# Graphite only (plan 05.5b).
OTHER_THEME = re.compile(r"^crates/app/assets/themes/([^/]+?)(?:\.toml$|/)")
CARGO_CONFIG = re.compile(r"(^|/)\.cargo/|(^|/)rust-toolchain", re.I)
OTHER_PLATFORMS = re.compile(r"^(linux|macos|mac|ubuntu|flatpak|safari|helium)\b|linux and macos",
                             re.I)
ADDED_LINTS = (
    ("P", re.compile(r'target_os\s*=\s*"(linux|macos|ios|android)"|cfg\(\s*unix\s*\)|'
                     r'not\(\s*windows\s*\)|target_family\s*=\s*"unix"')),
    ("P", re.compile(r"right-to-left|\bis_rtl\b|\brtl\b|text direction|dir=\W?rtl", re.I)),
    ("I", re.compile(r"anefzaoui(?!/[\w.-]+\.git)", re.I)),
)
ISSUE = re.compile(r"(?<![\w/])#(\d+)")
PICK_TRAILER = re.compile(r"^Upstream-Pick:\s*([0-9a-fA-F]{40})\s*$", re.M)
# A pick made by hand with `git cherry-pick -x` (upstream's own such lines are neutralised in picks).
CHERRY_X = re.compile(r"^\(cherry picked from commit ([0-9a-fA-F]{40})\)\s*$", re.M)
PORT_TRAILER = re.compile(r"^Upstream-Port:\s*([0-9a-fA-F]{7,40})\s*$", re.M)
REVERTS = re.compile(r"^This reverts commit ([0-9a-fA-F]{40})", re.M)
MARKERS = re.compile(r"^(?=Upstream-(Pick|Port):|This reverts commit|.*cherry picked from commit)",
                     re.M | re.I)
ROW = re.compile(r"^\|\s*([0-9a-fA-F]{7,40})\s*\|\s*([\w?-]+)\s*\|(.*)\|\s*$")
# A reference GitHub would turn into a link, and so a mention on the original's issue or PR.
UPSTREAM_LINK = re.compile(
    r"anefzaoui/misterdownloadmanager(#\d+|/(issues|pull|discussions)/\d+)|"
    r"github\.com/anefzaoui/misterdownloadmanager/(issues|pull|discussions)/", re.I)


MENTION = re.compile(r"(?<![\w/])@anefzaoui\b", re.I)


class Fail(Exception):
    """A stop with a reason for the person: printed, exit code 1."""


def git(*args: str, check: bool = True, env: dict | None = None,
        stdin: str | None = None) -> subprocess.CompletedProcess:
    # Paths come back as they are (no "\303\251" quoting) and go in as they are (no globbing:
    # `git rm "img[1].txt"` must never remove img1.txt).
    try:
        r = subprocess.run(
            ["git", "-c", "core.quotepath=off", *args],
            input=stdin,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            env={**os.environ, "GIT_LITERAL_PATHSPECS": "1", **(env or {})},
        )
    except OSError as e:
        raise Fail(f"git {' '.join(args[:3])}…: {e}")
    if check and r.returncode != 0:
        raise Fail(f"git {' '.join(args)}: {r.stderr.strip() or r.stdout.strip()}")
    return r


def out(*args: str) -> str:
    return git(*args).stdout.strip()


def zlist(*args: str) -> list[str]:
    """A NUL-separated list (`-z` added by the caller), exactly as git wrote each name."""
    return [p for p in git(*args).stdout.split("\0") if p]


def root() -> Path:
    return Path(out("rev-parse", "--show-toplevel"))


def slug(url: str) -> str:
    """owner/name of a GitHub remote address (https or ssh), lower case; '' when not GitHub."""
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


def full_sha(sha: str) -> str | None:
    r = git("rev-parse", "--verify", "--quiet", f"{sha}^{{commit}}", check=False)
    return r.stdout.strip() if r.returncode == 0 else None


def current_branch() -> str:
    return git("symbolic-ref", "--short", "-q", "HEAD", check=False).stdout.strip()


def git_path(name: str) -> Path:
    return Path(out("rev-parse", "--git-path", name))


def today() -> str:
    return dt.date.today().isoformat()


# ---------------------------------------------------------------------------------------------
# What the fork removed or moved, and what it changed


@dataclass
class Fork:
    gone: set[str]                 # files deleted here (not moved)
    gone_dirs: set[str]            # folders deleted here, with nothing in them moved out
    moved: dict[str, str]          # upstream path -> the fork's (git sure: 90 % or more alike)
    rewritten: dict[str, str]      # deleted here, perhaps rewritten as (50-90 % alike)
    changed: set[str]              # every path the fork changed, both names of a move
    added: set[str]                # files the fork added
    by_case: dict[str, str]        # lower-case path (files and folders) -> as the fork has it
    max_path: int | None = None    # Windows without core.longpaths: the longest path that fits

    def is_gone(self, path: str) -> bool:
        if path in self.moved or path in self.rewritten:
            return False
        if path in self.gone:
            return True
        parts = path.split("/")
        return any("/".join(parts[:i]) in self.gone_dirs for i in range(1, len(parts)))


def renames(base: str, head: str, threshold: str) -> list[tuple[int, str, str]]:
    items = zlist("diff", f"-M{threshold}", "--diff-filter=R", "--name-status", "-z", base, head)
    return [(int(items[i][1:] or 0), items[i + 1], items[i + 2]) for i in range(0, len(items), 3)]


def licence_text(path: str) -> bool:
    """A licence or notice file: alike in every folder (MIT texts, credits), so git's pairing of
    two of them says nothing about a move (plan 05.5b: an old theme's icon licence is not
    Graphite's)."""
    name = path.rsplit("/", 1)[-1]
    return (name.startswith(("LICENSE", "COPYING", "AUTHORS"))
            or name in ("README.txt", "readme.txt", "SOURCES.txt"))


def fork_state(base: str, head: str) -> Fork:
    deleted = set(zlist("diff", "--no-renames", "--diff-filter=D", "--name-only", "-z", base, head))
    moved, rewritten = {}, {}
    for score, src, dst in renames(base, head, "50%"):
        if licence_text(src) and licence_text(dst):
            continue  # deleted here: the source counts as gone
        (moved if score >= 90 else rewritten)[src] = dst
    gone = deleted - moved.keys() - rewritten.keys()
    then_dirs = set(zlist("ls-tree", "-r", "-d", "--name-only", "-z", base))
    now_dirs = set(zlist("ls-tree", "-r", "-d", "--name-only", "-z", head))
    alive = set()  # folders that still hold, or moved out, something the fork keeps
    for path in zlist("ls-tree", "-r", "--name-only", "-z", base):
        if path not in gone:
            parts = path.split("/")
            alive.update("/".join(parts[:i]) for i in range(1, len(parts)))
    changed = set()
    items = zlist("diff", "-M50%", "--name-status", "-z", base, head)
    i = 0
    while i < len(items):
        status = items[i]
        n = 2 if status[0] in "RC" else 1
        changed.update(items[i + 1:i + 1 + n])
        i += 1 + n
    added = set(zlist("diff", "--no-renames", "--diff-filter=A", "--name-only", "-z", base, head))
    by_case = {}
    for path in zlist("ls-tree", "-r", "--name-only", "-z", head) + sorted(now_dirs):
        by_case.setdefault(path.lower(), path)
    max_path = None
    if os.name == "nt" and git("config", "--bool", "core.longpaths",
                               check=False).stdout.strip() != "true":
        max_path = 259 - len(str(root())) - 1
    return Fork(gone, (then_dirs - now_dirs) - alive, moved, rewritten, changed, added, by_case,
                max_path)


def existing(rev: str, paths: list[str]) -> set[str]:
    """Which of these paths (files or folders) exist in a commit's tree: one git process."""
    paths = [x for x in dict.fromkeys(paths) if "\n" not in x]
    if not paths:
        return set()
    r = git("cat-file", "--batch-check=%(objectname)", stdin="".join(f"{rev}:{x}\n" for x in paths),
            check=False)
    return {x for x, line in zip(paths, r.stdout.splitlines()) if not line.endswith("missing")}


def gone_checker(fork: Fork, sha: str, paths: list[str]):
    """Gone here because the fork removed it, not because upstream did before this commit (a
    file upstream deleted and brings back, a folder it dropped and starts again)."""
    def chain(path: str) -> list[str]:
        parts = path.split("/")
        return ["/".join(parts[:i]) for i in range(1, len(parts) + 1)]
    there = existing(f"{sha}^", [x for path in paths for x in chain(path)])

    def in_upstream(path: str) -> bool:
        if not set(chain(path)) <= set(c for x in paths for c in chain(x)):
            there.update(existing(f"{sha}^", chain(path)))
        if path in fork.gone:
            return path in there
        return any(d in there for d in chain(path)[:-1] if d in fork.gone_dirs)
    return lambda path: fork.is_gone(path) and in_upstream(path)


def case_clash(fork: Fork, path: str) -> str | None:
    """The fork's path that `path` would land on in Windows' case-blind file system, if another."""
    parts = path.split("/")
    for i in range(1, len(parts) + 1):
        prefix = "/".join(parts[:i])
        have = fork.by_case.get(prefix.lower())
        if have and have != prefix:
            return have
    return None


# ---------------------------------------------------------------------------------------------
# What has been taken or decided


@dataclass
class Row:
    sha: str
    status: str
    date: str
    ref: str
    subject: str
    note: str


def read_ledger(path: Path) -> tuple[list[Row], list[str]]:
    """The rows, and what's wrong with any (a bad status, a duplicate)."""
    rows, problems, seen = [], [], set()
    if not path.exists():
        return rows, problems
    try:
        text = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        raise Fail(f"{LEDGER} is not UTF-8: save it as UTF-8 again")
    for n, line in enumerate(text.splitlines(), 1):
        m = ROW.match(line)
        if not m:
            continue
        cells = [c.strip() for c in m.group(3).split("|")]
        cells += [""] * (4 - len(cells))
        row = Row(m.group(1).lower(), m.group(2).lower(), *cells[:4])
        if row.status not in STATUSES:
            problems.append(f"ledger line {n}: unknown status {row.status!r} (row ignored)")
            continue
        if row.sha in seen:
            problems.append(f"ledger line {n}: a second row for {row.sha[:12]} (the last counts)")
        seen.add(row.sha)
        rows.append(row)
    return rows, problems


@dataclass
class Taken:
    status: dict[str, str] = field(default_factory=dict)   # upstream sha -> its status here
    warnings: list[str] = field(default_factory=list)

    def of(self, sha: str) -> str | None:
        return self.status.get(sha)


DONE = {"picked", "ported", "skipped", "covered", "equivalent"}


def patch_ids(args: list[str]) -> dict[str, str]:
    """commit -> its patch-id, for the commits `git log <args>` lists (one pass, two processes)."""
    diff = git("log", "-p", "--no-merges", "--no-color", "--no-ext-diff", "--format=medium",
               *args).stdout
    res = {}
    for line in git("patch-id", "--stable", stdin=diff).stdout.splitlines():
        pid, _, commit = line.partition(" ")
        res[commit.strip()] = pid
    return res


def taken(refs: list[str], fork_point: str, up: str, in_range: set[str], ledger: Path) -> Taken:
    t = Taken()
    refs = [r for r in dict.fromkeys(refs) if has_ref(r)]
    picked, ported, reverted = set(), set(), set()
    undone: dict[str, str] = {}  # a fork commit -> the revert that undoes it (still in effect)

    def upstream_sha(sha: str) -> str:
        """An upstream commit's full sha from a short one, by prefix: no git per row."""
        sha = sha.lower()
        if len(sha) == 40:
            return sha
        hits = [s for s in in_range if s.startswith(sha)]
        return hits[0] if len(hits) == 1 else sha

    if refs:
        # Oldest first, so a later revert undoes an earlier pick, a revert of that revert
        # redoes it, and a pick again redoes it too. effect: commit -> (+1 or -1, picks, ports).
        raw = out("log", "--reverse", "--topo-order", "--format=%H%x1f%B%x1e", f"^{fork_point}",
                  *refs)
        effect: dict[str, tuple[int, set[str], set[str]]] = {}
        for record in raw.split("\x1e"):
            if "\x1f" not in record:
                continue
            sha, body = record.strip().split("\x1f", 1)
            for r in REVERTS.findall(body):
                r = r.lower()
                back = [k for k, v in undone.items() if v == r]  # a revert of a revert
                for k in back:
                    del undone[k]
                parents = out("rev-list", "--parents", "-n", "1", r).split()[1:] if full_sha(r) else []
                inner = (out("rev-list", f"{parents[0]}..{parents[1]}").splitlines()
                         if len(parents) == 2 else [])
                if not back:
                    for c in [r] + inner:
                        undone[c] = sha
                rev = effect.get(r)
                if rev is None and inner:  # a merged PR reverted: all it brought
                    rp, rq = set(), set()
                    for c in inner:
                        e = effect.get(c)
                        if e and e[0] > 0:
                            rp, rq = rp | e[1], rq | e[2]
                    rev = (1, rp, rq) if rp or rq else None
                if rev:
                    effect[sha] = (-rev[0], rev[1], rev[2])
                    if rev[0] > 0:
                        picked.difference_update(rev[1])
                        ported.difference_update(rev[2])
                        reverted.update(rev[1] | rev[2])
                    else:
                        picked.update(rev[1])
                        ported.update(rev[2])
                        reverted.difference_update(rev[1] | rev[2])
            if REVERTS.search(body):
                continue
            p = {upstream_sha(x) for x in PICK_TRAILER.findall(body) + CHERRY_X.findall(body)}
            q = {upstream_sha(x) for x in PORT_TRAILER.findall(body)}
            if p or q:
                effect.setdefault(sha, (1, p, q))
                picked |= p
                ported |= q
                reverted -= p | q
    stale = sorted(s for s in picked | ported if s not in in_range)
    if stale:
        t.warnings.append(f"{len(stale)} trailer(s) name commits that aren't upstream's since the "
                          f"fork point (rewritten upstream history?): "
                          + ", ".join(s[:12] for s in stale[:5]) + ("…" if len(stale) > 5 else ""))
    for sha in picked:
        t.status[sha] = "picked"
    for sha in ported:
        t.status[sha] = "ported"
    for sha in reverted:
        t.status[sha] = "reverted"
    # The same change already here by patch-id (a pick made by hand, a rebase), unless the fork
    # commit that carries it was reverted since.
    equal = set()
    for ref in refs:
        for line in out("cherry", ref, up, fork_point).splitlines():
            if line.startswith("- ") and line[2:].strip() not in reverted:
                equal.add(line[2:].strip())
    if equal and refs:
        mine = patch_ids([f"^{fork_point}", *refs])
        theirs = patch_ids(["--no-walk", *sorted(equal)])
        for sha in equal:
            carriers = [c for c, pid in mine.items() if pid == theirs.get(sha)]
            if carriers and all(c in undone for c in carriers):
                t.status[sha] = "reverted"
            else:
                t.status.setdefault(sha, "equivalent")
    # Picks on other sync branches not in dev yet: their PR open (in-pr), or local, unpushed
    # (in-branch). Never picked twice; a stale branch is reported.
    current = current_branch()
    target = slug(remote_url(FORK_REMOTE))
    pushed = set()
    for ref in out("for-each-ref", "--format=%(refname:short)",
                   f"refs/remotes/{FORK_REMOTE}/{SYNC_PREFIX}").splitlines():
        branch = ref.split("/", 1)[1]
        pushed.add(branch)
        if not refs or branch == current:
            continue
        log = out("log", "--format=%B", f"^{refs[0]}", ref)
        shas = {upstream_sha(x) for x in PICK_TRAILER.findall(log) + PORT_TRAILER.findall(log)}
        if not shas:
            continue
        prs = gh_json(["pr", "list", "-R", target, "--state", "open", "--head", branch,
                       "--json", "number"]) if target else None
        if prs:
            for sha in shas:
                t.status.setdefault(sha, "in-pr")
        elif prs is None:
            t.warnings.append(f"{branch} holds picks not in dev, and gh can't tell whether its PR "
                              "is open: don't pick them twice")
    for branch in out("for-each-ref", "--format=%(refname:short)",
                      f"refs/heads/{SYNC_PREFIX}").splitlines():
        if not refs or branch == current or branch in pushed:
            continue
        log = out("log", "--format=%B", f"^{refs[0]}", branch)
        shas = {upstream_sha(x) for x in PICK_TRAILER.findall(log) + PORT_TRAILER.findall(log)}
        fresh = [s for s in shas if t.status.get(s) not in DONE]
        for sha in fresh:
            t.status.setdefault(sha, "in-branch")
        if fresh:
            t.warnings.append(f"local branch {branch} (not pushed) holds {len(fresh)} pick(s) not "
                              "in dev: finish it, or delete it to pick them here")
    rows, problems = read_ledger(ledger)
    t.warnings += problems
    unknown = []
    for row in rows:
        full = upstream_sha(row.sha)
        if full not in in_range:
            unknown.append(row.sha[:12])
            continue
        if row.status == "picked":
            # Only a pick that is really here counts: its trailer, or the same change by
            # patch-id. A row whose pick isn't (a squash dropped the trailer, the pick was
            # reset away, the row came along from another branch) says so.
            if full in reverted or t.status.get(full) == "reverted":
                t.status[full] = "reverted"
            elif full in picked or t.status.get(full) == "equivalent":
                t.status[full] = "picked"
            else:
                t.status[full] = "picked?"
        elif row.status == "ported":
            t.status[full] = ("reverted" if full in reverted
                              else "ported" if full in ported else "ported?")
        else:
            t.status[full] = row.status
        if row.status == "deferred":
            try:
                age = (dt.date.today() - dt.date.fromisoformat(row.date)).days
            except ValueError:
                t.warnings.append(f"{full[:12]} deferred with no date ({row.date!r}): record it "
                                  "again, so it can come up for review")
                continue
            if age > STALE_DEFERRED_DAYS:
                t.warnings.append(f"{full[:12]} deferred {age} days ago: decide it again")
    if unknown:
        t.warnings.append(f"{len(unknown)} ledger row(s) aren't upstream commits since the fork "
                          "point (rewritten upstream history, or a typo): "
                          + ", ".join(unknown[:5]) + ("…" if len(unknown) > 5 else ""))
    for sha, st in t.status.items():
        if st == "picked?":
            t.warnings.append(f"{sha[:12]} is recorded picked, but its pick isn't in dev or here "
                              "(a squash merge, a reset, or a row from another branch): decide "
                              "it again")
        if st == "ported?":
            t.warnings.append(f"{sha[:12]} is recorded ported, but no `Upstream-Port` trailer for "
                              "it is in dev or here: its PR not merged yet, or closed")
        if st == "reverted":
            t.warnings.append(f"{sha[:12]} was taken, then reverted here: decide it again")
    return t


# ---------------------------------------------------------------------------------------------
# Sorting one upstream commit


@dataclass
class Commit:
    sha: str
    index: int
    date: str
    subject: str
    kind: str = ""
    flags: str = ""
    critical_why: list[str] = field(default_factory=list)
    issues: list[str] = field(default_factory=list)
    same_issue: list[str] = field(default_factory=list)
    changelog: list[str] = field(default_factory=list)
    touches_fork: list[str] = field(default_factory=list)
    left_out: list[str] = field(default_factory=list)
    sensitive: list[str] = field(default_factory=list)
    conflicts: list[str] = field(default_factory=list)
    after: list[str] = field(default_factory=list)
    notes: list[str] = field(default_factory=list)
    status: str | None = None


def changes(sha: str) -> list[tuple[str, str, str | None]]:
    """(status letter, path, renamed-to) for each change; a rename once, with both names."""
    items = zlist("diff-tree", "-r", "-M", "--no-commit-id", "--name-status", "-z", "--root", sha)
    res, i = [], 0
    while i < len(items):
        st = items[i][0]
        if st in "RC":
            res.append((st, items[i + 1], items[i + 2]))
            i += 3
        else:
            res.append((st, items[i + 1], None))
            i += 2
    return res


def simulate(sha: str, onto: str) -> tuple[list[str], str]:
    """Cherry-pick `sha` onto `onto` in memory (nothing on disk moves): the conflicted paths, and
    the tree it would give."""
    r = git("merge-tree", "--write-tree", "--name-only", "--no-messages", "-z", "-X", RENAMES,
            f"--merge-base={sha}^", onto, sha, check=False)
    if r.returncode not in (0, 1):
        raise Fail(f"merge-tree on {sha[:12]}: {r.stderr.strip()}")
    parts = r.stdout.split("\0")
    conflicts = []
    for p in parts[1:]:
        if not p:
            break
        conflicts.append(p)
    return list(dict.fromkeys(conflicts)), parts[0].strip()


def added_lines(sha: str) -> dict[str, list[str]]:
    """Every file's added lines in a commit, from one `git show`."""
    res: dict[str, list[str]] = {}
    current, header = None, False
    for ln in git("show", "--format=", "--no-color", "--unified=0", "--no-renames", "--no-ext-diff",
                  "--src-prefix=a/", "--dst-prefix=b/", sha).stdout.splitlines():
        if ln.startswith("diff --git ") or ln.startswith("diff --cc "):
            current, header = None, True
        elif header and ln.startswith("@@"):
            header = False
        elif header and ln.startswith("+++ "):
            name = ln[4:].rstrip("\t")
            if name.startswith('"') and name.endswith('"'):  # git's C quoting
                name = codecs.escape_decode(name[1:-1].encode("utf-8"))[0].decode("utf-8", "replace")
            current = name[2:] if name.startswith("b/") else None
            if current is not None:
                res.setdefault(current, [])
        elif not header and ln.startswith("+") and current is not None:
            res[current].append(ln[1:])
    return res


def crate_tomls(paths: list[str]) -> list[str]:
    tomls = set()  # a crate's own manifest: a dependency it gained, which a later fix uses
    for p in paths:
        m = re.match(r"(crates/[^/]+)/", p)
        if m:
            tomls.add(f"{m.group(1)}/Cargo.toml")
    return sorted(tomls)


def sort_commit(c: Commit, onto: str, fork: Fork, fork_point: str, t: Taken,
                messages: dict[str, str]) -> Commit:
    msg = messages.get(c.sha) or out("log", "-1", "--format=%B", c.sha)
    ch = changes(c.sha)
    paths = list(dict.fromkeys(p for _, a, b in ch for p in (a, b) if p))
    adds = added_lines(c.sha)
    if "CHANGELOG.md" in paths:
        c.changelog = [ln for ln in adds.get("CHANGELOG.md", []) if ln.strip()]
    c.issues = sorted(set(ISSUE.findall(msg)), key=int)
    words = CRITICAL.findall(msg + "\n" + "\n".join(c.changelog))
    c.critical_why = sorted({w.lower() for w in words})
    areas = [a for a in CRITICAL_AREAS if any(p.startswith(a) for p in paths)]
    if areas:
        c.notes.append("touches " + ", ".join(areas) + " (security, IPC, crash or release code): "
                       "read it for a fix that matters even when its subject doesn't say so")
    themes = sorted({m.group(1) for p in paths if (m := OTHER_THEME.match(p))} - {"graphite"})
    if themes:
        c.notes.append(f"a theme other than Graphite ({', '.join(themes)}): this fork keeps "
                       "Graphite only (plan 05.5): skip")
    is_gone = gone_checker(fork, c.sha, paths)
    own = [p for p in paths if p in OWN]
    gone = [p for p in paths if p not in OWN and is_gone(p)]
    rest = [p for p in paths if p not in OWN and p not in gone]
    c.left_out = gone + own
    c.touches_fork = [p for p in rest if p in fork.changed]
    c.sensitive = [p for p in rest if SENSITIVE.search(p)]
    if any(p.endswith("Cargo.toml") for p in paths):
        c.notes.append("dependencies change: a new crate's build script and macros run at build; "
                       "read what it adds")
    for _, src, _dst in ch:
        if src in fork.rewritten:
            c.notes.append(f"{src} was deleted here, perhaps rewritten as {fork.rewritten[src]}: "
                           "check whether the fix belongs there")
        elif src in gone and "/" in src:
            folder = src.rsplit("/", 1)[0] + "/"
            near = sorted(a for a in fork.added if a.startswith(folder))[:3]
            if near:
                c.notes.append(f"{src} was deleted here, and the fork added {', '.join(near)} "
                               "beside it: check whether the fix belongs in one of them")
    clashes = [(p, case_clash(fork, p)) for p in rest if case_clash(fork, p)]
    c.notes += [f"{p} would land on the fork's {h} (Windows ignores case): read it as a port"
                for p, h in clashes]
    look = rest or paths
    if len(look) > 100:  # Windows' command line: the folders instead of every file
        look = sorted({p.rsplit("/", 1)[0] if "/" in p else p for p in look})
        if len(look) > 100:
            look = sorted({p.split("/", 1)[0] for p in look})
    earlier = out("rev-list", "--reverse", "--no-merges", f"{fork_point}..{c.sha}^", "--",
                  *look, *crate_tomls(look))
    c.after = [s for s in earlier.splitlines() if t.of(s) not in DONE]
    if not rest:
        c.kind = "removed" if gone else "nothing"
    else:
        moves_out = [(src, dst) for st, src, dst in ch
                     if st == "R" and dst and (dst in OWN or is_gone(dst)) and src in rest]
        moves_own = [(src, dst) for st, src, dst in ch if st == "R" and src in OWN]
        conflicts, tree = simulate(c.sha, onto)
        c.conflicts = [p for p in conflicts if p not in OWN and not is_gone(p)]
        if not conflicts and tree == out("rev-parse", f"{onto}^{{tree}}"):
            c.kind = "nothing"
            c.notes.append("already in the fork: applying it changes nothing")
            c.notes = list(dict.fromkeys(c.notes))
            return c
        # Files upstream had and changes, which the fork doesn't have any more (not a path git
        # only proposes, such as where a folder rename would carry one of the fork's own files).
        missing = sorted((set(c.conflicts) - existing(onto, c.conflicts))
                         & existing(f"{c.sha}^", c.conflicts))
        if missing:
            c.notes.append(f"the fork no longer has {', '.join(missing)} (deleted after the fork "
                           "point, or by an earlier pick): if that was on purpose, skip this")
        if fork.max_path is not None:
            long = [p for p in rest if len(p) > fork.max_path]
            if long:
                c.notes.append(f"{long[0]} is too long for Windows here without "
                               "`git config core.longpaths true`: the pick would fail")
        if clashes:
            c.kind = "port"
        elif moves_out or moves_own:
            c.kind = "port"
            c.notes += [f"upstream moves {s} to {d}, which this fork doesn't keep: a pick would "
                        "delete the fork's file" for s, d in moves_out + moves_own]
        elif c.conflicts == ["Cargo.lock"]:
            c.kind = "lock"
        elif c.conflicts:
            c.kind = "port"
        elif gone or set(conflicts) & set(own):
            c.kind = "prune"
        else:
            c.kind = "clean"
        if c.kind == "port" and c.conflicts:
            if all(p in fork.changed for p in c.conflicts):
                c.notes.append("the fork changed the conflicting files: port by hand now; never "
                               "take a feature only to make this apply")
            elif c.after and c.critical_why:
                c.notes.append("critical-looking: port it now; upstream's earlier commits (A) "
                               "show what it builds on, never what it waits for")
            elif c.after:
                c.notes.append("the fork never changed some conflicting files: upstream's earlier "
                               "commits (A) are missing; decide those first, then sort again")
        added = {p: adds.get(p, []) for p in rest}
        for flag, rx in ADDED_LINTS:
            hits = [p for p, lines in added.items() if any(rx.search(ln) for ln in lines)]
            if hits and flag not in c.flags:
                c.flags += flag
            what = "another platform or text direction" if flag == "P" else "upstream's identity"
            c.notes += [f"{p}: added lines name {what}" for p in hits]
    if OTHER_PLATFORMS.search(c.subject) and "P" not in c.flags:
        c.flags += "P"
    if c.critical_why:
        c.flags = "C" + c.flags
    if c.sensitive:
        c.flags += "S"
        c.notes.append("sensitive (" + ", ".join(c.sensitive) + "): read every line; never "
                       "batch-picked (pick it alone with --allow-sensitive after review)")
    if c.after:
        c.flags += "A"
    if c.touches_fork and c.kind in ("clean", "prune", "lock"):
        c.notes.append("applies, but in files the fork changed: review the logic, not just the text")
    c.notes = list(dict.fromkeys(c.notes))
    return c


def pending(fork_point: str, up: str) -> tuple[list[Commit], dict[str, str]]:
    commits, messages = [], {}
    raw = out("log", "--reverse", "--topo-order", "--no-merges", "--date=short",
              "--format=%H%x1f%ad%x1f%s%x1f%B%x1e", f"{fork_point}..{up}")
    for i, record in enumerate(r for r in raw.split("\x1e") if r.strip()):
        sha, date, subject, body = record.strip().split("\x1f", 3)
        commits.append(Commit(sha, i + 1, date, subject))
        messages[sha] = body
    return commits, messages


def evil_merges(fork_point: str, up: str, t: Taken) -> list[Commit]:
    """Upstream merges whose tree isn't what merging their parents gives: changes of their own."""
    res = []
    for line in out("rev-list", "--merges", "--reverse", "--format=%H %P",
                    f"{fork_point}..{up}").splitlines():
        if line.startswith("commit "):
            continue
        sha, *parents = line.split()
        if t.of(sha) in DONE:
            continue
        r = git("merge-tree", "--write-tree", *parents, check=False) if len(parents) == 2 else None
        tree = r.stdout.split("\n", 1)[0].strip() if r else ""
        if r is None or r.returncode != 0 or tree != out("rev-parse", f"{sha}^{{tree}}"):
            c = Commit(sha, 0, out("log", "-1", "--date=short", "--format=%ad", sha),
                       out("log", "-1", "--format=%s", sha), kind="merge")
            c.notes.append("an upstream merge with changes of its own: `git show --cc` it, port "
                           "what's wanted, `record` it")
            res.append(c)
    return res


# ---------------------------------------------------------------------------------------------
# Commands


@dataclass
class Context:
    up: str
    fork_point: str
    onto: str
    fork: Fork
    t: Taken
    in_range: set[str]


def context() -> Context:
    up, forkref = upstream_ref(), fork_ref()
    for ref in (up, forkref):
        if not has_ref(ref):
            raise Fail(f"{ref} is missing: run `doctor --fix`, then `status --fetch`")
    fork_point = git("merge-base", forkref, up, check=False).stdout.strip()
    if not fork_point:
        raise Fail(f"{forkref} and {up} share no history")
    in_range = set(out("rev-list", f"{fork_point}..{up}").splitlines())
    fork = fork_state(fork_point, "HEAD")
    t = taken([forkref, "HEAD"], fork_point, up, in_range, root() / LEDGER)
    return Context(up, fork_point, "HEAD", fork, t, in_range)


def sorted_commits(ctx: Context, everything: bool) -> list[Commit]:
    commits, messages = pending(ctx.fork_point, ctx.up)
    for c in commits:
        c.status = ctx.t.of(c.sha)
    todo = [c for c in commits if everything or c.status not in DONE]
    # Each commit is sorted on its own, read only (merge-tree writes objects, never refs), so a
    # few at once: on Windows most of the time is starting git processes.
    pool = ThreadPoolExecutor(max_workers=min(4, os.cpu_count() or 1))
    try:
        list(pool.map(lambda c: sort_commit(c, ctx.onto, ctx.fork, ctx.fork_point, ctx.t,
                                            messages), todo))
    finally:
        pool.shutdown(wait=True, cancel_futures=True)
    by_issue: dict[str, list[Commit]] = {}
    for c in todo:
        for n in c.issues:
            by_issue.setdefault(n, []).append(c)
    for c in todo:
        others = {x.sha: x.index for n in c.issues for x in by_issue[n] if x.sha != c.sha}
        c.same_issue = sorted(others, key=others.get)
    return todo + evil_merges(ctx.fork_point, ctx.up, ctx.t)


def cmd_status(a) -> int:
    if a.fetch:
        git("fetch", UPSTREAM_REMOTE, "--no-tags", "--prune")
        git("fetch", FORK_REMOTE)
    ctx = context()
    todo = sorted_commits(ctx, a.all)
    syncs = sync_branches()
    if a.json:
        print(json.dumps({
            "upstream": ctx.up, "fork_point": ctx.fork_point, "onto": ctx.onto,
            "sync_branches": syncs, "warnings": ctx.t.warnings,
            "commits": [asdict(c) for c in todo],
        }, indent=1))
        return 0
    print(f"Upstream {ctx.up} since the fork point {ctx.fork_point[:12]}: {len(ctx.in_range)} "
          f"commits; {len(todo)} to look at, in upstream's order. Sorted against {ctx.onto}.")
    for s in syncs + ctx.t.warnings:
        print(f"! {s}")
    print("\n|  # | sha | date | kind | flags | status | subject |")
    print("|---:|---|---|---|---|---|---|")
    for c in todo:
        print(f"| {c.index or '-'} | {c.sha[:12]} | {c.date} | {c.kind} | {c.flags} | "
              f"{c.status or ''} | {c.subject.replace('|', '/')} |")
    print("\nFlags: C critical-looking, S sensitive, A earlier upstream commits not taken, "
          "I upstream's identity, P another platform or language.")
    for c in todo:
        bits = []
        if c.critical_why:
            bits.append("critical? " + ", ".join(c.critical_why))
        if c.conflicts:
            bits.append("conflicts in " + ", ".join(c.conflicts))
        if c.after:
            bits.append("A: " + ", ".join(s[:12] for s in c.after))
        if c.same_issue:
            bits.append("same issue as " + ", ".join(s[:12] for s in c.same_issue))
        bits += c.notes
        if bits:
            print(f"\n{c.index or '-'} {c.sha[:12]} ({c.kind}):\n  - " + "\n  - ".join(bits))
    return 0


def cmd_show(a) -> int:
    ctx = context()
    sha = full_sha(a.sha)
    if not sha or sha not in ctx.in_range:
        raise Fail(f"{a.sha} is not an upstream commit since the fork point")
    commits, messages = pending(ctx.fork_point, ctx.up)
    c = next((x for x in commits if x.sha == sha), None)
    if c is None:
        c = Commit(sha, 0, out("log", "-1", "--date=short", "--format=%ad", sha),
                   out("log", "-1", "--format=%s", sha))
    c.status = ctx.t.of(sha)
    if len(out("rev-list", "--parents", "-n", "1", sha).split()) > 2:
        merges = [m for m in evil_merges(ctx.fork_point, ctx.up, Taken()) if m.sha == sha]
        if merges:
            c = merges[0]
        else:
            c.kind = "merge"
            c.notes.append("a plain merge: its changes come with the commits it merged")
    else:
        sort_commit(c, ctx.onto, ctx.fork, ctx.fork_point, ctx.t, messages)
    print(json.dumps(asdict(c), indent=1))
    return 0


def gh_json(args: list[str]):
    if os.environ.get("UPSTREAM_SYNC_OFFLINE") or not shutil.which("gh"):
        return None
    try:
        r = subprocess.run(["gh", *args], capture_output=True, text=True, encoding="utf-8",
                           errors="replace")
    except OSError:
        return None
    if r.returncode != 0:
        return None
    try:
        return json.loads(r.stdout or "null")
    except json.JSONDecodeError:
        return None


def watch_data() -> dict:
    """Upstream's work not on its main branch yet: open PRs and branches. Read only."""
    prs = gh_json(["pr", "list", "-R", UPSTREAM_SLUG, "--state", "open", "--limit", "100",
                   "--json", "number,title,headRefName,updatedAt,isDraft"])
    advisories = gh_json(["api", f"repos/{UPSTREAM_SLUG}/security-advisories"])
    branches = []
    for ref in out("for-each-ref", "--format=%(refname:short)",
                   f"refs/remotes/{UPSTREAM_REMOTE}/").splitlines():
        name = ref.split("/", 1)[1] if "/" in ref else ref
        if name in ("HEAD", UPSTREAM_BRANCH, "pr-shots") or not has_ref(ref):
            continue
        n = int(out("rev-list", "--count", "--no-merges", f"{upstream_ref()}..{ref}") or 0)
        if n:
            subj = out("log", "-1", "--format=%s", ref)
            branches.append({"branch": name, "commits": n, "last": subj,
                             "critical": bool(CRITICAL.search(subj + " " + name))})
    for p in prs or []:
        p["critical"] = bool(CRITICAL.search(p["title"] + " " + p["headRefName"]))
    return {"prs": prs, "advisories": advisories, "branches": branches}


def cmd_watch(a) -> int:
    d = watch_data()
    if a.json:
        print(json.dumps(d, indent=1))
        return 0
    if d["prs"] is None:
        print("! gh is missing or can't read upstream's PRs: only its branches are listed")
    else:
        print(f"Upstream's open PRs: {len(d['prs'])} (C critical-looking). Not picked until merged "
              "there; port one early only when the owner says so.\n")
        for p in sorted(d["prs"], key=lambda p: (not p["critical"], -p["number"])):
            print(f"  {'C' if p['critical'] else ' '} {p['number']:>5} "
                  f"{'(draft) ' if p['isDraft'] else ''}{p['title']}  [{p['headRefName']}]")
    if not isinstance(d["advisories"], list):
        print("\n! upstream's security advisories could not be read (they may need access)")
    elif d["advisories"]:
        print(f"\n! upstream has {len(d['advisories'])} security advisories: read them")
    print(f"\nUpstream branches ahead of {upstream_ref()}: {len(d['branches'])}")
    for b in d["branches"]:
        print(f"  {'C' if b['critical'] else ' '} {b['branch']}: {b['commits']} commits, "
              f"last: {b['last']}")
    return 0


def code(text: str) -> str:
    """Inline code for Markdown: no link, no mention, whatever the text holds."""
    return "`" + text.replace("`", "'").replace("\n", " ").replace("|", "\\|") + "`"


def cmd_report(a) -> int:
    ctx = context()
    todo = sorted_commits(ctx, False)
    d = watch_data()
    crit = [c for c in todo if "C" in c.flags]
    lines = [
        f"Upstream ({code(UPSTREAM_SLUG)}, {code(UPSTREAM_BRANCH)}) since the fork point "
        f"{code(ctx.fork_point[:12])}: **{len(todo)}** commits to look at, **{len(crit)}** "
        f"critical-looking. Updated {dt.datetime.now(dt.timezone.utc):%Y-%m-%d %H:%M} UTC; run "
        "`/upstream-sync` to act on it. Kinds and flags: `tools/upstream-sync/sync.py`.", "",
        "| # | sha | kind | flags | subject |", "|---:|---|---|---|---|",
    ]
    rows = sorted(todo, key=lambda c: ("C" not in c.flags, c.index))
    shown = 0
    for c in rows[:REPORT_ROWS]:
        line = (f"| {c.index or '-'} | {code(c.sha[:12])} | {c.kind} | {c.flags} | "
                f"{code(c.subject[:200])} |")
        if sum(len(x) + 1 for x in lines) + len(line) > REPORT_CHARS * 3 // 4:
            break
        lines.append(line)
        shown += 1
    if len(rows) > shown:
        lines.append(f"| | | | | …and {len(rows) - shown} more: `sync.py status` |")
    if ctx.t.warnings:
        lines += ["", "Warnings:"] + [f"- {code(w[:300])}" for w in ctx.t.warnings[:20]]
        if len(ctx.t.warnings) > 20:
            lines.append(f"- …and {len(ctx.t.warnings) - 20} more: `sync.py status`")
    if d["prs"] is None:
        lines += ["", "**Upstream's open PRs could not be read** (gh missing or refused)."]
    else:
        crit_prs = [p for p in d["prs"] if p["critical"]]
        lines += ["", f"Upstream's open PRs that look critical (not merged there yet): "
                  f"{len(crit_prs)}"]
        lines += [f"- {code(str(p['number']) + ' ' + p['title'])}" for p in crit_prs]
    if not isinstance(d["advisories"], list):
        lines += ["", "Upstream's security advisories could not be read (they may need access)."]
    elif d["advisories"]:
        lines += ["", f"**Upstream has {len(d['advisories'])} security advisories.**"]
    text = "\n".join(lines)
    if len(text) > REPORT_CHARS:
        text = text[:REPORT_CHARS].rsplit("\n", 1)[0] + "\n\n…cut to fit an issue: `sync.py status`"
    print(text)
    return 0


def staged() -> list[str]:
    return zlist("diff", "--cached", "--name-only", "--no-renames", "-z", "HEAD")


def unstaged(ignore: tuple[str, ...] = ()) -> list[str]:
    return [p for p in zlist("diff", "--name-only", "--no-renames", "-z") if p not in ignore]


def undo_pick() -> None:
    if git_path("CHERRY_PICK_HEAD").exists():
        git("cherry-pick", "--abort", check=False)
    git("reset", "--merge", "HEAD", check=False)


def neutralise(msg: str) -> str:
    """Upstream's message, made safe to carry: its own markers can't count as this fork's
    record, and its #numbers can't link to this fork's issues and PRs."""
    msg = MARKERS.sub("upstream's text: ", msg)
    msg = MENTION.sub("anefzaoui", msg)
    msg = re.sub(r"(https?://)?(www\.)?github\.com/anefzaoui/misterdownloadmanager/"
                 r"(issues|pull|discussions)/(\d+)\S*", r"upstream \3 \4", msg, flags=re.I)
    msg = re.sub(r"anefzaoui/misterdownloadmanager#(\d+)", r"upstream issue \1", msg, flags=re.I)
    msg = re.sub(r"\bGH-(\d+)\b", r"upstream \1", msg, flags=re.I)
    return ISSUE.sub(lambda m: f"upstream {m.group(1)}", msg)


def lock_bumps(sha: str) -> list[tuple[str, str, str]]:
    """(name, old version, new version) of the packages upstream's Cargo.lock bumped."""
    def packages(rev: str) -> dict[str, set[str]]:
        text = git("show", f"{rev}:Cargo.lock", check=False).stdout.replace("\r", "")
        res: dict[str, set[str]] = {}
        for m in re.finditer(r'name = "([^"]+)"\nversion = "([^"]+)"', text):
            res.setdefault(m.group(1), set()).add(m.group(2))
        return res
    before, after = packages(f"{sha}^"), packages(sha)
    bumps = []
    for name, vers in after.items():
        old, new = before.get(name, set()) - vers, vers - before.get(name, set())
        if len(old) == 1 and len(new) == 1:
            bumps.append((name, old.pop(), new.pop()))
    return bumps


def apply_lock(sha: str) -> str | None:
    """The fork's Cargo.lock kept, then upstream's version bumps made in it. None when done."""
    if not shutil.which("cargo"):
        return "cargo is not on PATH"
    if any(CARGO_CONFIG.search(p) for _, a, b in changes(sha) for p in (a, b) if p):
        return "it changes cargo's own configuration: port it by hand after reading it"
    git("checkout", "HEAD", "--", "Cargo.lock")
    cmds = [["cargo", "metadata", "--format-version", "1"]]
    cmds += [["cargo", "update", "-p", f"{n}@{o}", "--precise", v] for n, o, v in lock_bumps(sha)]
    for cmd in cmds:
        r = subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace",
                           cwd=root())
        if r.returncode != 0 and "did not match any packages" not in r.stderr:
            return f"`{' '.join(cmd)}` failed: {r.stderr.strip()[-300:]}"
    git("add", "--", "Cargo.lock")
    return None


def pick_one(sha: str, ctx: Context, commits: list[Commit], messages: dict[str, str],
             a) -> tuple[bool, str]:
    """One upstream commit onto HEAD, with the fork's rules. (taken?, what happened)."""
    status = ctx.t.of(sha)
    if status in DONE:
        return True, f"{sha[:12]} already {status}: not picked again"
    if status == "in-pr":
        return False, f"{sha[:12]} is picked on another sync branch whose PR is open: not again"
    if status == "in-branch":
        return False, (f"{sha[:12]} is picked on another local sync branch: finish that branch, "
                       "or delete it, first")
    c = next(x for x in commits if x.sha == sha)
    c = Commit(c.sha, c.index, c.date, c.subject)
    sort_commit(c, "HEAD", ctx.fork, ctx.fork_point, ctx.t, messages)
    if c.kind in ("nothing", "removed"):
        return False, (f"{sha[:12]} is {c.kind}: nothing to take; "
                       f"`record {sha[:12]} skipped --note \"…\"`")
    if c.kind == "port":
        return False, f"{sha[:12]} needs a port: " + "; ".join(
            ([f"conflicts in {', '.join(c.conflicts)}"] if c.conflicts else []) + c.notes)
    if c.sensitive and not a.allow_sensitive:
        return False, (f"{sha[:12]} is sensitive ({', '.join(c.sensitive)}): read every line, "
                       "then pick it alone with --allow-sensitive")
    if c.kind == "lock" and not a.lock:
        return False, f"{sha[:12]} conflicts only in Cargo.lock: pick it with --lock"
    gone = set(c.left_out) - set(OWN)
    r = git("cherry-pick", "--no-commit", "-X", RENAMES, sha, check=False)
    conflicted = set(zlist("diff", "--name-only", "--diff-filter=U", "-z"))
    if r.returncode != 0 and not conflicted:
        undo_pick()
        return False, f"{sha[:12]}: git refused the pick ({r.stderr.strip()}); nothing changed"
    left_out = []
    for path in sorted(set(staged()) | conflicted):
        if path in OWN:
            if git("cat-file", "-e", f"HEAD:{path}", check=False).returncode == 0:
                git("checkout", "HEAD", "--", path)
            else:
                git("rm", "-f", "--quiet", "--", path)
            left_out.append(path)
        elif path in gone:
            git("rm", "-f", "--quiet", "--", path)
            left_out.append(path)
    if c.kind == "lock" and "Cargo.lock" in conflicted:
        problem = apply_lock(sha)
        if problem:
            undo_pick()
            git("checkout", "HEAD", "--", "Cargo.lock", check=False)
            return False, f"{sha[:12]}: Cargo.lock not made ({problem}); nothing changed"
    still = zlist("diff", "--name-only", "--diff-filter=U", "-z")
    if still:
        undo_pick()
        return False, f"{sha[:12]} needs a port: conflicts in {', '.join(still)} (nothing changed)"
    if not staged():
        undo_pick()
        return False, (f"{sha[:12]}: nothing left to commit. Already in the fork? "
                       f"`record {sha[:12]} covered --ref <commit>`; otherwise skipped")
    body = neutralise(messages.get(sha) or out("log", "-1", "--format=%B", sha)).rstrip()
    if left_out:
        body += "\n\nLeft out here (removed in this fork, or the fork's own record):\n"
        body += "".join(f"- {p}\n" for p in left_out)
    if c.kind == "lock":
        body += "\nCargo.lock: the fork's, with upstream's version bumps made in it.\n"
    body = body.rstrip() + f"\n\nUpstream-Pick: {sha}\n"
    msg_file = git_path("UPSTREAM_SYNC_MSG")
    msg_file.write_text(body, encoding="utf-8", newline="\n")
    try:
        r = git("commit", "--quiet", "--cleanup=whitespace", "-F", str(msg_file),
                "--author=" + out("log", "-1", "--format=%an <%ae>", sha),
                "--date=" + out("log", "-1", "--format=%aI", sha), check=False)
    finally:
        msg_file.unlink(missing_ok=True)
    if r.returncode != 0:
        undo_pick()
        return False, f"{sha[:12]}: not committed ({r.stderr.strip()}); nothing changed"
    new = out("rev-parse", "--short=12", "HEAD")
    write_row(sha, "picked", new, f"left out {len(left_out)} file(s)" if left_out else "")
    ctx.t.status[sha] = "picked"
    extra = f", left out {len(left_out)} file(s)" if left_out else ""
    warn = "".join(f"\n    ! {n}" for n in c.notes)
    return True, f"{sha[:12]} picked as {new}{extra}{warn}"


def guard_sync_branch() -> str:
    branch = current_branch()
    if not branch:
        raise Fail("HEAD is detached: work on a sync branch (upstream-sync/<date>)")
    if not branch.startswith(SYNC_PREFIX):
        raise Fail(f"on {branch}: picks and PRs only on a branch named {SYNC_PREFIX}…, "
                   "in the run's own worktree")
    merge = git("config", "--get", f"branch.{branch}.merge", check=False).stdout.strip()
    if merge in {f"refs/heads/{b}" for b in PROTECTED}:
        raise Fail(f"{branch} tracks {merge}: a push from an editor would land on it. "
                   f"`git branch --unset-upstream {branch}` first")
    return branch


def cmd_pick(a) -> int:
    guard_sync_branch()
    for state in ("CHERRY_PICK_HEAD", "MERGE_HEAD", "REVERT_HEAD", "rebase-merge", "rebase-apply"):
        if git_path(state).exists():
            raise Fail("a cherry-pick, merge, revert or rebase is in progress: finish or abort it")
    if staged():
        raise Fail("staged changes (" + ", ".join(staged()) + "): commit or unstage them; a "
                   "pick interrupted half way is undone with `git reset --merge HEAD`")
    dirty = unstaged((LEDGER,))
    if dirty:
        raise Fail("uncommitted changes (" + ", ".join(dirty) + "): commit or stash them first "
                   "(only the ledger may stay uncommitted)")
    if a.allow_sensitive and len(set(a.sha)) > 1:
        raise Fail("--allow-sensitive takes one commit, read line by line: never a batch")
    ctx = context()
    shas = []
    for s in a.sha:
        full = full_sha(s)
        if not full or full not in ctx.in_range:
            raise Fail(f"{s} is not an upstream commit since the fork point")
        if full not in shas:
            shas.append(full)
    commits, messages = pending(ctx.fork_point, ctx.up)
    order = [c.sha for c in commits]
    for s in shas:
        if s not in order:
            raise Fail(f"{s[:12]} is an upstream merge: read it (`show`), port what's wanted")
    shas.sort(key=order.index)  # upstream's order: a fix can build on the one before
    ledger = root() / LEDGER
    failed = 0
    for sha in shas:
        saved = ledger.read_bytes() if ledger.exists() else None
        try:
            ok, line = pick_one(sha, ctx, commits, messages, a)
        except (Fail, OSError) as e:
            undo_pick()
            ok, line = False, f"{sha[:12]}: stopped half way ({e}); undone, nothing changed"
        if saved is not None and not ok:
            ledger.write_bytes(saved)  # an undone pick never takes the ledger's rows with it
        print(line)
        if not ok:
            failed += 1
            if not a.keep_going:
                print("Stopped: later picks may build on this one (--keep-going to go on).")
                return 1
    return 1 if failed else 0


def write_row(sha: str, status: str, ref: str, note: str) -> str:
    """The commit's row, replacing every earlier one for it, at the end of the ledger."""
    path = root() / LEDGER
    path.parent.mkdir(parents=True, exist_ok=True)
    lines = path.read_text(encoding="utf-8").splitlines() if path.exists() else []
    kept = []
    for ln in lines:
        m = ROW.match(ln)
        if m and sha.startswith(m.group(1).lower()):
            continue
        kept.append(ln)
    subject = out("log", "-1", "--format=%s", sha)
    cells = [sha, status, today(), ref, subject, note]
    row = "| " + " | ".join(c.replace("|", "/").replace("\n", " ").strip() for c in cells) + " |"
    kept.append(row)
    path.write_text("\n".join(kept) + "\n", encoding="utf-8", newline="\n")
    return row


def cmd_record(a) -> int:
    ctx = context()
    sha = full_sha(a.sha)
    if not sha or sha not in ctx.in_range:
        raise Fail(f"{a.sha} is not an upstream commit since the fork point")
    if not a.note.strip():
        raise Fail("--note is required: say why, for whoever reads the ledger next")
    if a.status in ("covered", "ported") and not a.ref:
        raise Fail(f"{a.status} needs --ref: the fork's commit or PR that has the fix")
    print(write_row(sha, a.status, a.ref or "", a.note))
    return 0


def sync_branches() -> list[str]:
    """What earlier runs left: sync branches not in dev, by their PR's state when gh can tell."""
    forkref = fork_ref()
    if not has_ref(forkref):
        return []
    notes = []
    target = slug(remote_url(FORK_REMOTE))
    for ref in out("for-each-ref", "--format=%(refname:short)",
                   f"refs/remotes/{FORK_REMOTE}/{SYNC_PREFIX}").splitlines():
        if git("merge-base", "--is-ancestor", ref, forkref, check=False).returncode == 0:
            continue
        branch = ref.split("/", 1)[1]
        prs = gh_json(["pr", "list", "-R", target, "--state", "all", "--head", branch,
                       "--json", "number,state"]) if target else None
        if prs is None:
            notes.append(f"{branch} is not in {forkref} (gh can't tell its PR's state)")
        elif not prs:
            notes.append(f"{branch} has no PR: open it with `sync.py pr`, or delete the branch")
        elif prs[0]["state"] == "MERGED":
            continue  # squash- or rebase-merged: its ledger rows are what count
        elif prs[0]["state"] == "CLOSED":
            notes.append(f"{branch}: PR {prs[0]['number']} closed unmerged; its picks and rows "
                         "never landed: delete the branch and decide its commits again")
        else:
            notes.append(f"{branch}: PR {prs[0]['number']} open; its commits show here until "
                         "it merges (don't pick them again elsewhere)")
    return notes


def fork_slug() -> str:
    for push in (False, True):
        s = slug(remote_url(FORK_REMOTE, push=push))
        if not s:
            raise Fail(f"{FORK_REMOTE}'s {'push' if push else 'fetch'} address is not GitHub")
        if s == UPSTREAM_SLUG:
            raise Fail(f"{FORK_REMOTE} {'pushes' if push else 'points'} to the original "
                       f"({UPSTREAM_SLUG}): never push there")
    return slug(remote_url(FORK_REMOTE))


def cmd_pr(a) -> int:
    branch = guard_sync_branch()
    target = fork_slug()
    if not shutil.which("gh"):
        raise Fail("gh is not installed: nothing pushed")
    body = Path(a.body_file)
    if not body.is_file():
        raise Fail(f"no body file {a.body_file}: nothing pushed")
    try:
        text = body.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        raise Fail(f"{a.body_file} is not UTF-8 (PowerShell 5.1's `>` writes UTF-16: use "
                   "`Out-File -Encoding utf8`); nothing pushed")
    for t in (text, a.title):
        m = UPSTREAM_LINK.search(t) or MENTION.search(t)
        if m:
            raise Fail(f"{m.group(0)!r} would link to, and so mention this PR on, the original: "
                       "write \"upstream issue N\" instead; nothing pushed")
    if staged() or unstaged():
        raise Fail("uncommitted changes: commit them (the ledger too) before the PR")
    if has_ref(fork_ref()):
        m = UPSTREAM_LINK.search(out("log", "--format=%B", f"{fork_ref()}..HEAD"))
        if m:
            raise Fail(f"a commit on this branch says {m.group(0)!r}, which would mention this "
                       "fork on the original: reword it (`upstream issue N`); nothing pushed")
    existing = gh_json(["pr", "list", "-R", target, "--state", "open", "--head", branch,
                        "--json", "number,url"])
    push = ["push", "-u", FORK_REMOTE, f"HEAD:refs/heads/{branch}"]
    if a.force:
        push.insert(1, "--force-with-lease")
    git(*push)
    if existing:
        print(f"pushed; its PR is open already: {existing[0]['url']}")
        return 0
    # -R and --base always: in a fork, gh's default for a new PR is the parent repository.
    args = ["gh", "pr", "create", "-R", target, "--base", FORK_BASE, "--head", branch,
            "--title", a.title, "--body-file", str(body)]
    if a.draft:
        args.append("--draft")
    try:
        return subprocess.run(args).returncode
    except OSError as e:
        raise Fail(f"gh failed ({e}); the branch is pushed: open the PR against "
                   f"{target} {FORK_BASE} (not the original)")


def checks() -> list[tuple[bool, str, list[str] | None]]:
    """(ok, what, the git command that fixes it, if one does)."""
    c = []
    up_url = remote_url(UPSTREAM_REMOTE)
    want = f"https://github.com/{UPSTREAM_SLUG}.git"
    c.append((slug(up_url) == UPSTREAM_SLUG, f"{UPSTREAM_REMOTE} fetches {UPSTREAM_SLUG}",
              ["remote", "add" if not up_url else "set-url", UPSTREAM_REMOTE, want]))
    push = remote_url(UPSTREAM_REMOTE, push=True)
    c.append((bool(up_url) and push == "DISABLED", f"{UPSTREAM_REMOTE} cannot be pushed to",
              ["remote", "set-url", "--push", UPSTREAM_REMOTE, "DISABLED"]))
    tag = git("config", "--get", f"remote.{UPSTREAM_REMOTE}.tagOpt", check=False).stdout.strip()
    c.append((tag == "--no-tags", f"{UPSTREAM_REMOTE}'s tags stay out (they'd clash with ours, "
              "and a v* tag pushed here starts a release)",
              ["config", f"remote.{UPSTREAM_REMOTE}.tagOpt", "--no-tags"]))
    for push_side in (False, True):
        s = slug(remote_url(FORK_REMOTE, push=push_side))
        c.append((bool(s) and s != UPSTREAM_SLUG,
                  f"{FORK_REMOTE} {'pushes to' if push_side else 'is'} this fork "
                  f"({s or 'missing'}), not the original", None))
    gh = git("config", "--get", f"remote.{FORK_REMOTE}.gh-resolved", check=False).stdout.strip()
    c.append((gh == "base", f"gh's default repository is {FORK_REMOTE} (not the parent)",
              ["config", f"remote.{FORK_REMOTE}.gh-resolved", "base"]))
    pd = git("config", "--get", "remote.pushDefault", check=False).stdout.strip()
    c.append((pd != UPSTREAM_REMOTE, f"remote.pushDefault is not {UPSTREAM_REMOTE}",
              ["config", "--unset", "remote.pushDefault"]))
    for line in out("for-each-ref", "--format=%(refname:short) %(upstream)",
                    f"refs/heads/{SYNC_PREFIX}").splitlines():
        b, _, up = line.partition(" ")
        if up in {f"refs/remotes/{FORK_REMOTE}/{p}" for p in PROTECTED}:
            c.append((False, f"{b} tracks {up} (an editor's push would land there)",
                      ["branch", "--unset-upstream", b]))
    h = git("merge-tree", "-h", check=False)
    usage = h.stdout + h.stderr
    c.append(("merge-base <" in usage and "strategy-option" in usage,
              "git's merge-tree takes --merge-base and -X (a recent git)", None))
    c.append((has_ref(upstream_ref()), f"{upstream_ref()} fetched",
              ["fetch", UPSTREAM_REMOTE, "--no-tags"]))
    return c


def cmd_doctor(a) -> int:
    bad = 0
    for ok, what, fix in checks():
        if not ok and a.fix and fix:
            git(*fix)
            ok, what = True, what + " (fixed)"
        print(("ok    " if ok else "FAIL  ") + what)
        bad += not ok
    if not shutil.which("gh"):
        print("note  gh is not installed: `watch` and `pr` need it")
    for s in sync_branches():
        print(f"note  {s}")
    for b in out("for-each-ref", "--format=%(refname:short)",
                 f"refs/heads/{SYNC_PREFIX}").splitlines():
        print(f"note  local sync branch {b}")
    for line in out("worktree", "list", "--porcelain").splitlines():
        if line.startswith("branch refs/heads/" + SYNC_PREFIX):
            print(f"note  a worktree has {line.split(' ', 1)[1][len('refs/heads/'):]} checked out")
    _, problems = read_ledger(root() / LEDGER)
    for p in problems:
        print(f"note  {p}")
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
    sh = sub.add_parser("show")
    sh.add_argument("sha")
    w = sub.add_parser("watch")
    w.add_argument("--json", action="store_true")
    sub.add_parser("report")
    k = sub.add_parser("pick")
    k.add_argument("sha", nargs="+")
    k.add_argument("--keep-going", action="store_true")
    k.add_argument("--lock", action="store_true", help="make upstream's Cargo.lock bumps (cargo)")
    k.add_argument("--allow-sensitive", action="store_true", help="after reading every line")
    r = sub.add_parser("record")
    r.add_argument("sha")
    r.add_argument("status", choices=RECORDABLE)
    r.add_argument("--note", required=True)
    r.add_argument("--ref", help="the fork's commit or PR, for ported and covered")
    pr = sub.add_parser("pr")
    pr.add_argument("--title", required=True)
    pr.add_argument("--body-file", required=True)
    pr.add_argument("--draft", action="store_true")
    pr.add_argument("--force", action="store_true", help="--force-with-lease, this branch only")
    a = p.parse_args(argv)
    try:
        os.chdir(root())  # pathspecs are relative to the current folder; ours are the repo's
        return {"doctor": cmd_doctor, "status": cmd_status, "show": cmd_show, "watch": cmd_watch,
                "report": cmd_report, "pick": cmd_pick, "record": cmd_record, "pr": cmd_pr}[a.cmd](a)
    except Fail as e:
        print(f"error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
