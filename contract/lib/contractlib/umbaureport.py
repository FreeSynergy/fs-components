"""Writers of START.md and ZUORDNUNG.md. Pattern: Template method (fixed layout, rows fill it)."""
from __future__ import annotations

from collections import Counter

from . import styledefs, umbau, vocab

CMD = "python3 -I contract/umbau/count --write contract/umbau"
ARIA_URL = "https://www.w3.org/TR/wai-aria-1.2/#"
OPENUI_URL = "https://open-ui.org/components/"


def totals(groups):
    t = Counter()
    for g in groups:
        t.update(g["fixed"])
    return t


def start_md(data, style_kinds):
    groups, t = data["groups"], totals(data["groups"])
    crates = {}
    for g in groups:
        c = crates.setdefault(g["crate"], {"fixed": Counter(), "names": set(), "ncontract": 0})
        c["fixed"].update(g["fixed"])
    names = data["names"]
    by_status = Counter(n["status"] for n in names)
    for g in groups:
        crates[g["crate"]]["ncontract"] += g["names_not_in_contract"]
    for n in names:
        for gr in n["groups"]:
            crates[gr.split("/")[0]]["names"].add(n["name"])
    out = ["# Starting state of the conversion (ADR 0074)", "",
           "Generated. Do not edit. Reproduce with `" + CMD + "` (Python 3, no dependencies; reads `crates/` and `contract/`). "
           "Production code only: tests, `#[cfg(test)]` parts and the style definitions (`styles/craft.rs`, `windows.rs`, `apple.rs`, "
           "`admin.rs`, `simple.rs`, which ARE the tokens) are not counted as violations. One group = one crate (`crates/<name>`); "
           "the module table below splits it by the first folder or file under `src/`. Workflow input `group`: a crate name.", "",
           f"Contract version {data['contract_version']}. `contract/hardcoded` finds hex, `Color::rgba(..)`/`Rgba::hex(..)` with literals, and "
           "`px` values in CSS strings and numbers in Rust calls (`.p(8.0)`, `.w(..)`). `umbau` adds CSS `rgb()/hsl()`, named CSS colours in "
           "colour properties and `Color::NAME` constants (none found), and leaves out the one place `contract/hardcoded` counts wrongly "
           "(`Live::new().gap(250)` in `message_list/handle.rs`: milliseconds, no spacing). So spacing here is one lower than the spacing count of `HARDCODED-VALUES.md`.", "",
           "## Totals", "", "| Kind | Places in blocks | Style definition values (tokens, not violations) |", "|---|---|---|"]
    for k in data["kinds"]:
        out.append(f"| {k} | {t[k]} | {style_kinds.get(k, 0)} |")
    out += ["", f"Names (class and id names of production code, distinct): **{len(names)}** = " + ", ".join(f"{k} {v}" for k, v in sorted(by_status.items())) + ". "
            "No old name is itself a contract name: every `mapped`, `provisional` and `remove` name has to change or go; `exception` names are no class "
            "(thread names, handle ids; `contract/exceptions.toml`). Names not in the contract: " + str(len(names) - by_status.get("exception (not a class)", 0)) + ".", "",
            "Colours in test code (not counted above), per crate: " + (", ".join(f"{k} {v}" for k, v in data["colors_in_tests"].items()) or "none")
            + f" = {sum(data['colors_in_tests'].values())}.", "",
            "Compared with the figures named in ADR 0074 (205 fixed colours, 474 class and id names): **not reproduced** [open]. Colours: "
            f"{t['color']} in blocks + {style_kinds.get('color', 0)} in the style definitions (tokens) + {sum(data['colors_in_tests'].values())} in tests; "
            "no combination of these is 205 without a counting rule that is not written down. Names: the distinct names of production code are "
            f"{len(names)}; 474 matches no status group.", "",
            "## Per crate (the groups)", "",
            "| Group (crate) | Colours | Spacing | Size | Other fixed (radius, font, border, shadow, opacity) | Names not in contract |", "|---|---|---|---|---|---|"]
    for crate, c in sorted(crates.items(), key=lambda i: (sum(i[1]["fixed"].values()), i[0])):
        f = c["fixed"]
        other = sum(f[k] for k in ("radius", "font", "border", "shadow", "opacity"))
        out.append(f"| `{crate}` | {f['color']} | {f['spacing']} | {f['size']} | {other} | {len([n for n in names if any(g.split('/')[0] == crate for g in n['groups']) and not n['status'].startswith('exception')])} |")
    out += ["", "A name used in two crates is counted in both rows, so the rows add up to more than the distinct total.", "",
            "## Per module", "", "| Group | Module | Colours | Spacing | Size | Other | Names not in contract |", "|---|---|---|---|---|---|---|"]
    for g in groups:
        f = g["fixed"]
        other = sum(f[k] for k in ("radius", "font", "border", "shadow", "opacity"))
        out.append(f"| `{g['crate']}` | `{g['module']}` | {f['color']} | {f['spacing']} | {f['size']} | {other} | {g['names_not_in_contract']} |")
    return "\n".join(out) + "\n"


def source_of(block):
    if block.naming == "aria":
        return f"WAI-ARIA 1.2 role `{block.ref}`: {ARIA_URL}{block.ref}" + ("" if block.ref in vocab.ARIA_ROLES else " [role not in vocab.py]")
    if block.naming == "open-ui":
        return f"Open UI component `{block.ref}`: {OPENUI_URL}" + ("" if block.ref in vocab.OPEN_UI_NAMES else " [not in vocab.py]")
    return f"own name: {block.reason}"


def blocks_section(contract):
    mapped = Counter(e.target.split(".")[0].split(":")[0].split("/")[0] for e in contract.here() if e.target != "-")
    status = Counter((e.target.split(".")[0].split(":")[0].split("/")[0], e.status) for e in contract.here() if e.target != "-")
    out = ["## 3. Blocks: today's names to the fixed contract name", "",
           f"The mapping of every single old name ({len(contract.here())} entries in `contract/mapping/*.toml`, rendered in `contract/MAPPING.md`) already exists "
           "and is checked by `contract/check`; it is not copied here. This table says per contract block where its name comes from (the contract's own `naming` and "
           "`ref`; the lists of ARIA roles and Open UI names are in `contract/lib/contractlib/vocab.py`; the Open UI list is the repository's copy, not re-read online "
           "[open]).", "", "| Block | Name from | Old names mapped (all / provisional) |", "|---|---|---|"]
    for name, b in sorted(contract.blocks.items()):
        out.append(f"| `{name}` | {source_of(b)} | {mapped[name]} / {status[(name, 'provisional')]} |")
    removed = sorted(e.name for e in contract.here() if e.status == "remove")
    out += ["", f"### Old names with status `remove` ({len(removed)}): no contract name, the place goes away or is not a visual element", "",
            ", ".join(f"`{n}`" for n in removed), "",
            f"### Provisional ({sum(1 for e in contract.here() if e.status == 'provisional')} names)", "",
            "Names whose place in the contract is provisional (the block or part is expected to change when the code is converted). They are assigned a block, "
            "but the fixed name is not final: [open] for review.", ""]
    return out


def zuordnung_md(rows, contract):
    ok = [r for r in rows if r["token"]]
    zero = [r for r in rows if r["zero"]]
    gap = [r for r in rows if not r["token"] and not r["zero"]]
    out = ["# Assignment: counted values to tokens, blocks to contract names (ADR 0074)", "",
           "Generated by `contract/umbau/count --write contract/umbau`. Do not edit.", "",
           "**Token source.** There is no DTCG token file yet (step 2). The tokens that exist are the paths the style definitions fill "
           "(`color.surface.window`, `space.s`, `font.size.body` ...; read by `contract/lib/contractlib/styledefs.py`, listed in `HARDCODED-VALUES.md` part A). "
           "A value is assigned only when it is **equal** (`4px` and `4.0` are the same value) to the Craft value of such a token. Nothing is rounded to the nearest step, because rounding changes the "
           "picture and the check of the conversion is a pixel comparison. Everything else is in the gap list, with a proposal that is not decided.", "",
           f"Distinct values: {len(rows)}; places: {sum(r['places'] for r in rows)}. Assigned to an existing token: {len(ok)} values / {sum(r['places'] for r in ok)} places. "
           f"Zero (needs no token): {len(zero)} / {sum(r['places'] for r in zero)}. Without a fitting token: {len(gap)} / {sum(r['places'] for r in gap)}.", "",
           "**Caution (style dependent).** The same token has different values in the five styles (`space.s` = 8 in Craft, Admin, Apple, Windows; 12 in Simple; "
           "`radius.control` 0 / 4 / 6 / 12 / 14). Assigning a literal to a token therefore changes the picture in every style that does not share the value. "
           "Where several tokens are listed (`one of`), the role decides [open]. Pictures are equal only in the style the literal was written for [open: which style that is per block]. Column `s?` marks a token whose value differs between styles.", "",
           "## 1. Values with an existing token", "", "| Kind | Value | Token(s) | s? | Places | Crates | Property | First place |", "|---|---|---|---|---|---|---|---|"]
    for r in ok:
        out.append(f"| {r['kind']} | `{r['value']}` | {'one of: ' if len(r['token']) > 1 else ''}{', '.join('`' + t + '`' for t in r['token'])} | {'yes' if r['style_dependent'] else 'no'} | {r['places']} | {', '.join(r['groups'])} | {', '.join(f'`{p}`' for p in r['props'] if p)} | {r['sample']} |")
    out += ["", f"Zero values ({len(zero)}): `0`, `0.0` for size, spacing, radius, border: no token needed.", "",
            "## 2. Without a fitting token (list, proposal, not invented)", "", f"Proposals: proposal only; name and scale are for the owner to decide.", "",
            "| Kind | Value | Places | Crates | Property | Proposal | First place |", "|---|---|---|---|---|---|---|"]
    for r in sorted(gap, key=lambda r: (r["kind"], -r["places"], r["value"])):
        out.append(f"| {r['kind']} | `{r['value']}` | {r['places']} | {', '.join(r['groups'])} | {', '.join(f'`{p}`' for p in r['props'] if p)} | `{r['proposal']}` | {r['sample']} |")
    return "\n".join(out + [""] + blocks_section(contract)) + "\n"


def style_kind_counts(root):
    names = {"space": "spacing"}
    return Counter(names.get(v.token.split(".")[0], v.token.split(".")[0]) for v in styledefs.read_all(root))
