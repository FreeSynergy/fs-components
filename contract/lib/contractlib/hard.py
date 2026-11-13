"""Fixed values in the code of the blocks: colours, radii, spacing, font sizes, shadows, sizes, opacity.
Pattern: Strategy (each Finder looks for one kind of value) + Visitor-like grouping by contract block."""
from __future__ import annotations

import glob
import os
import re
import tomllib
from abc import ABC, abstractmethod
from collections import Counter, namedtuple

Hit = namedtuple("Hit", "kind value rel line prop via")

STYLE_FILES = ("craft.rs", "windows.rs", "apple.rs", "admin.rs", "simple.rs")
NUM = r"-?\d+(?:\.\d+)?"
KINDS = {"color": r"(?:^|-)color$|^background(?:-color)?$|^fill$|^stroke$", "radius": r"^border(?:-[a-z]+)*-radius$", "spacing": r"^(?:padding|margin)(?:-[a-z]+)?$|^(?:row-|column-)?gap$",
         "font": r"^font-size$|^line-height$|^letter-spacing$", "shadow": r"^box-shadow$", "border": r"^border(?:-[a-z]+)?-width$",
         "size": r"^(?:min-|max-)?(?:width|height)$", "opacity": r"^opacity$"}
RUST_CALLS = {"spacing": "p px py pt pb pl pr m mx my mt mb ml mr gap gap_x gap_y", "radius": "rounded rounded_t rounded_b rounded_l rounded_r",
              "font": "text_size font_size", "size": "h w min_h min_w max_h max_w size", "border": "border", "opacity": "opacity"}


def strip_tests(text):
    cut = text.find("#[cfg(test)]")
    return text if cut < 0 else text[:cut]


class Finder(ABC):
    """Interface: text of one file in, hits out."""

    @abstractmethod
    def find(self, text: str, rel: str):
        ...


def line_of(text, offset):
    return text.count("\n", 0, offset) + 1


class HexFinder(Finder):
    """`#rrggbb` in strings and CSS (not attributes `#[..]`)."""
    pattern = re.compile(r"(?<![\w&])#([0-9a-fA-F]{8}|[0-9a-fA-F]{6}|[0-9a-fA-F]{3})(?![\w])")

    def find(self, text, rel):
        for m in self.pattern.finditer(text):
            line_start = text.rfind("\n", 0, m.start()) + 1
            if text[line_start:m.start()].lstrip().startswith("//") or text[m.start() - 1:m.start()] == "[":
                continue
            yield Hit("color", "#" + m.group(1).lower(), rel, line_of(text, m.start()), "", "css")


class ColorCallFinder(Finder):
    """`Color::rgba(0.5, 0.5, 0.5, 1.0)` and `Rgba::hex(0x..)` with only literal numbers, also over several lines."""
    pattern = re.compile(r"(?:Color|Rgba)::(rgba?|hex)\(([^()]*)\)")

    def find(self, text, rel):
        for m in self.pattern.finditer(text):
            args = [a.strip() for a in m.group(2).split(",") if a.strip()]
            if args and all(re.fullmatch(rf"{NUM}|0x[0-9A-Fa-f_]+", a) for a in args):
                yield Hit("color", f"{m.group(1)}({', '.join(args)})", rel, line_of(text, m.start()), "", "rust")


class CssFinder(Finder):
    """Declarations `prop: value;` of CSS strings: px values and bare numbers (not var(..), not percent)."""
    decl = re.compile(r"([a-z-]+)\s*:\s*([^;{}\"]+?)\s*[;}]")

    def find(self, text, rel):
        for m in self.decl.finditer(text):
            prop = m.group(1)
            kind = next((k for k, pat in KINDS.items() if re.search(pat, prop)), None)
            if kind in (None, "color") or "var(" in m.group(2) and not re.search(r"\d+px", m.group(2)):
                continue
            for number in re.finditer(rf"(?<![\w.#-])({NUM})(px)?(?![\w%.])", re.sub(r"var\([^)]*\)", "", m.group(2))):
                if number.group(2) or kind in ("opacity",):
                    yield Hit(kind, number.group(1) + (number.group(2) or ""), rel, line_of(text, m.start()), prop, "css")


class RustCallFinder(Finder):
    """`.p(8.0)`, `.rounded(4.0)`, `.text_size(12.0)` ... with a literal number."""

    def __init__(self):
        self.patterns = [(k, re.compile(r"\.(%s)\(\s*(%s)(?:_f32)?\s*[,)]" % ("|".join(names.split()), NUM))) for k, names in RUST_CALLS.items()]
        self.shadow = re.compile(r"\.(shadow\w*)\(")

    def find(self, text, rel):
        for kind, pattern in self.patterns:
            for m in pattern.finditer(text):
                yield Hit(kind, m.group(2), rel, line_of(text, m.start()), m.group(1), "rust")
        for m in self.shadow.finditer(text):
            yield Hit("shadow", "shadow call", rel, line_of(text, m.start()), m.group(1), "rust")


FINDERS = (HexFinder(), ColorCallFinder(), CssFinder(), RustCallFinder())


def exemptions(root):
    """`[fixed-values]` (a whole file is no violation) and `[data-colours]` (its colours are content) of contract/exceptions.toml:
    {path: reason}. The reason starts with `style:` (a style definition) or `content:` (a drawing of its own)."""
    path = os.path.join(root, "contract", "exceptions.toml")
    data = tomllib.load(open(path, "rb")) if os.path.exists(path) else {}
    return {"fixed": dict(data.get("fixed-values", {})), "colours": dict(data.get("data-colours", {}))}


def source_files(root, exempt=True):
    skip = exemptions(root)["fixed"] if exempt else {}
    for path in sorted(glob.glob(os.path.join(root, "crates/*/src/**/*.rs"), recursive=True)):
        rel = os.path.relpath(path, root)
        base = os.path.basename(rel)
        if "/tests/" in "/" + rel or base == "tests.rs" or base.endswith("_tests.rs"):
            continue
        if "/styles/" in rel and base in STYLE_FILES:
            continue  # style definitions: read by styledefs, they are the values of the styles
        if rel in skip:
            continue
        yield rel, path


def keep(hit, ex):
    return not (hit.kind == "color" and hit.rel in ex["colours"])


def collect(root, exempt=True):
    hits, ex = [], exemptions(root) if exempt else {"colours": {}}
    for rel, path in source_files(root, exempt):
        text = strip_tests(open(path, encoding="utf-8").read())
        for finder in FINDERS:
            hits.extend(h for h in finder.find(text, rel) if keep(h, ex))
    return sorted(set(hits), key=lambda h: (h.rel, h.line, h.kind, h.value, h.prop, h.via))


def lines_of(root, rel):
    return strip_tests(open(os.path.join(root, rel), encoding="utf-8").read()).split("\n")


class BlockLocator:
    """Which contract block a hit belongs to: the nearest class name in the same CSS line or within three lines of a Rust
    call, else the block most used in the file, else the crate (the answer is a heuristic and the table says so)."""

    def __init__(self, contract, usage):
        self.block_of = {e.name: e.target.split(".")[0].split(":")[0].split("/")[0] for e in contract.here() if e.target != "-"}
        self.by_file = {}
        for name, sites in usage.src.items():
            if name in self.block_of:
                for site in sites:
                    rel, _, line = site.rpartition(":")
                    self.by_file.setdefault(rel, []).append((int(line), self.block_of[name]))

    def locate(self, hit):
        """Returns (block, how) with how = line | file | none."""
        near = [(abs(line - hit.line), b) for line, b in self.by_file.get(hit.rel, []) if abs(line - hit.line) <= (0 if hit.via == "css" else 3)]
        if near:
            return min(near)[1], "line"
        counts = Counter(b for _, b in self.by_file.get(hit.rel, []))
        if counts:
            return counts.most_common(1)[0][0], "file"
        parts = hit.rel.split("/")
        return f"(no block) {parts[1].replace('fs-components-', '')}/{os.path.splitext(parts[-1])[0]}", "none"
