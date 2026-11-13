"""Builds tokens.json (ADR 0074 strategy rules 2 to 7): scales per kind from the style definitions, every counted literal placed on a
token, a scale step or a NEW token, the deviations, and the proposed exceptions. Pattern: Builder (rows in, one data tree out)."""
from __future__ import annotations

from collections import Counter, defaultdict

from . import assign, hard, styledefs, tokenscale as ts, umbau

STYLEDEF = "crates/fs-components-styles/src/styles/"
ICON_FILES = ("crates/fs-components-widgets/src/icon_kind.rs", "crates/fs-components-widgets/src/icon_export.rs")
LAST_WAVE = {("fs-components-widgets", "plan"), ("fs-components-workbenches", "view")}
ZERO_KINDS = ("spacing", "size", "radius", "border")


def klass(hit):
    """Which rule decides the place: style definition (strategy 4), drawn icon = content (3), last wave (5), else in scope."""
    if hit.rel.startswith(STYLEDEF):
        return "style-definition"
    if hit.rel in ICON_FILES:
        return "icon-content"
    return "last-wave" if umbau.group_of(hit.rel) in LAST_WAVE else "scope"


def canon(hit):
    if hit.kind == "color":
        return assign.hex_of(hit.value) or hit.value
    return hit.value if hit.kind == "shadow" else assign.step(hit.value)


def collect(root):
    rows = defaultdict(lambda: {"classes": Counter(), "sites": defaultdict(list)})
    for hit in umbau.fixed_values(root):
        row = rows[(hit.kind, canon(hit))]
        row["classes"][klass(hit)] += 1
        row["sites"][klass(hit)].append(f"{hit.rel}:{hit.line}")
    return rows


def px_scales(root, rows, scope_only=True, tolerance=None, share=False):
    """Per kind: base steps (Craft value), new steps, and the literal -> (token, step) placement."""
    out, saved = {}, dict(ts.TOLERANCE)
    if tolerance:
        ts.TOLERANCE.update({k: tolerance for k in tolerance_kinds(tolerance)})
    try:
        for kind in ts.PX_KINDS:
            values, where = ts.base_scale(root, kind)
            base = {t: s[ts.REFERENCE] for t, s in values.items() if ts.REFERENCE in s}
            if share and kind == "size":  # variant: a box may sit on the space ladder and on a line width
                base.update({t: v for k in ("spacing", "border") for t, v in out[k]["scale"].items()})
            lits, allv = {}, {}
            for (k, v), row in rows.items():
                number = assign.number(v)
                if k != kind or number is None or (kind in ZERO_KINDS and number == 0) or (kind == "opacity" and number == 1):
                    continue
                allv[number] = row["classes"]["scope"] + row["classes"]["last-wave"]
                if row["classes"]["scope"] or not scope_only:
                    lits[number] = row["classes"]["scope"] + (0 if scope_only else row["classes"]["last-wave"])
            added = ts.new_steps(kind, {v: n for v, n in lits.items() if n}, base)
            scale = dict(base, **{a["token"]: a["value"] for a in added})
            out[kind] = {"base": base, "values": values, "where": where, "new": added, "scale": scale,
                         "place": {v: ts.place(kind, v, scale) for v in allv}}
    finally:
        ts.TOLERANCE.update(saved)
    return out


def tolerance_kinds(_value):
    return [k for k in ts.TOLERANCE if k != "opacity"]


def hexrgb(value):
    return tuple(int(value[i:i + 2], 16) for i in (1, 3, 5))


def color_map(root, value):
    """(token or None, nearest craft token, max channel distance): exact Craft equality only (no colour tolerance was given)."""
    tokens = assign.Tokens(root)
    exact = sorted(tokens.by_value.get(("color", value), ()))
    if exact:
        return exact, None, 0
    if " a=" in value or not value.startswith("#"):
        return [], None, None
    best = min(((max(abs(a - b) for a, b in zip(hexrgb(value), hexrgb(v.value))), v.token) for v in styledefs.read_all(root)
                if v.style == ts.REFERENCE and v.mode in ("light", "all") and v.token.startswith("color.") and " a=" not in v.value), default=None)
    return [], (best[1] if best else None), (best[0] if best else None)


def literal_rows(root, rows, scales):
    out = []
    for (kind, value), row in sorted(rows.items(), key=lambda x: (x[0][0], assign.number(x[0][1]) or 0, x[0][1])):
        classes = dict(row["classes"])
        entry = {"kind": kind, "value": value, "places": sum(classes.values()), "classes": classes}
        number = assign.number(value)
        if kind == "color":
            tok, near, dist = color_map(root, value)
            entry.update(token=tok, nearest=near, distance=dist, status="existing" if tok else "new-color")
        elif kind == "shadow":
            entry.update(token=[], status="exception")
        elif (kind in ZERO_KINDS and number == 0) or (kind == "opacity" and number == 1):
            entry.update(token=[], dev=0, status="none")
        else:
            placed = scales[kind]["place"].get(number)
            if placed:
                new = placed[0] in {a["token"] for a in scales[kind]["new"]}
                entry.update(token=[placed[0]], step=placed[1], dev=round(abs(number - placed[1]), 3), status="new" if new else "scale")
            else:
                entry.update(token=[], status="own-style")
        sites = {c: s for c, s in row["sites"].items() if c in ("scope", "last-wave") and entry.get("dev")}
        if sites:
            entry["sites"] = sites
        out.append(entry)
    return out


def build(root):
    rows = collect(root)
    scales = px_scales(root, rows)
    wide = px_scales(root, rows, scope_only=False)
    loose = px_scales(root, rows, tolerance=4.0)
    shared = px_scales(root, rows, share=True)
    literals = literal_rows(root, rows, scales)
    kinds = Counter()
    for (kind, _), row in rows.items():
        kinds[kind] += sum(row["classes"].values())
    new_colors = [e for e in literals if e["status"] == "new-color" and (e["classes"].get("scope"))]
    return {"rule": {"reference": ts.REFERENCE, "tolerance": ts.TOLERANCE,
                     "fit": "equal, or |diff| <= tolerance and |diff| <= half of the smaller value; tie: the smaller step",
                     "excluded": ["style-definition", "icon-content", "last-wave"]},
            "counted": dict(sorted(kinds.items())), "scales": {k: {"base": v["base"], "new": v["new"], "values": v["values"], "where": v["where"]} for k, v in scales.items()},
            "literals": literals, "new_colors": [{"value": e["value"], "places": e["classes"]["scope"], "nearest": e["nearest"], "distance": e["distance"]} for e in new_colors],
            "new_count": {"scope": {k: len(v["new"]) for k, v in scales.items()} | {"color": len(new_colors)},
                          "with_last_wave": {k: len(v["new"]) for k, v in wide.items()} | {"color": len({e["value"] for e in literals if e["status"] == "new-color" and (e["classes"].get("scope") or e["classes"].get("last-wave"))})},
                          "tolerance_4": {k: len(v["new"]) for k, v in loose.items()},
                          "size_shares_space_and_border": {k: len(v["new"]) for k, v in shared.items()}},
            "start_totals": hard_total(root)}


def hard_total(root):
    return dict(sorted(Counter(h.kind for h in umbau.fixed_values(root)).items()))
