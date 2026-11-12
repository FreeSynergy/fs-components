"""Generated Markdown of the contract (BLOCKS, BLOCKS.de, MAPPING). Pattern: Template method (one table writer, texts per language)."""
from __future__ import annotations

import os
from collections import Counter, defaultdict

TEXT = {
    "en": {
        "title": "Theme contract: blocks", "intro": "Generated from `contract/blocks/*.toml` by `contract/render`. Do not edit.",
        "scope": {"general": "General blocks", "block": "Block-specific blocks (one widget or crate)",
                  "program": "Program-specific blocks (rich blocks of one program)"},
        "cols": ("Block", "Name from", "Parts", "States", "Variants", "Summary"), "own": "own name", "none": "none",
        "version": "Contract version",
    },
    "de": {
        "title": "Theme-Vertrag: Bausteine", "intro": "Erzeugt aus `contract/blocks/*.toml` mit `contract/render`. Nicht von Hand ändern.",
        "scope": {"general": "Allgemeine Bausteine", "block": "Baustein-spezifische Bausteine (ein Widget oder Crate)",
                  "program": "Programm-spezifische Bausteine (reiche Bausteine eines Programms)"},
        "cols": ("Baustein", "Name nach", "Teile", "Zustände", "Varianten", "Kurzbeschreibung"), "own": "eigener Name", "none": "keine",
        "version": "Vertragsversion",
    },
}

OUTPUTS = (("BLOCKS.md", "en"), ("BLOCKS.de.md", "de"))


def _names(items, none):
    return ", ".join(f"`{i}`" for i in items) or none


def blocks_md(contract, lang):
    t = TEXT[lang]
    out = [f"# {t['title']}", "", t["intro"], "", f"{t['version']}: **{contract.meta['contract']['version']}**", ""]
    for scope in ("general", "block", "program"):
        rows = [b for b in contract.blocks.values() if b.scope == scope]
        out += [f"## {t['scope'][scope]} ({len(rows)})", "", "| " + " | ".join(t["cols"]) + " |", "|" + "---|" * len(t["cols"])]
        for b in sorted(rows, key=lambda b: b.name):
            src = f"{b.naming} `{b.ref}`" if b.naming != "own" else t["own"]
            text = (b.summary_de if lang == "de" else b.summary).replace("|", "/")
            out.append(f"| `{b.name}` | {src} | {_names(b.parts, t['none'])} | {_names(b.states, t['none'])} | "
                       f"{_names(b.variants, t['none'])} | {text} |")
        out.append("")
    return "\n".join(out)


def _block(entry):
    return entry.target.split(".")[0].split(":")[0].split("/")[0]


def _scope(contract, entry):
    block = contract.blocks.get(_block(entry))
    return block.scope if block else "none"


def mapping_md(contract):
    """Old class or id name -> contract name, grouped by origin and status. Names are language neutral."""
    entries = contract.entries
    count = Counter((e.origin.split(":")[0], e.status) for e in entries)
    out = ["# Theme contract: mapping of today's names", "",
           "Generated from `contract/mapping/*.toml` by `contract/render`. Do not edit. Status: `mapped` = has its place in the "
           "contract, only the name changes; `provisional` = the place or the program is still provisional or the target is a best guess; `remove` = no "
           "contract name (arrangement or style detail that tokens or the frontend take over).", "",
           "| Origin | mapped | provisional | remove |", "|---|---|---|---|"]
    for origin in ("here", "coming", "external"):
        out.append(f"| {origin} | " + " | ".join(str(count[(origin, s)]) for s in ("mapped", "provisional", "remove")) + " |")
    out.append(f"\nTest-only names (outside the contract): {len(contract.test_only)}. Exceptions: "
               f"{sum(len(v) for v in contract.exceptions.values())}.\n")
    groups = defaultdict(list)
    for e in entries:
        groups[e.origin].append(e)
    for origin in sorted(groups, key=lambda o: (o != "here", o)):
        out += [f"## {origin}", "", "| Today | Contract name | Scope | Status |", "|---|---|---|---|"]
        for e in sorted(groups[origin], key=lambda e: (_scope(contract, e), _block(e), e.target, e.name)):
            out.append(f"| `{e.name}` | `{e.target}` | {_scope(contract, e)} | {e.status} |")
        out.append("")
    out += ["## Exceptions", ""]
    for group, names in contract.exceptions.items():
        out += [f"### {group}", "", "| Name | Reason |", "|---|---|"] + [f"| `{n}` | {r} |" for n, r in sorted(names.items())] + [""]
    out += ["## Test-only", "", ", ".join(f"`{n}`" for n in sorted(contract.test_only)), ""]
    return "\n".join(out)


def documents(contract, usage=None):
    docs = {name: blocks_md(contract, lang) for name, lang in OUTPUTS}
    docs["MAPPING.md"] = mapping_md(contract)
    return docs


def write(contract, usage=None):
    for name, text in documents(contract, usage).items():
        with open(os.path.join(contract.root, "contract", name), "w", encoding="utf-8") as handle:
            handle.write(text.rstrip("\n") + "\n")


def stale(contract, usage=None):
    out = []
    for name, text in documents(contract, usage).items():
        path = os.path.join(contract.root, "contract", name)
        have = open(path, encoding="utf-8").read() if os.path.exists(path) else None
        if have != text.rstrip("\n") + "\n":
            out.append(f"contract/{name} is out of date: run `contract/render`")
    return out
