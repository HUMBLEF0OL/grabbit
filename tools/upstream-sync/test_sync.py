#!/usr/bin/env python3
"""sync.py against a throwaway repository: a fork that removed a platform and a language and
changed a file, an upstream with one commit of each kind. `python tools/upstream-sync/test_sync.py`
"""
from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

SYNC = Path(__file__).with_name("sync.py")


def run(cwd: Path, *args: str, ok: bool = True) -> subprocess.CompletedProcess:
    r = subprocess.run(args, cwd=cwd, capture_output=True, text=True, encoding="utf-8")
    if ok and r.returncode != 0:
        raise AssertionError(f"{args}: {r.returncode}\n{r.stdout}\n{r.stderr}")
    return r


def git(cwd: Path, *args: str) -> str:
    return run(cwd, "git", *args).stdout.strip()


def sync(cwd: Path, *args: str, ok: bool = True) -> subprocess.CompletedProcess:
    return run(cwd, sys.executable, str(SYNC), *args, ok=ok)


def write(repo: Path, path: str, text: str) -> None:
    p = repo / path
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(text, encoding="utf-8", newline="\n")


def commit(repo: Path, msg: str) -> str:
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", msg)
    return git(repo, "rev-parse", "HEAD")


def main() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        r = Path(tmp)
        git(r, "init", "-q", "-b", "main")
        for k, v in {"user.name": "T", "user.email": "t@example.com", "commit.gpgsign": "false",
                     "core.autocrlf": "false"}.items():
            git(r, "config", k, v)
        lines = "".join(f"line {i}\n" for i in range(1, 30))
        write(r, "a.rs", lines)
        write(r, "b.rs", lines)
        write(r, "plat/mac.rs", "mac\n")
        write(r, "loc/fr.txt", "bonjour\n")
        write(r, "CHANGELOG.md", "# Changelog\n\n- old\n")
        write(r, "ROADMAP.md", "# Roadmap\n")
        base = commit(r, "base")

        # The fork: another platform and language removed, a.rs's first line its own.
        git(r, "checkout", "-q", "-b", "dev")
        (r / "plat/mac.rs").unlink()
        (r / "loc/fr.txt").unlink()
        write(r, "a.rs", lines.replace("line 1\n", "fork line 1\n", 1))
        write(r, "CHANGELOG.md", "# Changelog\n\n- the fork's\n- old\n")
        commit(r, "fork: Windows and English only")
        git(r, "update-ref", "refs/remotes/origin/dev", "HEAD")

        # Upstream, one commit of each kind.
        git(r, "checkout", "-q", "main")
        write(r, "b.rs", lines.replace("line 20\n", "line 20 fixed\n"))
        write(r, "ROADMAP.md", "# Roadmap\n\n- upstream's line\n")
        u_clean = commit(r, "clean: b.rs fixed")
        write(r, "plat/mac.rs", "mac fixed\n")
        u_removed = commit(r, "macOS: only the removed platform")
        write(r, "b.rs", lines.replace("line 20\n", "line 20 fixed\n")
              .replace("line 25\n", "line 25 fixed\n"))
        write(r, "plat/mac.rs", "mac fixed again\n")
        write(r, "loc/de.txt", "hallo\n")
        write(r, "CHANGELOG.md", "# Changelog\n\n- upstream fix\n- old\n")
        u_prune = commit(r, "prune: b.rs, the removed platform and a new language")
        write(r, "a.rs", lines.replace("line 1\n", "upstream line 1\n", 1))
        u_port = commit(r, "port: a crash fix in a.rs's first line")
        write(r, "a.rs", lines.replace("line 1\n", "upstream line 1 again\n", 1))
        u_after = commit(r, "after: builds on the port")
        git(r, "update-ref", "refs/remotes/upstream/main", "HEAD")

        git(r, "checkout", "-q", "-b", "upstream-sync/test", "dev")
        st = json.loads(sync(r, "status", "--json").stdout)
        kinds = {c["sha"]: c for c in st["commits"]}
        assert st["fork_point"] == base
        assert kinds[u_clean]["kind"] == "clean", kinds[u_clean]
        assert "ROADMAP.md" in kinds[u_clean]["left_out"]
        assert kinds[u_removed]["kind"] == "removed", kinds[u_removed]
        assert kinds[u_prune]["kind"] == "prune", kinds[u_prune]
        assert {"plat/mac.rs", "loc/de.txt"} <= set(kinds[u_prune]["left_out"])
        assert kinds[u_port]["kind"] == "port" and kinds[u_port]["conflicts"] == ["a.rs"]
        assert kinds[u_port]["critical"]
        assert kinds[u_after]["kind"] == "port" and u_port in kinds[u_after]["after"]

        # Picks: never on dev; given out of order, taken in upstream's order.
        git(r, "checkout", "-q", "dev")
        assert sync(r, "pick", u_clean, ok=False).returncode == 1
        git(r, "checkout", "-q", "upstream-sync/test")
        out = sync(r, "pick", u_prune, u_clean).stdout
        assert out.index(u_clean[:12]) < out.index(u_prune[:12]), out
        assert not (r / "plat/mac.rs").exists() and not (r / "loc/de.txt").exists()
        assert "line 25 fixed" in (r / "b.rs").read_text(encoding="utf-8")
        assert (r / "ROADMAP.md").read_text(encoding="utf-8") == "# Roadmap\n"
        assert (r / "CHANGELOG.md").read_text(encoding="utf-8").startswith("# Changelog\n\n- the fork's")
        log = git(r, "log", "--format=%B", "-2")
        assert f"(cherry picked from commit {u_clean})" in log
        assert f"(cherry picked from commit {u_prune})" in log and "- loc/de.txt" in log
        assert git(r, "log", "-1", "--format=%an", "HEAD") == "T"

        # A pick taken already is not taken again; a conflict leaves nothing behind.
        assert "already picked" in sync(r, "pick", u_clean).stdout
        head = git(r, "rev-parse", "HEAD")
        res = sync(r, "pick", u_port, ok=False)
        assert res.returncode == 1 and "needs a port" in res.stdout, res.stdout
        assert git(r, "rev-parse", "HEAD") == head
        assert git(r, "status", "--porcelain", "--untracked-files=no") == ""
        assert not Path(git(r, "rev-parse", "--git-path", "CHERRY_PICK_HEAD")).exists()

        # A removed-only commit picked: nothing left, nothing committed.
        res = sync(r, "pick", u_removed, ok=False)
        assert "record it skipped" in res.stdout and git(r, "rev-parse", "HEAD") == head

        # Decisions without a commit: the ledger. A port: its trailer.
        (r / "tools/upstream-sync").mkdir(parents=True)
        write(r, "tools/upstream-sync/ledger.md", "| upstream | status | ref | subject | note |\n"
              "|---|---|---|---|---|\n")
        sync(r, "record", u_removed, "skipped", "--note", "macOS only")
        sync(r, "record", u_removed, "skipped", "--note", "macOS only, again")  # replaces
        ledger = (r / "tools/upstream-sync/ledger.md").read_text(encoding="utf-8")
        assert ledger.count(u_removed[:12]) == 1 and "again" in ledger
        # The ledger may be uncommitted while picking (it is the only file that may).
        write(r, "a.rs", lines.replace("line 1\n", "fork line 1, the port\n", 1))
        git(r, "commit", "-qam", f"Port of the crash fix\n\nUpstream-Port: {u_port[:12]}")
        st = json.loads(sync(r, "status", "--json").stdout)
        left = {c["sha"] for c in st["commits"]}
        assert left == {u_after}, [c["subject"] for c in st["commits"]]
        assert u_port not in json.loads(sync(r, "status", "--json").stdout)["commits"][0]["after"]

        # Deferred shows again; a ledger row upstream no longer has is reported.
        sync(r, "record", u_after, "deferred", "--note", "after the window's rewrite")
        st = json.loads(sync(r, "status", "--json").stdout)
        assert [c["status"] for c in st["commits"]] == ["deferred"]
        with open(r / "tools/upstream-sync/ledger.md", "a", encoding="utf-8") as f:
            f.write("| 0123456789ab | skipped |  | gone | rewritten |\n")
        assert json.loads(sync(r, "status", "--json").stdout)["ledger_unknown"] == ["0123456789ab"]

        # PR: refuses when origin is the original.
        git(r, "remote", "add", "origin", "https://github.com/anefzaoui/misterdownloadmanager.git")
        git(r, "add", "-A")
        git(r, "commit", "-qm", "ledger")
        res = sync(r, "pr", "--title", "t", "--body-file", "x", ok=False)
        assert res.returncode == 1 and "never push there" in res.stderr, res.stderr
    print("ok: sync.py sorts, picks, records and refuses as it should")


if __name__ == "__main__":
    os.environ.pop("GIT_DIR", None)
    main()
