#!/usr/bin/env python3
"""sync.py against a throwaway repository: a fork that removed a platform and a language, moved a
folder and changed files, and an upstream with a commit of every kind and every trap the audits
found. `python tools/upstream-sync/test_sync.py` (needs git 2.40+; gh only for the PR checks).
"""
from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

SYNC = Path(__file__).with_name("sync.py")
LEDGER = "tools/upstream-sync/ledger.md"
HEADER = "| upstream | status | date | ref | subject | note |\n|---|---|---|---|---|---|\n"


def run(cwd: Path, *args: str, ok: bool = True) -> subprocess.CompletedProcess:
    r = subprocess.run(args, cwd=cwd, capture_output=True, text=True, encoding="utf-8")
    if ok and r.returncode != 0:
        raise AssertionError(f"{args}: {r.returncode}\n{r.stdout}\n{r.stderr}")
    return r


def git(cwd: Path, *args: str) -> str:
    return run(cwd, "git", "-c", "core.quotepath=off", *args).stdout.strip()


def sync(cwd: Path, *args: str, ok: bool = True, online: bool = False):
    env = None if online else {**os.environ, "UPSTREAM_SYNC_OFFLINE": "1"}
    r = subprocess.run([sys.executable, str(SYNC), *args], cwd=cwd, capture_output=True,
                       text=True, encoding="utf-8", env=env)
    if ok and r.returncode != 0:
        raise AssertionError(f"sync {args}: {r.returncode}\n{r.stdout}\n{r.stderr}")
    return r


def status(cwd: Path) -> dict:
    d = json.loads(sync(cwd, "status", "--json").stdout)
    d["by"] = {c["sha"]: c for c in d["commits"]}
    return d


def write(repo: Path, path: str, text: str) -> None:
    p = repo / path
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(text, encoding="utf-8", newline="\n")


def commit(repo: Path, msg: str) -> str:
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", msg)
    return git(repo, "rev-parse", "HEAD")


def clean(repo: Path) -> bool:
    return git(repo, "status", "--porcelain", "--untracked-files=no") == ""


LINES = "".join(f"line {i}\n" for i in range(1, 30))


def edit(text: str, n: int, new: str) -> str:
    return text.replace(f"line {n}\n", new + "\n", 1)


def main() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        r = Path(tmp)
        git(r, "init", "-q", "-b", "main")
        for k, v in {"user.name": "T", "user.email": "t@example.com", "commit.gpgsign": "false",
                     "core.autocrlf": "false", "protocol.https.allow": "never"}.items():
            git(r, "config", k, v)
        for f in ("a.rs", "b.rs", "img1.txt", "img[1].txt", "old/x.rs"):
            write(r, f, LINES)
        write(r, "keep/k.rs", LINES)
        write(r, "plat/mac.rs", "mac\n")
        write(r, "loc/fr.txt", "bonjour\n")
        write(r, "Cargo.lock", 'name = "dep"\nversion = "1.0.0"\n')
        write(r, "CHANGELOG.md", "# Changelog\n\n- old\n")
        write(r, "ROADMAP.md", "# Roadmap\n")
        write(r, ".claude/settings.json", "{}\n")
        write(r, "CLAUDE.md", "# rules\n")
        write(r, "z.rs", LINES)
        base = commit(r, "base")

        # The fork: a platform and a language removed, a file of its own changed, one with a
        # glob-like name deleted, a folder moved, the ledger started.
        git(r, "checkout", "-q", "-b", "dev")
        for f in ("plat/mac.rs", "loc/fr.txt", "img[1].txt"):
            (r / f).unlink()
        git(r, "add", "-A")
        git(r, "mv", "old", "new")
        write(r, "a.rs", edit(LINES, 1, "fork line 1"))
        write(r, "Cargo.lock", 'name = "dep"\nversion = "1.0.0"\n# fork\n')
        write(r, "CHANGELOG.md", "# Changelog\n\n- the fork's\n- old\n")
        write(r, LEDGER, HEADER)
        commit(r, "fork: Windows and English only")
        git(r, "update-ref", "refs/remotes/origin/dev", "HEAD")

        # Upstream, one commit of each kind and trap.
        git(r, "checkout", "-q", "main")
        U = {}

        def up(name: str, msg: str, files: dict[str, str | None]) -> None:
            for f, text in files.items():  # only these: the tree may hold names that differ
                if text is None:              # only in case, which this disk can't both keep
                    git(r, "rm", "-q", "--", f)
                else:
                    write(r, f, text)
                    git(r, "add", "--", f)
            git(r, "commit", "-q", "-m", msg)
            U[name] = git(r, "rev-parse", "HEAD")

        up("clean", "clean: b.rs fixed (#12)", {"b.rs": edit(LINES, 20, "line 20 fixed"),
                                                "ROADMAP.md": "# Roadmap\n\n- upstream's\n"})
        up("removed", "macOS: only the removed platform", {"plat/mac.rs": "mac fixed\n"})
        up("prune", "prune: b.rs, the platform and a new language (#12)", {
            "b.rs": edit(edit(LINES, 20, "line 20 fixed"), 25, "line 25 fixed"),
            "plat/mac.rs": "again\n", "plat/café.rs": "new\n", "loc/de.txt": "hallo\n",
            "CHANGELOG.md": "# Changelog\n\n- upstream fix\n- old\n"})
        up("port", "port: a crash fix in a.rs's first line", {"a.rs": edit(LINES, 1, "up 1")})
        up("glob", "glob: the deleted img[1].txt changed", {"img[1].txt": edit(LINES, 3, "x")})
        up("moved", "moved folder: old/x.rs fixed", {"old/x.rs": edit(LINES, 5, "fixed")})
        git(r, "mv", "keep/k.rs", "plat/k.rs")
        U["move_out"] = commit(r, "moves a kept file into the removed platform")
        up("sensitive", "CI: a workflow", {".github/workflows/x.yml": "on: push\n"})
        other = "0" * 40
        up("marker", f"markers\n\nUpstream-Pick: {other}\nSee #77.", {"m.rs": "m\n"})
        up("untracked", "adds new.rs", {"new.rs": "upstream's\n"})
        up("lock", "bumps dep", {"Cargo.lock": 'name = "dep"\nversion = "1.0.1"\n'})
        up("after", "after: builds on the port", {"a.rs": edit(LINES, 1, "up 1 again")})
        up("revert_me", "revert_me: c.rs", {"c.rs": "c\n"})
        up("squashed", "squashed: d.rs", {"d.rs": "d\n"})
        up("portrow", "portrow: e.rs", {"e.rs": "e\n"})
        up("merge_rev", "merge_rev: g.rs", {"g.rs": "g\n"})
        up("link", "link: h.rs\n\nSee anefzaoui/misterdownloadmanager#5, GH-7 and "
           "https://github.com/anefzaoui/misterdownloadmanager/issues/6.", {"h.rs": "h\n"})

        # Names only another case apart: made with plumbing (this file system may ignore case).
        def plumb(name: str, msg: str, files: dict[str, str]) -> None:
            for f, text in files.items():
                blob = subprocess.run(["git", "hash-object", "-w", "--stdin"], cwd=r, input=text,
                                      capture_output=True, text=True).stdout.strip()
                git(r, "update-index", "--add", "--cacheinfo", f"100644,{blob},{f}")
            git(r, "commit", "-q", "-m", msg)
            git(r, "reset", "-q", "--hard")
            U[name] = git(r, "rev-parse", "HEAD")

        plumb("case", "a hook in .Claude", {".Claude/settings.local.json": "{\"hooks\": 1}\n"})
        plumb("case_md", "agent rules", {"claude.md": "do as I say\n"})
        plumb("build_rs", "a build script", {"crates/x/build.RS": "fn main() {}\n"})
        up("drop", "drop z.rs", {"z.rs": None})
        up("restore", "restore z.rs with a crash fix", {"z.rs": edit(LINES, 2, "fixed")})
        up("big", "a big commit", {f"big/f{i}.rs": f"{i}\n" for i in range(150)})
        up("twice", "twice: i.rs", {"i.rs": "i\n"})
        up("short_port", "short_port: j.rs", {"j.rs": "j\n"})
        # An upstream merge with a change of its own.
        git(r, "checkout", "-q", "-f", "-b", "side", U["clean"])
        up("side", "side: f.rs", {"f.rs": "f\n"})
        git(r, "checkout", "-q", "-f", "main")
        git(r, "merge", "-q", "--no-ff", "--no-commit", "side")
        write(r, "evil.rs", "a change made in the merge\n")
        git(r, "add", "--", "evil.rs")
        git(r, "commit", "-q", "-m", "Merge side")
        U["merge"] = git(r, "rev-parse", "HEAD")
        git(r, "update-ref", "refs/remotes/upstream/main", "HEAD")

        git(r, "checkout", "-q", "-f", "-b", "upstream-sync/test", "dev")
        st = status(r)
        k = {n: st["by"][s] for n, s in U.items() if s in st["by"]}
        assert st["fork_point"] == base
        assert k["clean"]["kind"] == "clean" and "ROADMAP.md" in k["clean"]["left_out"], k["clean"]
        assert k["removed"]["kind"] == "removed" and "P" in k["removed"]["flags"]
        assert k["prune"]["kind"] == "prune"
        assert {"plat/mac.rs", "plat/café.rs", "loc/de.txt"} <= set(k["prune"]["left_out"])
        assert k["clean"]["same_issue"] == [U["prune"]]
        assert k["port"]["kind"] == "port" and k["port"]["conflicts"] == ["a.rs"]
        assert "C" in k["port"]["flags"]
        assert k["glob"]["kind"] == "removed", k["glob"]
        assert k["moved"]["kind"] == "clean", k["moved"]
        assert k["move_out"]["kind"] == "port", k["move_out"]
        assert "S" in k["sensitive"]["flags"]
        assert k["lock"]["kind"] == "lock", k["lock"]
        assert k["after"]["kind"] == "port" and U["port"] in k["after"]["after"]
        for n in ("case", "case_md"):
            assert k[n]["kind"] == "port" and "S" in k[n]["flags"], k[n]
        assert "S" in k["build_rs"]["flags"], k["build_rs"]
        assert k["restore"]["kind"] != "removed" and k["drop"]["kind"] == "clean", (k["restore"], k["drop"])
        assert k["big"]["kind"] == "clean"
        merges = [c for c in st["commits"] if c["kind"] == "merge"]
        assert [c["sha"] for c in merges] == [U["merge"]], merges

        # Picks: only on a sync branch that doesn't track dev; in upstream's order.
        git(r, "checkout", "-q", "dev")
        assert "only on a branch named" in sync(r, "pick", U["clean"], ok=False).stderr
        git(r, "checkout", "-q", "-b", "upstream-sync/tracking", "dev")
        git(r, "config", "branch.upstream-sync/tracking.merge", "refs/heads/dev")
        assert "tracks refs/heads/dev" in sync(r, "pick", U["clean"], ok=False).stderr
        git(r, "checkout", "-q", "upstream-sync/test")

        out = sync(r, "pick", U["prune"], U["clean"]).stdout
        assert out.index(U["clean"][:12]) < out.index(U["prune"][:12]), out
        for f in ("plat/mac.rs", "plat/café.rs", "loc/de.txt"):
            assert not (r / f).exists(), f
        assert "line 25 fixed" in (r / "b.rs").read_text(encoding="utf-8")
        assert (r / "ROADMAP.md").read_text(encoding="utf-8") == "# Roadmap\n"
        assert (r / "CHANGELOG.md").read_text(encoding="utf-8").startswith("# Changelog\n\n- the")
        log = git(r, "log", "--format=%B", "-2")
        assert f"Upstream-Pick: {U['clean']}" in log and f"Upstream-Pick: {U['prune']}" in log
        assert "(upstream 12)" in log and "#12" not in log, log
        assert git(r, "log", "-1", "--format=%an", "HEAD") == "T"
        ledger = (r / LEDGER).read_text(encoding="utf-8")
        assert f"| {U['clean']} | picked |" in ledger and f"| {U['prune']} | picked |" in ledger

        # The unstaged ledger (tracked, edited by the picks) doesn't block the next pick.
        assert not clean(r) and git(r, "diff", "--name-only") == LEDGER
        assert "already picked" in sync(r, "pick", U["clean"]).stdout
        assert "picked as" in sync(r, "pick", U["moved"]).stdout
        assert "line 5 fixed" not in (r / "new/x.rs").read_text(encoding="utf-8")
        assert "fixed" in (r / "new/x.rs").read_text(encoding="utf-8")
        assert not (r / "old").exists()

        # A staged change blocks; a pick that fails keeps the ledger's unstaged rows.
        git(r, "add", LEDGER)
        assert "staged changes" in sync(r, "pick", U["port"], ok=False).stderr
        git(r, "reset", "-q", "--", LEDGER)
        before = (r / LEDGER).read_text(encoding="utf-8")
        head = git(r, "rev-parse", "HEAD")
        res = sync(r, "pick", U["port"], ok=False)
        assert "needs a port" in res.stdout, res.stdout
        assert git(r, "rev-parse", "HEAD") == head
        assert (r / LEDGER).read_text(encoding="utf-8") == before
        assert git(r, "diff", "--cached", "--name-only") == ""

        # The traps: a glob-like name, a move out of what the fork keeps, an untracked file in
        # the way, a sensitive file, a lock-only conflict: none changes anything it shouldn't.
        assert "nothing to take" in sync(r, "pick", U["glob"], ok=False).stdout
        assert (r / "img1.txt").exists()
        res = sync(r, "pick", U["move_out"], ok=False)
        assert "delete the fork's file" in res.stdout and (r / "keep/k.rs").exists(), res.stdout
        write(r, "new.rs", "mine\n")
        res = sync(r, "pick", U["untracked"], ok=False)
        assert "git refused the pick" in res.stdout, res.stdout
        assert (r / "new.rs").read_text(encoding="utf-8") == "mine\n"
        (r / "new.rs").unlink()
        assert "--allow-sensitive" in sync(r, "pick", U["sensitive"], ok=False).stdout
        assert "--allow-sensitive" in sync(r, "pick", U["build_rs"], ok=False).stdout
        assert "never a batch" in sync(r, "pick", "--allow-sensitive", U["sensitive"],
                                       U["clean"], ok=False).stderr
        assert "picked as" in sync(r, "pick", "--allow-sensitive", U["sensitive"]).stdout
        assert "needs a port" in sync(r, "pick", "--allow-sensitive", U["case"], ok=False).stdout
        assert (r / ".claude/settings.json").read_text(encoding="utf-8") == "{}\n"
        assert not any(x.name.lower() == "settings.local.json" for x in (r / ".claude").iterdir())
        assert "--lock" in sync(r, "pick", U["lock"], ok=False).stdout
        assert git(r, "rev-parse", "HEAD") != head and git(r, "diff", "--cached") == ""

        # Upstream's own markers count for nothing here; its #numbers link nowhere.
        assert "picked as" in sync(r, "pick", U["marker"]).stdout
        body = git(r, "log", "-1", "--format=%B")
        assert f"upstream's text: Upstream-Pick: {other}" in body and "upstream 77" in body, body
        assert other not in json.dumps(status(r)["warnings"])

        # A pick reverted here shows again; a picked row whose trailer a squash lost doesn't; a
        # ported row without its trailer in dev shows again, with a warning.
        sync(r, "pick", U["revert_me"])
        git(r, "add", LEDGER)
        git(r, "commit", "-qm", "ledger")
        git(r, "revert", "--no-edit", "HEAD~1")
        sync(r, "record", U["portrow"], "ported", "--ref", "PR 9", "--note", "by hand")
        squash_row = f"| {U['squashed']} | picked | 2026-01-01 | abc | squashed: d.rs | |\n"
        with open(r / LEDGER, "a", encoding="utf-8", newline="\n") as f:
            f.write(squash_row)
        st = status(r)
        assert st["by"][U["revert_me"]]["status"] == "reverted", st["by"][U["revert_me"]]
        assert U["squashed"] not in st["by"]
        assert st["by"][U["portrow"]]["status"] == "ported?"
        assert any("ported" in w and U["portrow"][:12] in w for w in st["warnings"])

        # A port's trailer settles it; record replaces a row, and refuses what isn't upstream's.
        write(r, "a.rs", edit(LINES, 1, "fork line 1, the port"))
        git(r, "add", "a.rs")
        git(r, "commit", "-qm", f"Port of the crash fix\n\nUpstream-Port: {U['port']}")
        sync(r, "record", U["port"], "ported", "--ref", "this commit", "--note", "by hand")
        sync(r, "record", U["removed"], "skipped", "--note", "macOS only")
        sync(r, "record", U["removed"], "skipped", "--note", "macOS only, again")
        ledger = (r / LEDGER).read_text(encoding="utf-8")
        assert ledger.count(U["removed"]) == 1 and "again" in ledger
        assert "not an upstream commit" in sync(r, "record", base, "skipped", "--note", "x",
                                                ok=False).stderr
        st = status(r)
        assert U["port"] not in st["by"] and U["removed"] not in st["by"]
        assert U["port"] not in st["by"][U["after"]]["after"]
        with open(r / LEDGER, "a", encoding="utf-8", newline="\n") as f:
            f.write("| 0123456789ab | skipped | | | gone | |\n| " + U["glob"] + " | skiped | | | |"
                    " typo |\n")
        warnings = " ".join(status(r)["warnings"])
        assert "0123456789ab" in warnings and "unknown status 'skiped'" in warnings, warnings

        # PRs: never through an origin that is, or pushes to, the original; never a body that
        # would mention the original's issues.
        git(r, "add", "-A")
        git(r, "commit", "-qm", "ledger")
        git(r, "remote", "add", "origin", "https://github.com/someone/fork.git")
        git(r, "remote", "set-url", "--push", "origin",
            "https://github.com/anefzaoui/misterdownloadmanager.git")
        assert "never push there" in sync(r, "pr", "--title", "t", "--body-file", "x",
                                          ok=False).stderr
        git(r, "remote", "set-url", "--push", "origin", "https://github.com/someone/fork.git")
        if shutil.which("gh"):
            for body in ("Fixes anefzaoui/misterdownloadmanager#771.\n",
                         "Fixes \\`anefzaoui/misterdownloadmanager#771\\`.\n"):
                write(r, "body.md", body)
                res = sync(r, "pr", "--title", "t", "--body-file", str(r / "body.md"), ok=False,
                           online=True)
                assert "would link to" in res.stderr, (body, res.stderr)
        assert git(r, "for-each-ref", "refs/remotes/origin/upstream-sync") == ""

        # Links to the original are rewritten in a pick; `show` knows a merge; `report` runs.
        assert "picked as" in sync(r, "pick", U["link"]).stdout
        body = git(r, "log", "-1", "--format=%B")
        assert "anefzaoui" not in body and "upstream issue 5" in body, body
        assert "upstream issues 6" in body and "upstream 7" in body and "GH-7" not in body, body

        # Upstream's own deletion, taken, doesn't make its later fix "removed".
        out = sync(r, "pick", U["drop"], U["restore"]).stdout
        assert out.count("picked as") == 2 and "fixed" in (r / "z.rs").read_text(), out

        # A revert of a revert brings the pick back; a short port trailer counts; a sync branch
        # whose PR's state can't be read warns, and doesn't block.
        sync(r, "pick", U["twice"])
        git(r, "add", LEDGER)
        git(r, "commit", "-qm", "ledger")
        git(r, "revert", "--no-edit", "HEAD~1")
        assert status(r)["by"][U["twice"]]["status"] == "reverted"
        git(r, "revert", "--no-edit", "HEAD")
        assert U["twice"] not in status(r)["by"]
        write(r, "j.rs", "j, ported\n")
        git(r, "add", "j.rs")
        git(r, "commit", "-qm", f"port\n\nUpstream-Port: {U['short_port'][:12].upper()}")
        assert U["short_port"] not in status(r)["by"]
        stale = run(r, "git", "commit-tree", "dev^{tree}", "-p", "dev", "-m",
                    f"an old run\n\nUpstream-Pick: {U['case']}").stdout.strip()
        git(r, "update-ref", "refs/remotes/origin/upstream-sync/old", stale)
        st = status(r)
        assert st["by"][U["case"]]["status"] is None, st["by"][U["case"]]
        assert any("upstream-sync/old" in w and "can't tell" in w for w in st["warnings"])
        assert json.loads(sync(r, "show", U["merge"]).stdout)["kind"] == "merge"
        rep = sync(r, "report").stdout
        assert "could not be read" in rep and "| 1 |" not in rep.split("\n")[0]
        w = json.loads(sync(r, "watch", "--json").stdout)
        assert w["prs"] is None and w["advisories"] is None
        assert [b["branch"] for b in w["branches"]] == [], w  # side is merged into main
        sync(r, "record", U["after"], "deferred", "--note", "later")
        ledger = (r / LEDGER).read_text(encoding="utf-8")
        today = next(ln for ln in ledger.splitlines() if U["after"] in ln).split("|")[3].strip()
        (r / LEDGER).write_text(ledger.replace(today, "someday"), encoding="utf-8", newline="\n")
        assert any("deferred with no date" in x for x in status(r)["warnings"])
        (r / LEDGER).write_text(ledger.replace(today, "2000-01-01"), encoding="utf-8",
                                newline="\n")
        assert any("deferred" in x and "days ago" in x for x in status(r)["warnings"])
        (r / LEDGER).write_text(ledger, encoding="utf-8", newline="\n")

        # A merged sync PR reverted on dev: its picks show again, as reverted, never as done.
        git(r, "add", "-A")
        git(r, "commit", "-qm", "ledger")
        git(r, "checkout", "-q", "-b", "upstream-sync/pr2", "dev")
        assert "picked as" in sync(r, "pick", U["merge_rev"]).stdout
        git(r, "add", "-A")
        git(r, "commit", "-qm", "ledger")
        git(r, "checkout", "-q", "dev")
        git(r, "merge", "-q", "--no-ff", "-m", "Merge the sync PR", "upstream-sync/pr2")
        git(r, "revert", "--no-edit", "-m", "1", "HEAD")
        git(r, "update-ref", "refs/remotes/origin/dev", "dev")
        git(r, "checkout", "-q", "-b", "upstream-sync/after-revert", "dev")
        assert status(r)["by"][U["merge_rev"]]["status"] == "reverted"
    print("ok: sync.py sorts, picks, records and refuses as it should")


def lock_case() -> None:
    """`pick --lock` through to the end, on a small Cargo workspace (path crates: no network):
    upstream bumps a crate, the fork added one, and only Cargo.lock conflicts."""
    cargo = shutil.which("cargo")
    if not cargo:
        print("skipped: the --lock pick (cargo is not on PATH)")
        return
    with tempfile.TemporaryDirectory() as tmp:
        r = Path(tmp)
        git(r, "init", "-q", "-b", "main")
        for k, v in {"user.name": "T", "user.email": "t@example.com", "commit.gpgsign": "false",
                     "core.autocrlf": "false", "protocol.https.allow": "never"}.items():
            git(r, "config", k, v)

        def crate(name: str, version: str, deps: str = "") -> None:
            write(r, f"{name}/Cargo.toml", f'[package]\nname = "{name}"\nversion = "{version}"\n'
                  f'edition = "2021"\n\n[dependencies]\n{deps}')
            write(r, f"{name}/src/lib.rs", "")

        def lockfile() -> None:
            run(r, cargo, "generate-lockfile", "--offline")

        write(r, "Cargo.toml", '[workspace]\nmembers = ["app", "dep"]\nresolver = "2"\n')
        write(r, ".gitignore", "target/\n")
        crate("app", "0.1.0", 'dep = { path = "../dep", version = "1" }\n')
        crate("dep", "1.0.0")
        lockfile()
        base = commit(r, "base")

        git(r, "checkout", "-q", "-b", "dev")
        write(r, "Cargo.toml", '[workspace]\nmembers = ["app", "dep", "dep2"]\nresolver = "2"\n')
        crate("dep2", "0.1.0")
        lockfile()
        write(r, LEDGER, HEADER)
        commit(r, "fork: a crate of its own")
        git(r, "update-ref", "refs/remotes/origin/dev", "HEAD")
        fork_lock = (r / "Cargo.lock").read_text(encoding="utf-8")

        git(r, "checkout", "-q", "main")
        crate("dep", "1.0.1")
        lockfile()
        bump = commit(r, "dep 1.0.1")
        git(r, "update-ref", "refs/remotes/upstream/main", "HEAD")

        git(r, "checkout", "-q", "-b", "upstream-sync/lock", "dev")
        c = status(r)["by"][bump]
        assert c["kind"] == "lock" and c["conflicts"] == ["Cargo.lock"], c
        assert "--lock" in sync(r, "pick", bump, ok=False).stdout
        out = sync(r, "pick", "--lock", bump).stdout
        assert "picked as" in out, out
        lock = (r / "Cargo.lock").read_text(encoding="utf-8")
        assert 'name = "dep2"' in lock, lock                 # the fork's lock, kept
        assert 'name = "dep"\nversion = "1.0.1"' in lock, lock  # with upstream's bump in it
        assert lock != fork_lock
        assert "Cargo.lock: the fork's, with upstream's version bumps" in git(
            r, "log", "-1", "--format=%B")
        assert git(r, "diff", "--name-only") == LEDGER and git(r, "diff", "--cached") == ""
        run(r, cargo, "metadata", "--format-version", "1", "--offline", "--locked")  # consistent
        assert base
    print("ok: a lock-only conflict is picked with --lock")


if __name__ == "__main__":
    main()
    lock_case()
