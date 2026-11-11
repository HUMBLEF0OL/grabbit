"""Export the Graphite icon set from the vendored Tabler Icons package.

Writes design/graphite/assets/icons/{16,24}/<need>.svg for every need in
crates/app/src/themeicons.rs NEEDS, plus SOURCES.txt and LICENSE beside them.

The names come from iconmap.py (MAP, first column) with gen3.py's REFINE applied. Both files
are read as source (not imported: they run their own work on import). Run it again after any
mapping change:

    python design/graphite/source/export_icons.py
"""

import ast
import base64
import hashlib
import re
import shutil
import sys
import tarfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
GRAPHITE = HERE.parent
REPO = GRAPHITE.parent.parent
VERSION = "3.31.0"
TGZ = GRAPHITE / "vendor" / f"tabler-icons-{VERSION}.tgz"
LICENSE = GRAPHITE / "vendor" / "TABLER-LICENSE"
OUT = GRAPHITE / "assets" / "icons"
NEEDS_RS = REPO / "crates" / "app" / "src" / "themeicons.rs"
SIZES = (16, 24)
STROKE_WIDTH = "1.75"
EMPTY_PATH = '<path stroke="none" d="M0 0h24v24H0z" fill="none"/>'


def literal(path: Path, name: str):
    """The literal value assigned to `name` at the top level of a Python file."""
    tree = ast.parse(path.read_text(encoding="utf8"))
    for node in tree.body:
        if isinstance(node, ast.Assign):
            for target in node.targets:
                if isinstance(target, ast.Name) and target.id == name:
                    return ast.literal_eval(node.value)
    sys.exit(f"{path.name}: no {name}")


def needs() -> list[str]:
    """The need ids in themeicons.rs NEEDS, in order."""
    src = NEEDS_RS.read_text(encoding="utf8")
    start = src.index("pub const NEEDS")
    body = src[start : src.index("];", start)]
    return re.findall(r'\(\s*"([a-z0-9-]+)",', body)


def mapping() -> dict[str, str]:
    """need -> Tabler outline name: iconmap.py's first column, then gen3.py's REFINE."""
    names = {need: pair[0] for need, pair in literal(HERE / "iconmap.py", "MAP").items()}
    names.update(literal(HERE / "gen3.py", "REFINE"))
    return names


def convert(svg: str, px: int) -> str:
    """Tabler's outline SVG as the app takes it: white ink, stroke 1.75, `px` square."""
    svg = svg.replace(EMPTY_PATH, "")
    svg = re.sub(r'\s+class="[^"]*"', "", svg)
    opening = re.match(r"<svg\b[^>]*>", svg)
    if not opening:
        raise ValueError("no <svg> element")
    head = opening.group(0)
    for attr, value in (
        ("width", str(px)),
        ("height", str(px)),
        ("stroke", "#ffffff"),
        ("stroke-width", STROKE_WIDTH),
    ):
        head, n = re.subn(rf'(\s){attr}="[^"]*"', rf'\g<1>{attr}="{value}"', head, count=1)
        if n != 1:
            raise ValueError(f"<svg> has no {attr}")
    if 'viewBox="0 0 24 24"' not in head:
        raise ValueError("viewBox is not 0 0 24 24")
    svg = head + svg[opening.end() :]
    # Drop the blank lines the removed path leaves; keep the drawing's own lines as they are.
    lines = [line.rstrip() for line in svg.splitlines() if line.strip()]
    return "\n".join(lines) + "\n"


def check(ids: list[str]) -> None:
    """Every need at both sizes and nothing else; each file well-formed SVG as written."""
    import xml.etree.ElementTree as ET

    ns = "{http://www.w3.org/2000/svg}"
    for px in SIZES:
        folder = OUT / str(px)
        found = sorted(p.stem for p in folder.glob("*.svg"))
        if found != sorted(ids):
            sys.exit(f"{folder}: files and NEEDS differ")
        for need in ids:
            root = ET.parse(folder / f"{need}.svg").getroot()
            want = {
                "width": str(px),
                "height": str(px),
                "viewBox": "0 0 24 24",
                "stroke": "#ffffff",
                "stroke-width": STROKE_WIDTH,
                "fill": "none",
            }
            got = {k: root.get(k) for k in want}
            if root.tag != f"{ns}svg" or got != want or "class" in root.attrib:
                sys.exit(f"{px}/{need}.svg: {got}")
            if len(root) == 0:
                sys.exit(f"{px}/{need}.svg: draws nothing")
            for el in root.iter():
                if el.get("d") == "M0 0h24v24H0z":
                    sys.exit(f"{px}/{need}.svg: the empty path is still there")
                if el.get("stroke") not in (None, "#ffffff"):
                    sys.exit(f"{px}/{need}.svg: a part stroked {el.get('stroke')}")


def main() -> None:
    ids = needs()
    names = mapping()
    unknown = sorted(set(names) - set(ids))
    missing = [n for n in ids if n not in names]
    if unknown or missing:
        sys.exit(f"mapping and NEEDS differ: not needs {unknown}, unmapped {missing}")

    data = TGZ.read_bytes()
    sha256 = hashlib.sha256(data).hexdigest()
    sha1 = hashlib.sha1(data).hexdigest()
    integrity = "sha512-" + base64.b64encode(hashlib.sha512(data).digest()).decode()

    with tarfile.open(TGZ, "r:gz") as tar:
        version = None
        pkg = tar.extractfile("package/package.json")
        if pkg:
            m = re.search(r'"version"\s*:\s*"([^"]+)"', pkg.read().decode("utf8"))
            version = m.group(1) if m else None
        if version != VERSION:
            sys.exit(f"{TGZ.name}: package.json says {version}, expected {VERSION}")
        sources = {}
        for need in ids:
            member = f"package/icons/outline/{names[need]}.svg"
            try:
                f = tar.extractfile(member)
            except KeyError:
                f = None
            if f is None:
                sys.exit(f"{need}: {member} is not in {TGZ.name}")
            sources[need] = f.read().decode("utf8")

    if OUT.exists():
        for px in SIZES:
            shutil.rmtree(OUT / str(px), ignore_errors=True)
    for px in SIZES:
        (OUT / str(px)).mkdir(parents=True, exist_ok=True)
        for need in ids:
            (OUT / str(px) / f"{need}.svg").write_text(
                convert(sources[need], px), encoding="utf8", newline="\n"
            )

    width = max(len(n) for n in ids)
    lines = [
        "The Graphite theme's icons: Tabler Icons, outline style.",
        "",
        f"Source:   @tabler/icons {VERSION}, npm tarball (design/graphite/vendor/{TGZ.name})",
        f"          https://registry.npmjs.org/@tabler/icons/-/icons-{VERSION}.tgz",
        "Homepage: https://tabler.io/icons (https://github.com/tabler/tabler-icons)",
        "Author:   Paweł Kuna",
        "Licence:  MIT (LICENSE in this folder)",
        f"Tarball SHA-256: {sha256}",
        f"Tarball SHA-1 (npm shasum): {sha1}",
        f"Tarball integrity (npm): {integrity}",
        "",
        "Changed from the originals: the empty bounding-box path and the class attribute removed,",
        f'stroke="#ffffff" (the app tints white), stroke-width="{STROKE_WIDTH}", width and height',
        "16 or 24; viewBox 0 0 24 24 kept. Made by design/graphite/source/export_icons.py.",
        "",
        "need -> Tabler outline icon",
    ]
    lines += [f"{need.ljust(width)}  {names[need]}" for need in ids]
    (OUT / "SOURCES.txt").write_text("\n".join(lines) + "\n", encoding="utf8", newline="\n")
    shutil.copyfile(LICENSE, OUT / "LICENSE")

    check(ids)
    written = len(ids) * len(SIZES)
    print(f"{len(ids)} needs, {written} files in {OUT.relative_to(REPO)}")


if __name__ == "__main__":
    main()
