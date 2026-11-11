"""Compare screenshots pixel for pixel (plans/05-main-window.md, 05a: "no visual change").

Two files, or two folders (every PNG in the first compared with the one of the same name in
the second). Prints, for each pair, how many pixels differ and the box around them; exits 1 if
any pair differs (size, pixels, or a picture missing from the second folder), 0 if every pair
is identical.

    python design/graphite/source/pixel_diff.py before.png after.png
    python design/graphite/source/pixel_diff.py target/05a/before/classic target/05a/after/classic

--diff DIR writes, for each pair that differs, a picture of the differing pixels (red on the
"before" picture dimmed) to DIR.
"""

import argparse
import sys
from pathlib import Path

from PIL import Image, ImageChops


def compare(a: Path, b: Path, diff_dir):
    """(differing pixels, box, message) for one pair; pixels -1 when the sizes differ."""
    ia = Image.open(a).convert("RGBA")
    ib = Image.open(b).convert("RGBA")
    if ia.size != ib.size:
        return -1, None, f"size {ia.size[0]}x{ia.size[1]} vs {ib.size[0]}x{ib.size[1]}"
    delta = ImageChops.difference(ia, ib)
    box = delta.getbbox()
    if box is None:
        return 0, None, f"identical ({ia.size[0]}x{ia.size[1]})"
    # A pixel differs when any channel does.
    r, g, b_, al = (c.point(lambda v: 255 if v else 0) for c in delta.split())
    mask = ImageChops.lighter(ImageChops.lighter(r, g), ImageChops.lighter(b_, al))
    count = mask.histogram()[255]
    if diff_dir is not None:
        diff_dir.mkdir(parents=True, exist_ok=True)
        shown = Image.blend(ia.convert("RGB"), Image.new("RGB", ia.size, "white"), 0.7)
        shown.paste(Image.new("RGB", ia.size, (255, 0, 0)), mask=mask)
        shown.save(diff_dir / a.name)
    return count, box, f"{count} of {ia.size[0] * ia.size[1]} pixels differ, in {box}"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("before", type=Path)
    ap.add_argument("after", type=Path)
    ap.add_argument("--diff", type=Path, default=None, help="folder for pictures of the differences")
    args = ap.parse_args()
    if args.before.is_dir() != args.after.is_dir():
        print("give two files or two folders", file=sys.stderr)
        return 2
    if args.before.is_dir():
        pairs = [(f, args.after / f.name) for f in sorted(args.before.glob("*.png"))]
        if not pairs:
            print(f"no PNG in {args.before}", file=sys.stderr)
            return 2
        extra = sorted(
            f.name for f in args.after.glob("*.png") if not (args.before / f.name).exists()
        )
    else:
        pairs = [(args.before, args.after)]
        extra = []
    bad = 0
    for a, b in pairs:
        if not b.exists():
            print(f"{a.name}: missing from {args.after}")
            bad += 1
            continue
        count, _, msg = compare(a, b, args.diff)
        print(f"{a.name}: {msg}")
        if count != 0:
            bad += 1
    for name in extra:
        print(f"{name}: only in {args.after}")
        bad += 1
    print(f"{len(pairs) - bad} of {len(pairs)} identical" if pairs else "nothing compared")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
