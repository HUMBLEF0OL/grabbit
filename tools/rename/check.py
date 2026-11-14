"""Lists the "Mister Download Manager" and "MDM" users still see (plan 10a's done-when check).

    python tools/rename/check.py              every hit, grouped by area, with file:line
    python tools/rename/check.py --summary    counts only
    python tools/rename/check.py --self-test  the checker's own asserts

Exits 1 while any user-visible hit remains. Hits inside Rust `#[cfg(test)]` code are listed
and counted, marked [test], but don't fail the check (tests follow the texts they assert).
Rust files: only string literals count (comments and doc comments are skipped). Other files:
every line counts, except whole comment lines of the installer script and the remote page.
Plan: design/graphite/plans/10-rename-grabbit.md. Stdlib only, Python 3.10+.
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

TERMS = {
    "Mister Download Manager": re.compile(r"Mister Download Manager"),
    "MDM": re.compile(r"\bMDM\b"),
}

# Names plan 10 keeps or never changes. They are removed from a text before TERMS are matched,
# so they are never reported. Most don't match TERMS anyway (case, underscores); listing them
# keeps them safe if TERMS are widened, and documents the rule.
ALLOW = [
    (r"\{\{?667DB4D8-32C2-4487-A0E5-5825BF29778C\}", "installer AppId: upgrades find the install by it"),
    (r"\b[a-p]{32}\b", "Chromium extension IDs (unpacked, store): a new ID is a new extension"),
    (r"integration@misterdownloadmanager\.com", "Firefox gecko ID: a new ID is a new add-on"),
    (r"com\.misterdownloadmanager\.host", "native messaging host name (10c registers a second one)"),
    (r"MisterDownloadManager\.(?:Torrent|torrent)", "torrent ProgID (10c migrates it)"),
    (r"MisterDownloadManager\.Running", "single-instance mutex (10c migrates it)"),
    (r"MisterDownloadManager(?:Test)?\b", "autostart Run value, User-Agent token, asset names (10c/10d)"),
    (r"MisterDownloadManager-[\w${}.-]*", "release asset names old builds update from (10d)"),
    (r"misterdownloadmanager[\w.-]*", "data folder, exe names, domain and historic URLs"),
    (r"mister-download-manager", "Firefox add-ons page slug (store URL)"),
    (r"\bmdm-[a-z0-9-]+", "crate names (decision: keep)"),
    (r"\bMDM_[A-Z0-9_]+", "environment variables (10c adds GRABBIT_ ones, keeps these)"),
    (r"\bMDM\.popup\b", "internal window-property marker, never shown"),
    (r"\bMDM-test-[\w{}:-]*", "keychain names of off-screen test copies, never shown"),
]
ALLOW_RE = re.compile("|".join(f"(?:{p})" for p, _ in ALLOW))

TEXT_EXT = {".md", ".html", ".json", ".txt", ".css", ".js", ".toml", ".yml", ".tsv", ".jsonl", ".webmanifest"}


def hits_in(text):
    """The TERMS found in `text` once the allowed names are taken out."""
    text = ALLOW_RE.sub(" ", text)
    return [name for name, rx in TERMS.items() if rx.search(text)]


def rust_strings(src):
    """(line, text, in_test) for every string literal of a Rust source, one entry per line a
    string spans. Skips comments; a `#[cfg(test)]` item's braces mark its strings as test."""
    out = []
    i, n, line = 0, len(src), 1
    depth, test_from, pending = 0, None, False
    while i < n:
        c = src[i]
        if c == "\n":
            line += 1
            i += 1
        elif src.startswith("//", i):
            i = src.find("\n", i)
            i = n if i < 0 else i
        elif src.startswith("/*", i):
            j = src.find("*/", i)
            j = n if j < 0 else j + 2
            line += src.count("\n", i, j)
            i = j
        elif src.startswith("#[cfg(test)]", i):
            pending = True
            i += 12
        elif c == "{":
            depth += 1
            if pending and test_from is None:
                test_from = depth
            pending = False
            i += 1
        elif c == "}":
            if test_from is not None and depth == test_from:
                test_from = None
            depth -= 1
            i += 1
        elif c == ";":
            pending = False
            i += 1
        elif c == "'":
            # A char literal ('x', '"', '\n', '\u{..}') or a lifetime ('a): skip only the former.
            if i + 2 < n and src[i + 2] == "'" and src[i + 1] != "\\":
                i += 3
            elif src.startswith("'\\", i):
                j = src.find("'", i + 2)
                i = n if j < 0 else j + 1
            else:
                i += 1
        elif c == '"' or (c == "r" and re.match(r'r#*"', src[i:i + 8]) and not (i and (src[i - 1].isalnum() or src[i - 1] == "_"))):
            if c == "r":
                hashes = len(re.match(r"r(#*)", src[i:]).group(1))
                end, i = '"' + "#" * hashes, i + 2 + hashes
                j = src.find(end, i)
                j = n if j < 0 else j
            else:
                i += 1
                j = i
                while j < n and src[j] != '"':
                    j += 2 if src[j] == "\\" else 1
                end = '"'
            body = src[i:j]
            for k, part in enumerate(body.split("\n")):
                out.append((line + k, part, test_from is not None))
            line += body.count("\n")
            i = j + len(end)
        else:
            i += 1
    return out


def scan_rust(path):
    for ln, text, test in rust_strings(path.read_text(encoding="utf-8", errors="replace")):
        for term in hits_in(text):
            yield ln, term, text.strip(), test


def scan_text(path, comment=None):
    for ln, text in enumerate(path.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
        if comment and text.lstrip().startswith(comment):
            continue
        for term in hits_in(text):
            yield ln, term, text.strip(), False


def files(pattern):
    return sorted(p for p in ROOT.glob(pattern) if p.is_file())


AREAS = [
    # ponytail: labserver is a developer's test server, never shipped; skipped.
    ("App (Rust strings)", lambda: [(p, scan_rust) for p in files("crates/*/src/**/*.rs") + files("crates/*/build.rs") if p.parts[len(ROOT.parts) + 1] != "labserver"]),
    ("App remote page", lambda: [(p, lambda q: scan_text(q, comment=("//", "/*", "*", "<!--"))) for p in files("crates/remote/web/*") if p.suffix in TEXT_EXT]),
    ("Installer", lambda: [(p, lambda q: scan_text(q, comment=(";", "//", "{"))) for p in files("tools/installer/*.iss")]),
    ("Extension", lambda: [(p, scan_text) for p in files("extension/_locales/**/*.json") + files("extension/*.html")]),
    ("Website", lambda: [(p, scan_text) for p in files("website/content/**/*") if p.suffix in TEXT_EXT]),
    ("Docs", lambda: [(p, scan_text) for p in files("docs/**/*") if p.suffix in TEXT_EXT]),
    ("README", lambda: [(p, scan_text) for p in files("README.md")]),
]


def run(summary):
    failing = 0
    rows = []
    for area, sources in AREAS:
        found = []
        for path, scan in sources():
            rel = path.relative_to(ROOT).as_posix()
            found += [(rel, *hit) for hit in scan(path)]
        counts = {t: sum(1 for f in found if f[2] == t and not f[4]) for t in TERMS}
        tests = sum(1 for f in found if f[4])
        nfiles = len({f[0] for f in found})
        failing += sum(counts.values())
        rows.append((area, counts, tests, nfiles))
        if not summary and found:
            print(f"\n== {area} ==")
            for rel, ln, term, text, test in found:
                mark = " [test]" if test else ""
                print(f"{rel}:{ln}: {term}{mark}: {text[:110]}")
    print("\nArea                  Mister Download Manager    MDM   [test]  files")
    for area, counts, tests, nfiles in rows:
        print(f"{area:<22}{counts['Mister Download Manager']:>23}{counts['MDM']:>7}{tests:>9}{nfiles:>7}")
    total = {t: sum(r[1][t] for r in rows) for t in TERMS}
    print(f"{'Total':<22}{total['Mister Download Manager']:>23}{total['MDM']:>7}{sum(r[2] for r in rows):>9}{sum(r[3] for r in rows):>7}")
    print("(counts are hits: a line with both names counts once under each)")
    return 1 if failing else 0


def self_test():
    src = '''
/// About MDM: doc comments don't count.
const A: &str = "Mister Download Manager"; // MDM in a comment doesn't count
const U: &str = "https://example.com/x"; const B: &str = "MDM runs";
const C: char = '"'; fn f<'a>(x: &'a str) {}
const M: &str = "line one \\
    then MDM";
const R: &str = r#"say "MDM""#;
#[cfg(test)]
mod tests {
    const T: &str = "Mister Download Manager test";
}
const AFTER: &str = "MDM after";
'''
    got = [(ln, test) for ln, text, test in rust_strings(src) if hits_in(text)]
    assert got == [(3, False), (4, False), (7, False), (8, False), (11, True), (13, False)], got
    for kept in [
        "MDM_DATA_DIR=x", "misterdownloadmanager-cli api on", "MDM.popup",
        "com.misterdownloadmanager.host", "MisterDownloadManager.Running", "mdm-app",
        "https://misterdownloadmanager.com/help/", "integration@misterdownloadmanager.com",
        "MisterDownloadManager-{version}-{tail}", "AppId={{667DB4D8-32C2-4487-A0E5-5825BF29778C}",
        "keychain:MDM-test-unit/aaaa0001/",
    ]:
        assert hits_in(kept) == [], kept
    assert hits_in("Support MDM") == ["MDM"]
    assert hits_in("Open Mister Download Manager (MDM)") == ["Mister Download Manager", "MDM"]
    print("self-test ok")


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")  # Windows consoles default to cp1252
    if "--self-test" in sys.argv:
        self_test()
        sys.exit(0)
    sys.exit(run("--summary" in sys.argv))
