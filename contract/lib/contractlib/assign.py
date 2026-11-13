"""Assign every counted fixed value to a token of the contract (the token names of the five built-in styles), or list it as a gap.
Pattern: Strategy (one matcher per kind) over the hits of `umbau`. A value is matched only when it is EQUAL to the value of an
existing token in the Craft style (the style the literals were written for: [open], see PLAN.md); nothing is rounded, nothing invented."""
from __future__ import annotations

import re
from collections import defaultdict

from . import hard, hardreport, styledefs, umbau




def number(value):
    m = re.match(r"-?\d+(?:\.\d+)?", value)
    return float(m.group(0)) if m else None


def hex_of(value):
    """Canonical `#rrggbb[ a=0x..]` of a counted colour (hex string, `hex(0x..)` or `rgba(r,g,b,a)` floats), else None."""
    if value.startswith("#"):
        return value if len(value) in (7, 9) else None
    m = re.fullmatch(r"(?:Rgba|Color)?(?:::)?hex\((0x[0-9A-Fa-f_]+)\)", value)
    if m:
        return "#%06x" % (int(m.group(1).replace("_", ""), 16) & 0xFFFFFF)
    m = re.fullmatch(r"rgba?\(([\d.]+),([\d.]+),([\d.]+)(?:,([\d.]+))?\)", value.replace(" ", ""))
    if m and all(float(x) <= 1.0 for x in m.groups(default="1")):
        r, g, b = (round(float(x) * 255) for x in m.groups()[:3])
        base = "#%02x%02x%02x" % (r, g, b)
        alpha = float(m.group(4) or 1)
        return base if alpha == 1 else f"{base} a={alpha}"
    return None


class Tokens:
    """Token name -> {style: value}, from the style definitions (the only token source today: no DTCG file exists yet)."""

    def __init__(self, root):
        self.by_value = defaultdict(set)  # (family, value) -> tokens (Craft values)
        self.all_styles = defaultdict(set)
        self.spread = {}
        values = defaultdict(dict)
        for v in styledefs.read_all(root):
            values[v.token].setdefault(v.style, set()).add(v.value)
            family = v.token.split(".")[0]
            key = (family, v.value if family == "color" else number(v.value))
            self.all_styles[key].add((v.token, v.style))
            if v.style == "craft":
                self.by_value[key].add(v.token)
        for token, styles in values.items():
            self.spread[token] = len({frozenset(s) for s in styles.values()}) > 1

    def find(self, kind, value):
        family = {"spacing": "space", "size": "size", "font": "font", "color": "color"}.get(kind)
        if family is None:
            return []
        key = (family, value if kind == "color" else number(value))
        exact = sorted(self.by_value.get(key, ()))
        if kind == "font":
            exact = [t for t in exact if t.startswith(("font.size", "font.line-height"))]
        elif kind in ("spacing", "size"):
            exact = [t for t in exact if t.startswith("space." if kind == "spacing" else "size.")]
        elif kind == "color":
            exact = [t for t in exact if not t.startswith("color.accent.tile")] or exact
        if not exact and kind == "color":
            exact = sorted({t for t, _ in self.all_styles.get(key, ())})  # equals a value of another style
        return exact


def step(value):
    n = number(value)
    return f"{n:g}" if n is not None else value


def proposal(hit, block, palette):
    base = hardreport.slug(block.replace("(no block) ", "").replace("/", "-"))
    if hit.kind == "color":
        return f"color.{base}.<part> (one token per part and state; the name is for the owner)"
    if hit.kind in ("spacing", "size"):
        return f"{'space' if hit.kind == 'spacing' else 'size'}.{step(hit.value)} (a step with exactly this value, or a role name)"
    if hit.kind == "font":
        return "font.letter-spacing.*" if hit.prop == "letter-spacing" else f"font.size.{step(hit.value)} (a step with exactly this value)"
    if hit.kind == "radius":
        return "radius.control, radius.card or radius.overlay by role (the literal is one style's value, the role is [open])"
    if hit.kind == "border":
        return "border.width.default" if (number(hit.value) or 0) <= 1 else "border.width.strong"
    if hit.kind == "shadow":
        return "shadow.card or shadow.overlay by role [open]"
    return f"opacity.<role> (disabled, muted ...) for {step(hit.value)} [open]"


def assign(root, contract, usage):
    tokens, palette = Tokens(root), hardreport.palette_tokens(styledefs.read_all(root))
    locate = hard.BlockLocator(contract, usage)
    rows = defaultdict(lambda: {"places": 0, "groups": set(), "props": set(), "block": set(), "sample": ""})
    for hit in umbau.fixed_values(root):
        canon = hex_of(hit.value) if hit.kind == "color" else (step(hit.value) if hit.kind != "shadow" else hit.value)
        value = canon or hit.value
        block, _ = locate.locate(hit)
        row = rows[(hit.kind, value)]
        row["places"] += 1
        row["groups"].add(umbau.group_of(hit.rel)[0].replace("fs-components-", ""))
        row["props"].add(hit.prop or hit.via)
        row["block"].add(block)
        row["sample"] = row["sample"] or f"{hit.rel}:{hit.line}"
        row["hit"] = hit
    out = []
    for (kind, value), row in sorted(rows.items()):
        zero = kind in ("spacing", "size", "border", "radius") and number(value) == 0
        found = [] if zero else tokens.find(kind, value)
        spread = bool(found) and any(tokens.spread.get(t) for t in found)
        block = sorted(row["block"])[0]
        out.append({"kind": kind, "value": value, "token": found, "zero": zero, "style_dependent": spread,
                    "places": row["places"], "groups": sorted(row["groups"]), "props": sorted(row["props"]), "sample": row["sample"],
                    "proposal": "" if found or zero else proposal(row["hit"], block, palette)})
    return out
