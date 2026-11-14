"""Builds every app icon size from art/grabbit.svg, Grabbit's double-chevron mark.

The SVG is plain on purpose: four polygons, one linear gradient, a stroke width that rounds
the corners (stroke-linejoin round). This script reads those and draws them with Pillow, so no
SVG renderer is needed: the mark at 1024 px, then each size scaled down from it.

Writes crates/app/assets/icons/app/mdm-<size>.png, mdm.ico (all sizes), the browser
extension's icons, the copy of icon-48 that extension/video.js carries as data, and the
website's logos and favicon (website/assets/). tools/readme/social.py takes mdm-256.png.
Run from the repository root: python tools/icon/build_icon.py
"""

import base64
import re
import xml.etree.ElementTree as ET
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[2]
SRC = ROOT / "art" / "grabbit.svg"
APP = ROOT / "crates" / "app" / "assets" / "icons" / "app"
EXT = ROOT / "extension" / "icons"
VIDEO_JS = ROOT / "extension" / "video.js"
SITE = ROOT / "website" / "assets"
WEB = {"favicon-32.png": 32, "logo-64.png": 64, "apple-touch-icon.png": 256, "logo-256.webp": 256, "logo-640.png": 640}
SIZES = [16, 20, 24, 32, 48, 64, 128, 256]
BIG = 1024
NS = {"s": "http://www.w3.org/2000/svg"}


def hex_rgb(h: str):
    h = h.lstrip("#")
    return tuple(int(h[i : i + 2], 16) for i in (0, 2, 4))


def render(svg: Path, size: int) -> Image.Image:
    root = ET.parse(svg).getroot()
    vx, vy, vw, vh = map(float, root.get("viewBox").split())
    k = size / max(vw, vh)
    to_px = lambda x, y: ((x - vx) * k, (y - vy) * k)

    group = root.find("s:g", NS)
    radius = float(group.get("stroke-width")) / 2 * k
    mask = Image.new("L", (size, size), 0)
    draw = ImageDraw.Draw(mask)
    for poly in group.findall("s:polygon", NS):
        nums = list(map(float, re.split(r"[ ,]+", poly.get("points").strip())))
        pts = [to_px(nums[i], nums[i + 1]) for i in range(0, len(nums), 2)]
        draw.polygon(pts, fill=255)
        # The stroke with round joins: each edge as a thick line, each corner as a disc.
        for a, b in zip(pts, pts[1:] + pts[:1]):
            draw.line([a, b], fill=255, width=round(2 * radius))
        for x, y in pts:
            draw.ellipse([x - radius, y - radius, x + radius, y + radius], fill=255)

    grad = root.find("s:defs/s:linearGradient", NS)
    x1, y1 = to_px(float(grad.get("x1")), float(grad.get("y1")))
    x2, y2 = to_px(float(grad.get("x2")), float(grad.get("y2")))
    c0, c1 = (hex_rgb(s.get("stop-color")) for s in grad.findall("s:stop", NS))
    dx, dy = x2 - x1, y2 - y1
    d2 = dx * dx + dy * dy
    # ponytail: per-pixel loop in plain Python, about a second at 1024 px; numpy if it ever matters.
    t = Image.new("L", (size, size))
    t.putdata(
        [
            round(255 * min(1.0, max(0.0, ((x - x1) * dx + (y - y1) * dy) / d2)))
            for y in range(size)
            for x in range(size)
        ]
    )
    colour = Image.composite(Image.new("RGB", (size, size), c1), Image.new("RGB", (size, size), c0), t)
    colour.putalpha(mask)
    return colour


def main() -> None:
    big = render(SRC, BIG)
    icons = {s: big.resize((s, s), Image.LANCZOS) for s in SIZES}
    for size, im in icons.items():
        im.save(APP / f"mdm-{size}.png", optimize=True)
    icons[256].save(
        APP / "mdm.ico",
        sizes=[(s, s) for s in SIZES],
        append_images=[icons[s] for s in SIZES if s != 256],
    )
    for size in (16, 32, 48, 128):
        icons[size].save(EXT / f"icon-{size}.png", optimize=True)
    # video.js shows the icon on pages, as data: extension files aren't exposed to them.
    b64 = base64.b64encode((EXT / "icon-48.png").read_bytes()).decode()
    js = VIDEO_JS.read_bytes()  # bytes, so its line endings stay as they are
    js, n = re.subn(rb'"data:image/png;base64,[A-Za-z0-9+/=]*"', f'"data:image/png;base64,{b64}"'.encode(), js, count=1)
    if n != 1:
        raise SystemExit("extension/video.js: no data:image/png LOGO to replace")
    VIDEO_JS.write_bytes(js)
    # The website: favicon, header logo, apple-touch and web-app icons, the schema.org logo.
    for name, size in WEB.items():
        im = icons[size] if size in icons else big.resize((size, size), Image.LANCZOS)
        im.save(SITE / name, **({"lossless": True} if name.endswith(".webp") else {"optimize": True}))
    print("wrote", ", ".join(map(str, SIZES)), "+ mdm.ico + extension icons + video.js LOGO + website logos")


if __name__ == "__main__":
    main()
