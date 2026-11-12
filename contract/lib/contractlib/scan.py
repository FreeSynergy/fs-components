"""Finding the class and id names the code uses. Pattern: Strategy (each Finder looks for names one way)."""
from __future__ import annotations

import glob
import os
import re
from abc import ABC, abstractmethod
from collections import defaultdict

LITERAL = re.compile(r'\.(?:class|id)\(\s*"([^"]+)"')
CSS_SELECTOR = re.compile(r'(?:^|[\s,>{}+])\.([a-z][a-z0-9-]*)(?=[\s,{:.>\[#)]|$)')


def is_test_file(rel):
    base = os.path.basename(rel)
    return "/tests/" in "/" + rel or base == "tests.rs" or base.endswith("_tests.rs")


class Finder(ABC):
    """Interface: one line in, the names found in it out (a name followed by `{` is a format hole and is skipped)."""

    @abstractmethod
    def find(self, line: str, rel: str):
        ...


class LiteralFinder(Finder):
    """`.class("a b")` and `.id("a")`: every space separated word is a name."""

    def find(self, line, rel):
        for match in LITERAL.finditer(line):
            yield from match.group(1).split()


class PrefixFinder(Finder):
    """Names with one of the family prefixes (`fs-x`, `craft-x`, `cn-x`) anywhere in a code line: CSS selectors,
    constants, match arms. Comment lines are skipped."""

    def __init__(self, prefixes):
        alt = "|".join(re.escape(p.rstrip("-")) for p in prefixes)
        self.pattern = re.compile(rf"(?<![\w-])((?:{alt})-[a-z0-9][a-z0-9-]*[a-z0-9])(?![\w-]|\{{)")

    def find(self, line, rel):
        if line.lstrip().startswith("//"):
            return
        for match in self.pattern.finditer(line):
            yield match.group(1)


class CssFinder(Finder):
    """Selectors in `.css` files."""

    def find(self, line, rel):
        if rel.endswith(".css"):
            for match in CSS_SELECTOR.finditer(line):
                yield match.group(1)


class Usage:
    """Where each name is used. `src` = production code, `tests` = test files."""

    def __init__(self):
        self.src = defaultdict(list)
        self.tests = defaultdict(list)
        self.literal = set()

    def names(self):
        return set(self.src) | set(self.tests)


def scan(root, prefixes, globs=("crates/*/src/**/*.rs", "crates/*/src/**/*.css", "crates/*/tests/**/*.rs")):
    finders = [LiteralFinder(), PrefixFinder(prefixes), CssFinder()]
    usage = Usage()
    seen = set()
    for pattern in globs:
        for path in sorted(glob.glob(os.path.join(root, pattern), recursive=True)):
            if path in seen:
                continue
            seen.add(path)
            rel = os.path.relpath(path, root)
            bucket = usage.tests if is_test_file(rel) else usage.src
            with open(path, encoding="utf-8", errors="replace") as handle:
                for number, line in enumerate(handle, 1):
                    for finder in finders:
                        for name in finder.find(line, rel):
                            bucket[name].append(f"{rel}:{number}")
                            if isinstance(finder, LiteralFinder):
                                usage.literal.add(name)
    return usage
