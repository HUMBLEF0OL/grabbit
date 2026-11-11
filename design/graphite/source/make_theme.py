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

# The toolbar's faces. Its band is the window (`bg`) until plan 05 puts it on a panel, and
# `hover` is made for panels (`surface`): on `bg` it is 5 apart in light, not seen. So the
# faces are the window shaded toward the ink, as much as `hover` is from `surface` (about 14),
# pressed twice that, the other half of a split button half of it.
TOOLBAR_HOT = ("mix", "bg", "ink", 0.06)
TOOLBAR_PRESSED = ("mix", "bg", "ink", 0.12)
TOOLBAR_FAINT = ("mix", "bg", "ink", 0.03)

# Faces that must be told apart by eye (pal.STEP), beyond pal.py's own (pal.APART).
FACES_APART = [
    ("toolbar hot / band", TOOLBAR_HOT, "bg"),
    ("toolbar pressed / hot", TOOLBAR_PRESSED, TOOLBAR_HOT),
    ("toolbar faint / band", TOOLBAR_FAINT, "bg"),
    ("toolbar faint / hot", TOOLBAR_FAINT, TOOLBAR_HOT),
]

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

# (comment, [(role, token)]); a token "ink@0f" is the ink at that alpha, ("mix", a, b, t) the
# token a moved toward b by t (0 to 1), "none" transparent. Roles left out are derived from the
# seeds (crates/app/src/derive.rs), which tints them with the accent: every role a painter of
# the chosen controls reads in an outlaw colour or a neutral is set here.
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
    # Pressed is `press`, told from the pointer's `hover` (issue #7: never the same), its label
    # the secondary ink; an accent button pressed is `outp`, a step deeper than `out`.
    ("Buttons: the default one outlined in the outlaw colour, an accent one filled with it", [
        ("button", "surface"), ("button.border", "line"), ("button.bottom", "line"),
        ("button.hot", "hover"), ("button.pressed", "press"), ("button.text.pressed", "ink2"),
        ("button.default.border", "out"), ("button.default.bottom", "out"),
        ("button.accent", "out"), ("button.accent.bottom", "out"),
        ("button.accent.text", "onout"), ("button.accent.hot", "out2"),
        ("button.accent.pressed", "outp"), ("button.accent.text.pressed", "onout"),
    ]),
    ("Check boxes and radios: the border kept under the pointer, fainter pressed", [
        ("check.on", "acc"), ("check.mark", "onacc"), ("check.border", "ctl"),
        ("check.border.hot", "ctl"), ("check.border.pressed", ("mix", "ctl", "surface", 0.3)),
        ("check.border.disabled", "dis"),
    ]),
    ("Progress: the outlaw colour on the track", [
        ("progress.track", "track"), ("meter.track", "track"), ("progress.bar", "out"),
        ("meter.fill", "out"), ("segments.done", "out"), ("segments.missing", "track"),
        ("pieces.have", "out"), ("pieces.partial", ("mix", "out", "surface", 0.55)),
        ("pieces.missing", "track"),
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
    ("Toolbar: a neutral face under the pointer, no border", [
        ("toolbar.hot", TOOLBAR_HOT), ("toolbar.pressed", TOOLBAR_PRESSED),
        ("toolbar.hot.faint", TOOLBAR_FAINT), ("toolbar.hot.edge", "line"),
        ("toolbar.hot.border", "none"),
        ("toolbar.text.disabled", "dis"), ("icon.disabled", "dis"),
    ]),
    ("Categories tree", [("tree", "surface"), ("tree.expander", "ink3")]),
    ("The drawn title bar: the panel colour, its buttons a tint of the ink (6 %, 12 %)", [
        ("caption", "surface"), ("caption.text", "ink"), ("caption.text.inactive", "ink3"),
        ("caption.hot", "ink@0f"), ("caption.pressed", "ink@1f"),
    ]),
    # Plan 02b: the roles the Graphite layout (chrome = "graphite") draws with; nothing reads
    # them before 05b, 07a and 09a.
    ("The Graphite layout: panels, icon tiles, shortcuts, danger buttons, banners", [
        ("panel", "surface"), ("tile", "tile"), ("menu.shortcut", "ink3"),
        ("toolbar.checked", "sel"), ("button.danger", "dng"), ("button.danger.text", "ondng"),
        ("banner.error", "errSoft"), ("banner.info", "sel"), ("banner.warning", "warnSoft"),
    ]),
    ("Two-tone finish and glow (09a): the outlaw colour's lighter stop, a glow in dark only", [
        ("glow", "glow"), ("button.accent.top", "out2"), ("progress.bar.end", "out2"),
    ]),
]

HEADER = """\
# Graphite: one of Mister Download Manager's built-in themes, its own new look (design/graphite):
# colours and shapes of our own design, light and dark (jet black, royal amethyst, electric
# cyan); it borrows nothing but its MIT-licensed Tabler icons (graphite/icons/SOURCES.txt). The
# typeface is the app's usual one for now.
# It is also an example of a theme file: seeds for the colours, many roles set by hand (the
# outlaw colour on the default button and the progress bars), rounded corners, Windows 11's
# controls with a focus ring, taller list rows, a flat drawn title bar on Windows and an icon
# set in one colour.
# Written by design/graphite/source/make_theme.py from source/pal.py, palette "{name}": edit
# the palette or the mapping there, not here.

name = "Graphite"
icon-set = "graphite"
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
# Graphite's layout (title-bar menus, a toolbar on a panel, dialog footers), not IDM's.
chrome = "graphite"

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
    if token == "none" or tokens.get(token) == "transparent":
        return "#00000000"
    if isinstance(token, tuple):
        _, a, b, t = token
        x, y = colour(tokens, variant, a), colour(tokens, variant, b)
        return "#" + "".join(
            f"{round(int(x[i : i + 2], 16) * (1 - t) + int(y[i : i + 2], 16) * t):02x}"
            for i in (1, 3, 5)
        )
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
        ("button text pressed / pressed", "ink2", "press", 4.5),
        ("muted / window", "ink3", "bg", 4.5),
        ("link / window", "acct", "bg", 4.5),
        ("accent / window", "acc", "bg", 3.0),
        ("accent button text", "onout", "out", 4.5),
        ("accent button text, hot", "onout", "out2", 4.5),
        ("accent button text, pressed", "onout", "outp", 4.5),
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
        ("menu shortcut / menu", "ink3", "surface", 4.5),
        ("danger button text / danger", "ondng", "dng", 4.5),
        ("text / error banner", "ink", "errSoft", 4.5),
        ("text / info banner", "ink", "sel", 4.5),
        ("text / warning banner", "ink", "warnSoft", 4.5),
        ("text / tile", "ink", "tile", 4.5),
        ("toolbar label / checked", "ink2", "sel", 4.5),
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
        # Issue #7: a pressed face is told by eye from the pointer over it, from the face at
        # rest and from its border (pal.py asks it of every palette; here, for this one).
        apart = [(f"{a} / {b}", a, b) for a, b in pal.APART] + FACES_APART
        for what, a, b in apart:
            x, y = colour(t, variant, a), colour(t, variant, b)
            s = pal.step(x, y)
            rows.append(f"{variant:5} {what:34} {x} vs {y}  step {s}  (needs {pal.STEP})")
            if s < pal.STEP:
                bad.append(rows[-1])
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
