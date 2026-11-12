"""Compare screenshots pixel for pixel (plans/05-main-window.md, 05a: "no visual change").

Two files, or two folders: every PNG under the first (its subfolders too) is compared with the
one at the same relative path under the second. Prints, for each pair, how many pixels differ
(any channel, alpha included) and the box around them; exits 1 if any pair differs (size,
pixels, a picture missing from either side or one that can't be read), 0 if every pair is
identical, 2 on a bad command line.

    python design/graphite/source/pixel_diff.py before.png after.png
    python design/graphite/source/pixel_diff.py target/05a/before target/05a/after

--diff DIR writes, for each pair that differs, a picture of the differing pixels (red on the
"before" picture dimmed) to DIR, at the pair's relative path.
"""

import argparse
import sys
from pathlib import Path

from PIL import Image, ImageChops


def compare(a: Path, b: Path, diff_to):
    """(differing pixels, message) for one pair; -1 when the sizes differ."""
    ia = Image.open(a).convert("RGBA")
    ib = Image.open(b).convert("RGBA")
    if ia.size != ib.size:
        return -1, f"size {ia.size[0]}x{ia.size[1]} vs {ib.size[0]}x{ib.size[1]}"
    delta = ImageChops.difference(ia, ib)
    # A pixel differs when any of its four channels does (getbbox() alone looks at alpha only).
    r, g, b_, al = (c.point(lambda v: 255 if v else 0) for c in delta.split())
    mask = ImageChops.lighter(ImageChops.lighter(r, g), ImageChops.lighter(b_, al))
    box = mask.getbbox()
    if box is None:
        return 0, f"identical ({ia.size[0]}x{ia.size[1]})"
    count = mask.histogram()[255]
    if diff_to is not None:
        diff_to.parent.mkdir(parents=True, exist_ok=True)
        shown = Image.blend(ia.convert("RGB"), Image.new("RGB", ia.size, "white"), 0.7)
        shown.paste(Image.new("RGB", ia.size, (255, 0, 0)), mask=mask)
        shown.save(diff_to)
    return count, f"{count} of {ia.size[0] * ia.size[1]} pixels differ, in {box}"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("before", type=Path)
    ap.add_argument("after", type=Path)
    ap.add_argument("--diff", type=Path, default=None, help="folder for pictures of the differences")
    args = ap.parse_args()
    for p in (args.before, args.after):
        if not p.exists():
            print(f"{p}: not found", file=sys.stderr)
            return 2
    if args.before.is_dir() != args.after.is_dir():
        print("give two files or two folders", file=sys.stderr)
        return 2
    if args.before.is_dir():
        names = sorted(f.relative_to(args.before) for f in args.before.rglob("*.png"))
        if not names:
            print(f"no PNG under {args.before}", file=sys.stderr)
            return 2
        pairs = [(args.before / n, args.after / n, n) for n in names]
        extra = sorted(
            f.relative_to(args.after)
            for f in args.after.rglob("*.png")
            if not (args.before / f.relative_to(args.after)).exists()
        )
    else:
        pairs = [(args.before, args.after, Path(args.before.name))]
        extra = []
    bad = 0
    for a, b, name in pairs:
        if not b.exists():
            print(f"{name}: missing from {args.after}")
            bad += 1
            continue
        try:
            count, msg = compare(a, b, args.diff / name if args.diff else None)
        except OSError as e:
            print(f"{name}: can't be read ({e})")
            bad += 1
            continue
        print(f"{name}: {msg}")
        if count != 0:
            bad += 1
    for name in extra:
        print(f"{name}: only in {args.after}")
        bad += 1
    print(f"{len(pairs) - bad} of {len(pairs)} identical")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
