"""Starting state of the conversion (ADR 0074): per group of blocks (crate, and module inside it) the fixed colours, the fixed
spacings and sizes, and the names that are not contract names. Pattern: Strategy (extra Finders) + Builder (rows in, report out).
Reuses `hard` (what `contract/hardcoded` finds) and `scan` (class and id names); adds CSS colour functions and named colours."""
from __future__ import annotations

import glob
import os
import re
from collections import Counter, defaultdict

from . import hard, stats
from .checks import Context
from .scan import is_test_file, scan

KINDS = ("color", "spacing", "size", "radius", "font", "border", "shadow", "opacity")
NAMED = frozenset("""aliceblue antiquewhite aqua aquamarine azure beige bisque black blanchedalmond blue blueviolet brown burlywood
cadetblue chartreuse chocolate coral cornflowerblue cornsilk crimson cyan darkblue darkcyan darkgoldenrod darkgray darkgreen darkgrey
darkkhaki darkmagenta darkolivegreen darkorange darkorchid darkred darksalmon darkseagreen darkslateblue darkslategray darkslategrey
darkturquoise darkviolet deeppink deepskyblue dimgray dimgrey dodgerblue firebrick floralwhite forestgreen fuchsia gainsboro ghostwhite
gold goldenrod gray green greenyellow grey honeydew hotpink indianred indigo ivory khaki lavender lavenderblush lawngreen lemonchiffon
lightblue lightcoral lightcyan lightgoldenrodyellow lightgray lightgreen lightgrey lightpink lightsalmon lightseagreen lightskyblue
lightslategray lightslategrey lightsteelblue lightyellow lime limegreen linen magenta maroon mediumaquamarine mediumblue mediumorchid
mediumpurple mediumseagreen mediumslateblue mediumspringgreen mediumturquoise mediumvioletred midnightblue mintcream mistyrose moccasin
navajowhite navy oldlace olive olivedrab orange orangered orchid palegoldenrod palegreen paleturquoise palevioletred papayawhip
peachpuff peru pink plum powderblue purple red rosybrown royalblue saddlebrown salmon sandybrown seagreen seashell sienna silver
skyblue slateblue slategray slategrey snow springgreen steelblue tan teal thistle tomato turquoise violet wheat white whitesmoke
yellow yellowgreen""".split())
COLOR_PROP = re.compile(hard.KINDS["color"])
DECL = re.compile(r"([a-z-]+)\s*:\s*([^;{}\"]+?)\s*[;}]")


class CssColorFinder(hard.Finder):
    """`rgb(..)`, `rgba(..)`, `hsl(..)`, `hsla(..)` with literal numbers, and named colours (`red`) in a colour property of a CSS string."""
    func = re.compile(r"(?<![\w:-])(rgba?|hsla?)\(\s*[-\d.%, /]+\)")

    def find(self, text, rel):
        for m in self.func.finditer(text):
            yield hard.Hit("color", re.sub(r"\s+", "", m.group(0)), rel, hard.line_of(text, m.start()), "", "css-fn")
        for m in DECL.finditer(text):
            if not COLOR_PROP.search(m.group(1)):
                continue
            for word in re.findall(r"(?<![\w#.-])[a-z]+(?![\w(-])", m.group(2)):
                if word in NAMED:
                    yield hard.Hit("color", word, rel, hard.line_of(text, m.start()), m.group(1), "css-name")


class ColorConstFinder(hard.Finder):
    """`Color::RED`, `Rgba::WHITE`: a named colour constant of Rust."""
    pattern = re.compile(r"\b(?:Color|Rgba)::([A-Z][A-Z0-9_]+)\b")

    def find(self, text, rel):
        for m in self.pattern.finditer(text):
            yield hard.Hit("color", m.group(0), rel, hard.line_of(text, m.start()), "", "rust-const")


EXTRA = (CssColorFinder(), ColorConstFinder())


def group_of(rel):
    """(crate, module): the module is the first file or folder under `src/` (`lib.rs` and `main.rs` count as `(root)`)."""
    parts = rel.split("/")
    crate = parts[1]
    if parts[2] != "src" or len(parts) < 4:
        return crate, f"({parts[2]})"
    first = os.path.splitext(parts[3])[0]
    return crate, "(root)" if first in ("lib", "main") and len(parts) == 4 else first


def is_nothing(hit):
    """A zero spacing, size, radius or border, and `opacity: 1`, set nothing: no token, no violation (the declaration is removed)."""
    m = re.match(r"-?\d+(?:\.\d+)?", hit.value)
    number = float(m.group(0)) if m else None
    return number is not None and ((hit.kind in ("spacing", "size", "radius", "border") and number == 0) or (hit.kind == "opacity" and number == 1))


def fixed_values(root, include_tests=False, everything=False):
    """Hits of production code (styles/*.rs definitions excluded: they are the tokens), plus the extra colour finders. Files of
    `[fixed-values]`, colours of `[data-colours]` and values that set nothing are left out, unless `everything` (the token report)."""
    exempt = not everything
    ex = hard.exemptions(root) if exempt else {"colours": {}}
    hits = list(hard.collect(root, exempt))
    for rel, path in hard.source_files(root, exempt):
        text = hard.strip_tests(open(path, encoding="utf-8").read())
        for finder in EXTRA:
            hits.extend(h for h in finder.find(text, rel) if hard.keep(h, ex))
    return sorted((h for h in set(hits) if not is_false_positive(root, h) and (everything or not is_nothing(h))), key=lambda h: (h.rel, h.line, h.kind, h.value))


# `contract/hardcoded` counts them, but they are no layout value: `Live::new().gap(250)` is a pause in milliseconds.
FALSE_POSITIVE = re.compile(r"Live::new\(\)\.gap\(")


def is_false_positive(root, hit):
    return hit.via == "rust" and bool(FALSE_POSITIVE.search(hard.lines_of(root, hit.rel)[hit.line - 1]))


def test_colors(root):
    """Colours in test files and `#[cfg(test)]` parts per crate (not violations; shown so the total can be compared with other counts)."""
    found = Counter()
    paths = glob.glob(os.path.join(root, "crates/*/src/**/*.rs"), recursive=True) + glob.glob(os.path.join(root, "crates/*/tests/**/*.rs"), recursive=True)
    for path in sorted(set(paths)):
        rel = os.path.relpath(path, root)
        text = open(path, encoding="utf-8").read()
        part = text if is_test_file(rel) else text[len(hard.strip_tests(text)):]
        for finder in (hard.HexFinder(), hard.ColorCallFinder(), *EXTRA):
            found[rel.split("/")[1]] += sum(1 for _ in finder.find(part, rel))
    return {k: v for k, v in sorted(found.items()) if v}


def name_rows(ctx):
    """Every class or id name of production code with its status and the groups that use it."""
    rows = []
    for name, sites in sorted(ctx.usage.src.items()):
        status = stats.classify(ctx, name)
        groups = sorted({"/".join(group_of(s.rpartition(":")[0])) for s in sites})
        rows.append({"name": name, "status": status, "uses": len(sites), "groups": groups})
    return rows


def build(root, contract):
    ctx = Context(contract, scan(root, contract.meta["contract"]["prefixes"]))
    table = defaultdict(lambda: {"fixed": Counter(), "names": Counter(), "name_set": set(), "files": set()})
    for hit in fixed_values(root):
        crate, module = group_of(hit.rel)
        table[(crate, module)]["fixed"][hit.kind] += 1
        table[(crate, module)]["files"].add(hit.rel)
    for name, sites in ctx.usage.src.items():
        status = stats.classify(ctx, name)
        for site in sites:
            key = group_of(site.rpartition(":")[0])
            table[key]["name_set"].add(name)
        for key in {group_of(s.rpartition(":")[0]) for s in sites}:
            table[key]["names"][status] += 1
    groups = []
    for (crate, module), row in sorted(table.items()):
        names = row["names"]
        groups.append({"crate": crate, "module": module, "fixed": {k: row["fixed"].get(k, 0) for k in KINDS},
                       "names": dict(sorted(names.items())), "names_total": len(row["name_set"]),
                       "names_not_in_contract": sum(v for s, v in names.items() if not s.startswith("exception"))})
    return {"commit_note": "counted from the working tree; see START.md for the command", "contract_version": contract.meta["contract"]["version"],
            "kinds": list(KINDS), "groups": groups, "names": name_rows(ctx), "colors_in_tests": test_colors(root)}
