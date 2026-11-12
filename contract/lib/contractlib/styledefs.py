"""The values inside the built-in style definitions (`fs-components-styles/src/styles/*.rs`). They ARE the style, so they are
not violations: they become the DTCG token files of step 2. Pattern: Interpreter (a small reader of the struct literals)."""
from __future__ import annotations

import os
import re
from collections import namedtuple

Value = namedtuple("Value", "style token mode value where")

BARE = re.compile(r"^(\s*)(Extras)\s*\{\s*$")
OPEN = re.compile(r"^(\s*)(\w+):\s*(?:Some\()?([\w:]+)\s*\{\s*$")
FIELD = re.compile(r"^\s*(\w+):\s*(.+?),?\s*$")
COLOR = re.compile(r"^(?:c|Rgba::hex)\((0x[0-9A-Fa-f_]+|0)\)(?:\.alpha\((0x[0-9A-Fa-f]+|\d+)\))?$")
NUMBER = re.compile(r"^-?\d+(?:\.\d+)?$")
ITEM = re.compile(r"^\s*c\((0x[0-9A-Fa-f]+)\),?\s*$")

PALETTE = {"ground": "color.surface.window", "panel": "color.surface.panel", "field": "color.surface.field",
           "text": "color.text.primary", "text_weak": "color.text.secondary", "border": "color.border.default",
           "accent": "color.accent.default", "on_accent": "color.accent.on", "focus": "color.focus.ring",
           "hover": "color.state.hover", "selected": "color.state.selected", "error": "color.status.error",
           "success": "color.status.success"}
EXTRAS = {"line": "color.border.line", "line_strong": "color.border.strong", "deep": "color.surface.deep",
          "text_dim": "color.text.dim", "badge_update": "color.badge.update", "badge_offer": "color.badge.offer",
          "badge_done": "color.badge.done"}
SECTIONS = {"spacing": "space", "heights": "size", "text": "font", "radii": "radius", "icons": "icon"}
TEXT = {"caption": "font.size.caption", "body": "font.size.body", "body_line": "font.line-height.body",
        "title": "font.size.title", "headline": "font.size.headline", "label_weight": "font.weight.label"}
SHADOWS = {"card_shadow": "shadow.card", "overlay_shadow": "shadow.overlay"}


def token(path, name, seen):
    """DTCG path and mode for the field `name` inside the nested struct names `path`; None if it is no token."""
    head = path[0] if path else ""
    if head in ("light", "dark") and len(path) == 1:
        return PALETTE.get(name), head
    if path and path[-1] == "Extras":
        mode = "dark" if seen == 1 else "light"
        return EXTRAS.get(name), mode
    if head in ("effects", "effects_dark"):
        mode = "dark" if head == "effects_dark" else "light"
        if len(path) == 2 and path[1] in SHADOWS:
            return f"{SHADOWS[path[1]]}.{name}", mode
        if len(path) == 2 and path[1] == "focus":
            return f"border.focus.{name}", mode
        if len(path) == 2 and path[1] in ("chrome", "overlay"):
            return (f"color.surface.{path[1]}.tint" if name == "tint" else f"surface.{path[1]}.{name}"), mode
        return None, mode
    if head == "text" and len(path) == 1:
        return TEXT.get(name), "all"
    if head in SECTIONS and len(path) == 1:
        return f"{SECTIONS[head]}.{name.replace('_', '-')}", "all"
    return None, "all"


def hexed(match):
    value = "#%06x" % int(match.group(1).replace("_", ""), 16) if match.group(1) != "0" else "#000000"
    return value + (" a=%s" % match.group(2) if match.group(2) else "")


def read_style(root, rel):
    style = os.path.basename(rel)[:-3]
    out, path, seen = [], [], 0
    list_name, index = None, 0
    for number, line in enumerate(open(os.path.join(root, rel), encoding="utf-8"), 1):
        if line.lstrip().startswith("//") or line.startswith("#[cfg(test)]"):
            if line.startswith("#[cfg(test)]"):
                break
            continue
        head = re.match(r"^pub const (ACCENTS(?:_TEXT)?):", line)
        if head:
            list_name, index = head.group(1), 0
        item = ITEM.match(line) if list_name else None
        if item:
            out.append(Value(style, f"color.accent.{'tile-text' if list_name.endswith('TEXT') else 'tile'}.{index}", "all", "#%06x" % int(item.group(1), 16), f"{rel}:{number}"))
            index += 1
            continue
        if list_name and line.startswith("];"):
            list_name = None
        bare = BARE.match(line)
        if bare:
            path.append((len(bare.group(1)), "Extras"))
            seen += 1
            continue
        opened = OPEN.match(line)
        if opened:
            path.append((len(opened.group(1)), opened.group(2)))
            continue
        if re.match(r"^\s*\}\)?,?\s*$", line) and path and len(line) - len(line.lstrip()) == path[-1][0]:
            path.pop()
            continue
        field = FIELD.match(line)
        if not field or not path:
            continue
        name, raw = field.groups()
        found, mode = token([p[1] for p in path], name, seen)
        if not found:
            continue
        color = COLOR.match(raw)
        if color:
            out.append(Value(style, found, mode, hexed(color), f"{rel}:{number}"))
        elif NUMBER.match(raw):
            out.append(Value(style, found, mode, raw, f"{rel}:{number}"))
    return out


def read_all(root):
    base = "crates/fs-components-styles/src/styles"
    names = sorted(n for n in os.listdir(os.path.join(root, base)) if n.endswith(".rs") and n not in ("mod.rs", "craft_sheet.rs"))
    return [v for n in names for v in read_style(root, f"{base}/{n}")]
