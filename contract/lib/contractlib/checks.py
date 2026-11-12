"""The checks of the contract. Pattern: Strategy + Template (every Check has `run`; `run_all` collects the problems)."""
from __future__ import annotations

import re
from abc import ABC, abstractmethod

from . import render, vocab
from .scan import scan

NAME = re.compile(vocab.NAME_PATTERN)
TARGET = re.compile(rf"^(?P<block>{vocab.NAME_PATTERN})(?:\.(?P<part>{vocab.NAME_PATTERN}))?"
                    rf"(?:/(?P<variant>{vocab.NAME_PATTERN}))?(?::(?P<state>{vocab.NAME_PATTERN}))?$")


class Context:
    def __init__(self, contract, usage):
        self.contract, self.usage = contract, usage
        self.exceptions = contract.exceptions
        self.not_class = set(self.exceptions.get("not-a-class", {}))
        self.unmapped = set(self.exceptions.get("unmapped", {}))
        self.here = {e.name for e in contract.here()}


class Check(ABC):
    """Interface: `title` names the check, `run` returns problem lines (empty list = the check holds)."""
    title = ""

    @abstractmethod
    def run(self, ctx: Context) -> list:
        ...


class UsedNamesMapped(Check):
    title = "every class or id used in the source is in the mapping or on the exception list"

    def run(self, ctx):
        known = ctx.here | ctx.not_class | ctx.unmapped
        return [f"`{n}` is used ({sites[0]}, {len(sites)}x) but not mapped" for n, sites in sorted(ctx.usage.src.items())
                if n not in known]


class MappingNotStale(Check):
    title = "every mapped name, exception and test-only name still exists in the source"

    def run(self, ctx):
        out = [f"mapping lists `{n}` but no source uses it" for n in sorted(ctx.here - set(ctx.usage.src))]
        for group in (ctx.not_class, ctx.unmapped):
            out += [f"exception `{n}` is not used in the source any more" for n in sorted(group - set(ctx.usage.src))]
        out += [f"test-only `{n}` is not used in any test" for n in sorted(set(ctx.contract.test_only) - set(ctx.usage.tests))]
        out += [f"test-only `{n}` is used in production code" for n in sorted(set(ctx.contract.test_only) & set(ctx.usage.src))]
        return out


class BlocksDocumented(Check):
    title = "every block has parts, states, a summary (EN and DE), a valid scope and a justified name"

    def run(self, ctx):
        out = []
        for b in ctx.contract.blocks.values():
            if not b.parts:
                out.append(f"block `{b.name}` has no parts")
            if not b.states:
                out.append(f"block `{b.name}` has no states")
            if not b.summary or not b.summary_de:
                out.append(f"block `{b.name}` lacks summary or summary_de")
            if b.scope not in vocab.SCOPES:
                out.append(f"block `{b.name}` has scope `{b.scope}` (allowed: {', '.join(vocab.SCOPES)})")
            out += self._naming(b)
        return out

    @staticmethod
    def _naming(b):
        if b.naming not in vocab.NAMING:
            return [f"block `{b.name}` has naming `{b.naming}` (allowed: {', '.join(vocab.NAMING)})"]
        if b.naming == "own":
            return [] if b.reason else [f"block `{b.name}` has an own name and no reason"]
        known = vocab.ARIA_ROLES if b.naming == "aria" else vocab.OPEN_UI_NAMES
        if b.ref not in known:
            return [f"block `{b.name}`: `{b.ref}` is not a {b.naming} name"]
        if b.name != b.ref:
            return [f"block `{b.name}` is named after {b.naming} `{b.ref}`: the names must be equal"]
        return []


class NamesWellFormed(Check):
    title = "contract names (blocks, parts, states, variants) are only [a-z0-9-]; mapping targets point at real parts"

    def run(self, ctx):
        out = []
        for b in ctx.contract.blocks.values():
            for kind, names in (("block", (b.name,)), ("part", b.parts), ("state", b.states), ("variant", b.variants)):
                out += [f"{kind} name `{n}` of block `{b.name}` is not [a-z0-9-]" for n in names if not NAME.fullmatch(n)]
        for e in ctx.contract.entries:
            out += self._target(ctx.contract.blocks, e)
        return out

    @staticmethod
    def _target(blocks, e):
        if e.target == "-":
            return [] if e.status == "remove" else [f"`{e.name}`: target `-` only with status remove"]
        m = TARGET.match(e.target)
        if not m:
            return [f"`{e.name}` -> `{e.target}` is not block[.part][/variant][:state]"]
        b = blocks.get(m["block"])
        if b is None:
            return [f"`{e.name}` -> unknown block `{m['block']}`"]
        out = []
        for key, allowed in (("part", b.parts), ("state", b.states), ("variant", b.variants)):
            if m[key] and m[key] not in allowed:
                out.append(f"`{e.name}` -> `{e.target}`: `{m[key]}` is not a {key} of `{b.name}`")
        return out


class NoDuplicates(Check):
    title = "no duplicates: blocks, parts, states, variants, mapped names, exceptions"

    def run(self, ctx):
        out = list(ctx.contract.problems)
        for b in ctx.contract.blocks.values():
            for kind, names in (("part", b.parts), ("state", b.states), ("variant", b.variants)):
                out += [f"block `{b.name}` lists {kind} `{n}` twice" for n in sorted({x for x in names if names.count(x) > 1})]
        seen = {}
        for e in ctx.contract.entries:
            key = (e.name, e.origin)
            if key in seen:
                out.append(f"`{e.name}` is mapped twice for {e.origin} ({seen[key]} and {e.file}:{e.status})")
            seen[key] = f"{e.file}:{e.status}"
        tests = ctx.contract.test_only
        out += [f"test-only `{n}` is listed twice" for n in sorted({x for x in tests if tests.count(x) > 1})]
        both = ctx.here & (ctx.not_class | ctx.unmapped | set(tests))
        out += [f"`{n}` is mapped and also an exception or test-only name" for n in sorted(both)]
        out += [f"`{n}` is on two exception lists" for n in sorted(ctx.not_class & ctx.unmapped)]
        return out


class DocsCurrent(Check):
    title = "the generated documents (BLOCKS.md, BLOCKS.de.md, MAPPING.md) match the TOML files"

    def run(self, ctx):
        return render.stale(ctx.contract, ctx.usage)


CHECKS = (UsedNamesMapped(), MappingNotStale(), BlocksDocumented(), NamesWellFormed(), NoDuplicates(), DocsCurrent())


def run_all(contract, checks=CHECKS):
    prefixes = contract.meta["contract"]["prefixes"]
    ctx = Context(contract, scan(contract.root, prefixes))
    return [(c.title, c.run(ctx)) for c in checks], ctx
