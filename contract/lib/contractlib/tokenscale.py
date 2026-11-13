"""Scales for the fixed values (ADR 0074, strategy rules 2 and 3): one scale per kind, read from the tokens the style definitions
already fill (Craft is the reference), and the fit of every counted literal onto it. Pattern: Strategy (one fit rule, kinds differ only
in table rows). Nothing is invented: a literal that no step reaches within the tolerance becomes a NEW token, named from NAMES or marked."""
from __future__ import annotations

from collections import defaultdict

from . import assign, styledefs

REFERENCE = "craft"
TOLERANCE = {"spacing": 2.0, "size": 2.0, "font": 2.0, "radius": 2.0, "border": 2.0, "opacity": 0.05}
FAMILY = {"spacing": ("space.",), "size": ("size.", "icon.size"), "font": ("font.size.",), "radius": ("radius.",), "border": (), "opacity": ()}
PX_KINDS = ("spacing", "border", "size", "font", "radius", "opacity")
# Names of new tokens: (kind, value) -> DTCG path. A value without a row gets `<family>.new-<value>` and is marked [offen].
NAMES = {("spacing", 1.5): "space.2xs", ("border", 1.0): "border.width.default", ("border", 3.0): "border.width.strong",
         ("opacity", 0.45): "opacity.disabled", ("opacity", 0.85): "opacity.soft", ("radius", 5.0): "radius.small",
         ("size", 1.0): "size.hairline", ("size", 2.0): "size.rule", ("size", 4.0): "size.track", ("size", 10.0): "size.dot",
         ("size", 13.0): "size.glyph", ("size", 24.0): "size.gutter", ("size", 56.0): "size.row-tall", ("size", 60.0): "size.row-taller",
         ("size", 64.0): "size.field-tall", ("size", 72.0): "size.col-narrow", ("size", 96.0): "size.col-s", ("size", 120.0): "size.col-m",
         ("size", 560.0): "size.dialog-max"}  # the roles are read from the places of use (TOKENS.md section 3); all names are [offen]


def fits(kind, literal, step):
    """A literal may sit on a step when it is equal, or within the tolerance and never more than half of the smaller of the two."""
    diff = abs(literal - step)
    if diff == 0:
        return True
    limit = TOLERANCE[kind]
    return diff <= limit + 1e-9 and (kind == "opacity" or diff <= 0.5 * min(literal, step) + 1e-9)


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
    return values, where


def new_steps(kind, literals, scale):
    """Greedy fewest new steps: take the literal that reaches the most still unreached places; then least total deviation, then smaller."""
    scale, added = dict(scale), []
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
    options = sorted((abs(literal - s), s, t) for t, s in scale.items() if fits(kind, literal, s))
    return (options[0][2], options[0][1]) if options else None
