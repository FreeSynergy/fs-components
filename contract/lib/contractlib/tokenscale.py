"""Scales for the fixed values (ADR 0074, strategy rules 2 and 3): one scale per kind, read from the tokens the style definitions
already fill (Craft is the reference), and the fit of every counted literal onto it. Pattern: Strategy (one fit rule, kinds differ only
in table rows). Nothing is invented: a literal that no step reaches within the tolerance becomes a NEW token, named from NAMES or marked."""
from __future__ import annotations

import os
from collections import defaultdict

from . import assign, styledefs

REFERENCE = "craft"
TOLERANCE = {"spacing": 2.0, "size": 2.0, "font": 2.0, "radius": 2.0, "border": 2.0, "opacity": 0.05}
FAMILY = {"spacing": ("space.",), "size": ("size.", "icon.size"), "font": ("font.size.",), "radius": ("radius.",), "border": (), "opacity": ()}
PX_KINDS = ("spacing", "border", "size", "font", "radius", "opacity")
# Names of new tokens: (kind, value) -> DTCG path. A value without a row gets `<family>.new-<value>` and is marked [offen].
NAMES = {("spacing", 1.0): "space.px", ("border", 1.0): "border.width.default", ("border", 2.0): "border.width.strong",
         ("border", 4.0): "border.width.indicator", ("opacity", 0.45): "opacity.disabled", ("opacity", 0.85): "opacity.soft",
         ("size", 4.0): "size.track", ("size", 10.0): "size.dot", ("size", 13.0): "size.glyph", ("size", 24.0): "size.gutter",
         ("size", 56.0): "size.row-tall", ("size", 60.0): "size.row-taller", ("size", 64.0): "size.field-tall", ("size", 72.0): "size.col-narrow",
         ("size", 96.0): "size.col-s", ("size", 120.0): "size.col-m", ("size", 420.0): "size.dialog-narrow", ("size", 560.0): "size.dialog-max"}  # roles read from the places of use; all [offen]
# Steps fixed by role (a line width is not a ladder): the three roles of a border. They count as new tokens.
PINNED = {"border": ((1.0, "border.width.default"), (2.0, "border.width.strong"), (4.0, "border.width.indicator"))}
# The ladder of the bridge (contract/umbau/TOKENS.md section 2): Blinc `--space-<k>` = xs * k (style_theme.rs, blinc_theme spacing.rs).
LADDER = (0.5, 1, 1.5, 2, 2.5, 3, 3.5, 4, 5, 6, 7, 8, 9, 10, 11, 12, 14, 16, 20, 24, 28, 32)
STYLE_THEME = "crates/fs-components-render/src/style_theme.rs"
BLINC_SPACING = "blinc_theme 0.6.0 src/tokens/spacing.rs:90-115"


def fits(kind, literal, step):
    """A literal may sit on a step when it is equal, or within the tolerance and never more than half of the smaller of the two."""
    diff = abs(literal - step)
    if diff == 0:
        return True
    limit = TOLERANCE[kind]
    return diff <= limit + 1e-9 and (kind == "opacity" or diff <= 0.5 * min(literal, step) + 1e-9)


def ladder_name(k):
    return "space.step." + f"{k:g}".replace(".", "-")


def blinc_var(k):
    return "--space-" + f"{k:g}".replace(".", "_")


def base_scale(root, kind):
    """{token: {style: value}} and {token: {style: where}} of the tokens that already make the scale of `kind`."""
    values, where = defaultdict(dict), defaultdict(dict)
    for v in styledefs.read_all(root):
        if v.mode == "dark" or not v.token.startswith(FAMILY[kind] or ("-",)):
            continue
        number = assign.number(v.value)
        if number is not None:
            values[v.token][v.style] = number
            where[v.token][v.style] = v.where
    if kind == "spacing":  # the bridge: every style derives `--space-k` as xs * k
        ref = next((i for i, l in enumerate(open(os.path.join(root, STYLE_THEME), encoding="utf-8"), 1) if "SpacingTokens::with_base" in l), 0)
        for k in LADDER:
            token = ladder_name(k)
            values[token] = {st: x * k for st, x in values["space.xs"].items()}
            where[token] = {st: f"{STYLE_THEME}:{ref} x {k:g}; {BLINC_SPACING}" for st in values[token]}
    if kind == "font":  # text-xl is the title size plus 2 (style_theme.rs typography_of)
        values["font.size.xl"] = {st: v + 2 for st, v in values["font.size.title"].items()}
        where["font.size.xl"] = {st: f"{STYLE_THEME} typography_of: title + 2" for st in values["font.size.xl"]}
    return values, where


def new_steps(kind, literals, scale):
    """Greedy fewest new steps: take the literal that reaches the most still unreached places; then least total deviation, then smaller."""
    scale, added = dict(scale), []
    for value, name in PINNED.get(kind, ()):
        scale[name] = value
        added.append({"token": name, "value": value, "named": True})
    while True:
        open_ = {v: n for v, n in literals.items() if not any(fits(kind, v, s) for s in scale.values())}
        if not open_:
            return added
        best = None
        for cand in sorted(set(literals) - set(scale.values())):
            reach = sum(n for v, n in open_.items() if fits(kind, v, cand))
            if not reach:
                continue
            pool = list(scale.values()) + [cand]
            dev = sum(n * min(abs(v - s) for s in pool if fits(kind, v, s)) for v, n in literals.items() if any(fits(kind, v, s) for s in pool))
            key = (-reach, dev, cand)
            best = key if best is None or key < best else best
        value = best[2]
        name = NAMES.get((kind, value), f"{FAMILY[kind][0].rstrip('.') if FAMILY[kind] else kind}.new-{value:g}")
        scale[name] = value
        added.append({"token": name, "value": value, "named": (kind, value) in NAMES})


def place(kind, literal, scale):
    """(token, step value) of the nearest fitting step (tie: the smaller), or None."""
    options = sorted((abs(literal - s), s, ".step." in t, t) for t, s in scale.items() if fits(kind, literal, s))
    return (options[0][3], options[0][1]) if options else None
