"""Numbers for the report. Pattern: none (pure functions)."""
from __future__ import annotations

from collections import Counter


def classify(ctx, name):
    for e in ctx.contract.here():
        if e.name == name:
            return f"{e.status}"
    if name in ctx.contract.test_only:
        return "test-only"
    if name in ctx.not_class:
        return "exception (not a class)"
    if name in ctx.unmapped:
        return "exception (unmapped)"
    return "MISSING"


def report(ctx):
    c = ctx.contract
    lines = [f"contract {c.meta['contract']['version']}: {len(c.blocks)} blocks, "
             f"{sum(len(b.parts) for b in c.blocks.values())} parts, {sum(len(b.states) for b in c.blocks.values())} states, "
             f"{sum(len(b.variants) for b in c.blocks.values())} variants"]
    by_scope = Counter(b.scope for b in c.blocks.values())
    lines.append("blocks by scope: " + ", ".join(f"{k} {v}" for k, v in sorted(by_scope.items())))
    for title, names in (("literal .class/.id names (src + tests)", ctx.usage.literal),
                         ("all names of the source (literals, CSS selectors, constants) and literals of tests", set(ctx.usage.src) | ctx.usage.literal)):
        count = Counter(classify(ctx, n) for n in names)
        lines.append(f"{title}: {len(names)} = " + ", ".join(f"{k} {v}" for k, v in sorted(count.items())))
    targets = {e.target.split(":")[0].split("/")[0] for e in c.entries if e.target != "-"}
    used = {t for t in targets if "." in t}
    unused = [f"{b.name}.{p}" for b in c.blocks.values() for p in b.parts if f"{b.name}.{p}" not in used]
    lines.append(f"parts that today's names reach: {sum(len(b.parts) for b in c.blocks.values()) - len(unused)} of "
                 f"{sum(len(b.parts) for b in c.blocks.values())}; blocks without any mapped name: "
                 f"{', '.join(sorted(b.name for b in c.blocks.values() if not any(t.split('.')[0] == b.name for t in targets))) or 'none'}")
    origin = Counter(e.origin.split(":")[0] for e in c.entries)
    lines.append("mapping entries: " + ", ".join(f"{k} {v}" for k, v in sorted(origin.items())))
    return "\n".join(lines)
