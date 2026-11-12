"""HARDCODED-VALUES.md: writer and token suggestions. Pattern: Template method (fixed layout, kinds fill the suggestions)."""
from __future__ import annotations

import os
import re
from collections import Counter, defaultdict

from . import hard, styledefs

CRAFT_SPACE = ((4, "xs"), (8, "s"), (12, "m"), (20, "l"), (28, "xl"))
CRAFT_FONT = ((11, "caption"), (13, "body"), (18, "title"), (28, "headline"))
KIND_ORDER = ("color", "radius", "spacing", "font", "shadow", "border", "size", "opacity")


def number(value):
    m = re.match(r"-?\d+(?:\.\d+)?", value)
    return float(m.group(0)) if m else None


def nearest(scale, value):
    n = number(value)
    if n is None:
        return None, False
    best = min(scale, key=lambda item: abs(item[0] - n))
    return best[1], abs(best[0] - n) < 0.01


def slug(block):
    return re.sub(r"[^a-z0-9-]+", "-", block.split(" ")[0].lower()).strip("-") or "misc"


def suggest(hit, block, palette):
    base = slug(block.replace("(no block) ", "").replace("/", "-"))
    if hit.kind == "color":
        known = palette.get(hit.value)
        return known or f"color.{base}.{ {'rust': 'value'}.get(hit.via, 'value')}"
    if hit.kind == "radius":
        n = number(hit.value) or 0
        return "radius.control" if n <= 6 else "radius.card" if n <= 12 else "radius.overlay"
    if hit.kind == "spacing":
        name, exact = nearest(CRAFT_SPACE, hit.value)
        return f"space.{name}" + ("" if exact else " (nearest)") if name else f"space.{base}"
    if hit.kind == "font":
        if hit.prop in ("line-height",):
            return "font.line-height.body"
        if hit.prop == "letter-spacing":
            return f"font.letter-spacing.{base}"
        name, exact = nearest(CRAFT_FONT, hit.value)
        return f"font.size.{name}" + ("" if exact else " (nearest)") if name else f"font.size.{base}"
    if hit.kind == "border":
        return "border.width.default" if (number(hit.value) or 0) <= 1 else "border.width.strong"
    if hit.kind == "shadow":
        return "shadow.card"
    if hit.kind == "opacity":
        return "opacity.disabled" if (number(hit.value) or 1) <= 0.5 else "opacity.muted"
    return f"size.{base}.{hit.prop.replace('_', '-') or 'value'}"


def palette_tokens(values):
    """Hex of the Craft palettes -> token path, so a literal that equals a palette value points at it."""
    out = {}
    for v in values:
        if v.style == "craft" and v.token.startswith("color.") and v.value.startswith("#") and " " not in v.value:
            out.setdefault(v.value, v.token)
    return out


def style_section(values):
    out = ["## A. Style definitions: the values of the five built-in styles", "",
           "These are the style, not violations. Each row is one token of step 2 (DTCG path), with its light and dark value. "
           "File and line are the light value (or the only one).", ""]
    for style in sorted({v.style for v in values}):
        rows = defaultdict(dict)
        for v in (v for v in values if v.style == style):
            rows[v.token][v.mode] = v
        out += [f"### {style}", "", "| Token (DTCG) | Value (light / all) | Value (dark) | Where |", "|---|---|---|---|"]
        for token in sorted(rows):
            modes = rows[token]
            first = modes.get("light") or modes.get("all") or modes["dark"]
            dark = modes["dark"].value if "dark" in modes and first is not modes["dark"] else ""
            out.append(f"| `{token}` | `{first.value}` | {f'`{dark}`' if dark else ''} | {first.where} |")
        out.append("")
    return out


def block_section(hits, locate, palette):
    grouped = defaultdict(list)
    for hit in hits:
        block, how = locate.locate(hit)
        grouped[block].append((hit, how))
    out = ["## B. Fixed values inside blocks (to become tokens)", "",
           "Found by `contract/hardcoded` in the production code (not tests, not the style definitions). The block is a heuristic: "
           "the class name on the same CSS line, or within 3 lines of a Rust call (`line`), else the block most used in the file (`file`), "
           "else none (`none`, grouped by crate and file). The token is only a proposal (nearest step of the Craft scale where it says `nearest`).", ""]
    for block in sorted(grouped):
        items = grouped[block]
        counts = Counter(h.kind for h, _ in items)
        out += [f"### {block}", "", ", ".join(f"{k} {counts[k]}" for k in KIND_ORDER if counts[k]), "",
                "| Where | Kind | Value | Property | Block found by | Token proposal |", "|---|---|---|---|---|---|"]
        for h, how in sorted(items, key=lambda i: (KIND_ORDER.index(i[0].kind), i[0].rel, i[0].line, i[0].prop, i[0].value)):
            out.append(f"| {h.rel}:{h.line} | {h.kind} | `{h.value}` | {h.prop and '`' + h.prop + '`'} | {how} | `{suggest(h, block, palette)}` |")
        out.append("")
    return out


def render(root, contract, usage):
    values = styledefs.read_all(root)
    hits = hard.collect(root)
    locate = hard.BlockLocator(contract, usage)
    palette = palette_tokens(values)
    kinds = Counter(h.kind for h in hits)
    style_kinds = Counter(v.token.split(".")[0] for v in values)
    out = ["# Fixed values in the code (basis of step 2: tokens)", "",
           "Generated by `contract/hardcoded` from the source of fs-components at the commit of the contract. Do not edit; run the "
           "script again. Part A are the values of the built-in styles, which become the DTCG token files. Part B are values written "
           "into blocks, which a block must not contain (ADR 0074, section 2); each needs a token.", "",
           "| Kind | Style definitions (A, values) | In blocks (B, places) |", "|---|---|---|"]
    names = {"color": "color", "radius": "radius", "spacing": "space", "font": "font", "shadow": "shadow", "border": "border", "size": "size", "opacity": "opacity"}
    for k in KIND_ORDER:
        out.append(f"| {k} | {style_kinds.get(names[k], 0)} | {kinds.get(k, 0)} |")
    out += ["", f"Style definition values: {len(values)}. Places in blocks: {len(hits)}. Palette literals that equal a Craft palette "
            "value point at that token.", ""]
    return "\n".join(out + style_section(values) + block_section(hits, locate, palette))
