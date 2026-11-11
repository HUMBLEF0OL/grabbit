"""Write crates/app/assets/themes/graphite.toml, the built-in Graphite theme, from pal.py.

The palette is the one chosen in PLAN.md (Colour direction). The token -> theme key mapping is
plans/01-prepare.md section 3 and plans/02-theme.md steps 1 to 3. After writing, it runs
pal.py's own check and a contrast check on the mapped values (the pairs the app's palette tests
look at, where the role is set here rather than derived). The app's tests check the file again
(`cargo test -p mdm-app palette`).

    python design/graphite/source/make_theme.py
"""

import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import pal  # noqa: E402

PALETTE = "amethyst"
REPO = HERE.parent.parent.parent
OUT = REPO / "crates" / "app" / "assets" / "themes" / "graphite.toml"

# Warning text: the palettes have only a warning background (warnSoft). Amber, readable on the
# window (4.5:1): the classic look's light amber, and the Obsidian palette's dark one.
WARNING = {"light": "#9d5d00", "dark": "#f5a524"}

# The keyboard focus ring. The mockup draws focus as an accent border with a faint halo (the
# `ring` token: 25 % light, 33 % dark); the app draws the ring alone, and its tests want it seen
# at 3:1 on the window and on boxes, its transparency included (palette.rs,
# style_themes_focus_rings_are_seen). `ring` gives 1.5 to 1.8:1, so the ring is the accent at
# a0 (63 %): 3.2 to 3.4:1 light, 3.5 to 3.6:1 dark.
RING = "acc@a0"

SEEDS = [
    ("background", "bg"),
    ("surface", "surface"),
    ("header", "surface"),
    ("text", "ink"),
    ("accent", "acc"),
    ("selection", "sel"),
    ("error", "err"),
    ("warning", None),
    ("success", "okt"),
]

# (comment, [(role, token)]); a token "ink@0f" is the ink at that alpha.
ROLES = [
    ("Text", [
        ("text.disabled", "dis"), ("text.muted", "ink3"), ("link", "acct"),
        ("focus.ring", RING),
    ]),
    ("Selection", [("selection.text", "ink"), ("selection.inactive", "hover")]),
    ("Fields", [
        ("field", "field"), ("field.border", "line"), ("field.bottom", "line"),
        ("field.disabled", "tile"),
    ]),
    # Greyed controls as the mockup draws them (.fld.off, .btn.off, .chk.off.on): one grey, dis.
    ("Greyed controls: the one greyed colour, no fading", [
        ("field.disabled.border", "line2"), ("button.disabled", "surface"),
        ("button.disabled.border", "line"), ("check.on.disabled", "dis"),
    ]),
    # Pressed is `press`, a step deeper than the pointer's `hover` (issue #7: never the same).
    ("Buttons: the default one filled with the outlaw colour; pressed a step deeper than hot", [
        ("button", "surface"), ("button.border", "line"), ("button.bottom", "line"),
        ("button.hot", "hover"), ("button.pressed", "press"), ("button.accent", "out"),
        ("button.accent.text", "onout"), ("button.accent.hot", "out2"),
        ("button.default.border", "out"),
    ]),
    ("Check boxes and radios", [
        ("check.on", "acc"), ("check.mark", "onacc"), ("check.border", "ctl"),
        ("check.border.disabled", "dis"),
    ]),
    ("Progress: the outlaw colour on the track", [
        ("progress.track", "track"), ("meter.track", "track"), ("progress.bar", "out"),
        ("meter.fill", "out"), ("segments.done", "out"), ("segments.missing", "track"),
        ("pieces.have", "out"), ("pieces.missing", "track"),
    ]),
    ("Lists", [
        ("list", "bg"), ("header", "bg"), ("header.line", "line"), ("grid", "line2"),
        ("list.grid", "line2"), ("chip.web", "chip"), ("chip.text", "chipt"),
        ("chip.torrent", "tile"),
    ]),
    ("Menus", [
        ("menubar", "surface"), ("menubar.line", "line"), ("menu", "surface"),
        ("menu.hot", "sel"), ("menu.border", "line"), ("menu.text.disabled", "dis"),
    ]),
    ("Toolbar", [
        ("toolbar.hot", "hover"), ("toolbar.pressed", "press"),
        ("toolbar.text.disabled", "dis"), ("icon.disabled", "dis"),
    ]),
    ("Categories tree", [("tree", "surface"), ("tree.expander", "ink3")]),
    ("The drawn title bar: the panel colour, its buttons a tint of the ink (6 %, 12 %)", [
        ("caption", "surface"), ("caption.text", "ink"), ("caption.text.inactive", "ink3"),
        ("caption.hot", "ink@0f"), ("caption.pressed", "ink@1f"),
    ]),
]

HEADER = """\
# Graphite: one of Mister Download Manager's built-in themes, its own new look (design/graphite):
# colours and shapes of our own design, light and dark (jet black, royal amethyst, electric
# cyan); it borrows nothing. Icons are the mono set until Graphite's own come, and the typeface
# is the app's usual one for now.
# It is also an example of a theme file: seeds for the colours, many roles set by hand (the
# outlaw colour on the default button and the progress bars), rounded corners, Windows 11's
# controls with a focus ring, taller list rows and a flat drawn title bar on Windows.
# Written by design/graphite/source/make_theme.py from source/pal.py, palette "{name}": edit
# the palette or the mapping there, not here.

name = "Graphite"
icons = "mono"

[controls]
field-focus = "ring"
focus = "ring"
check = "fluent"
button = "fluent"
progress = "line"
scroll = "overlay"
# A drawn, flat title bar coloured by the caption roles.
caption = "metro"

[sizes]
radius = 8
"radius.button" = 8
"radius.part" = 6
"radius.check" = 4
"radius.flyout" = 10
"radius.menu-item" = 6
"radius.popup" = 8
"radius.egui" = 8
# The mockup's row height.
"list.row" = 32
"""


def colour(tokens: dict, variant: str, token) -> str:
    if token is None:
        return WARNING[variant]
    if "@" in token:
        base, alpha = token.split("@")
        return tokens[base][:7] + alpha
    return tokens[token]


def over(fg: str, bg: str) -> str:
    """`fg` (#rrggbb or #rrggbbaa) drawn over the opaque `bg`, as the eye sees it."""
    if len(fg) == 7:
        return fg
    a = int(fg[7:9], 16) / 255
    mix = [
        min(255, round(int(fg[i : i + 2], 16) * a + int(bg[i : i + 2], 16) * (1 - a)))
        for i in (1, 3, 5)
    ]
    return "#" + "".join(f"{c:02x}" for c in mix)


def key(k: str) -> str:
    return k if k.replace("-", "").replace("_", "").isalnum() else f'"{k}"'


def render() -> str:
    p = pal.PALS[PALETTE]
    out = [HEADER.format(name=PALETTE)]
    for variant in ("light", "dark"):
        tokens = p[variant]
        out.append(f"\n[seeds.{variant}]")
        out += [f'{seed} = "{colour(tokens, variant, tok)}"' for seed, tok in SEEDS]
    for variant in ("light", "dark"):
        tokens = p[variant]
        out.append(f"\n[{variant}]")
        for comment, roles in ROLES:
            out.append(f"# {comment}.")
            out += [f'{key(r)} = "{colour(tokens, variant, tok)}"' for r, tok in roles]
    return "\n".join(out) + "\n"


def check() -> list[str]:
    """Contrast on the mapped values: pal.py's pairs, then the app's where both roles are set here."""
    pal.build()  # asserts every palette pair
    p = pal.PALS[PALETTE]
    pairs = [
        ("text / window", "ink", "bg", 4.5),
        ("text / field", "ink", "field", 4.5),
        ("text / list", "ink", "bg", 4.5),
        ("selection text / selection", "ink", "sel", 4.5),
        ("button text / button", "ink", "surface", 4.5),
        ("button text / pressed", "ink", "press", 4.5),
        ("muted / window", "ink3", "bg", 4.5),
        ("link / window", "acct", "bg", 4.5),
        ("accent / window", "acc", "bg", 3.0),
        ("accent button text", "onout", "out", 4.5),
        ("accent button text, hot", "onout", "out2", 4.5),
        ("check mark / check", "onacc", "acc", 4.5),
        ("check border / window", "ctl", "bg", 3.0),
        ("progress bar / track", "out", "track", 3.0),
        ("chip text / chip", "chipt", "chip", 4.5),
        ("caption text / caption", "ink", "surface", 4.5),
        ("caption text inactive / caption", "ink3", "surface", 4.5),
        ("tree expander / tree", "ink3", "surface", 3.0),
        ("warning / window", None, "bg", 4.5),
        ("error / window", "err", "bg", 4.5),
        ("success / window", "okt", "bg", 4.5),
    ]
    pairs += [
        ("focus ring / window", RING, "bg", 3.0),
        ("focus ring / field", RING, "field", 3.0),
        ("focus ring / panel", RING, "surface", 3.0),
    ]
    bad = []
    rows = []
    for variant in ("light", "dark"):
        t = p[variant]
        for what, fg, bg, need in pairs:
            f = over(colour(t, variant, fg), t[bg])
            r = pal.cr(f, t[bg])
            rows.append(f"{variant:5} {what:34} {f} on {t[bg]}  {r:5.2f}  (needs {need})")
            if r < need:
                bad.append(rows[-1])
        # Issue #7: a pressed face is told from the pointer over it and from the face at rest.
        for other in ("hover", "surface"):
            if t["press"] == t[other]:
                bad.append(f"{variant:5} pressed face {t['press']} is the same as {other}")
    print("\n".join(rows))
    return bad


def main() -> None:
    OUT.write_text(render(), encoding="utf8", newline="\n")
    bad = check()
    if bad:
        sys.exit("contrast fails:\n" + "\n".join(bad))
    print(f"wrote {OUT.relative_to(REPO).as_posix()}; every pair passes")


if __name__ == "__main__":
    main()
