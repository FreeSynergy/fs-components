"""Counter-tests: break a copy of the repository in one way and expect the matching check to fail.
Pattern: Command (each Mutation changes the copy and says which message must appear)."""
from __future__ import annotations

import os
import re
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from typing import Callable


@dataclass(frozen=True)
class Mutation:
    name: str
    apply: Callable[[str], None]
    expect: str  # a substring of the output that proves the right check noticed it


def edit(path, old, new, count=1):
    text = open(path, encoding="utf-8").read()
    if old not in text:
        raise SystemExit(f"selftest broken: `{old}` not found in {path}")
    open(path, "w", encoding="utf-8").write(text.replace(old, new, count))


def append(path, text):
    with open(path, "a", encoding="utf-8") as handle:
        handle.write(text)


def c(root, *parts):
    return os.path.join(root, "contract", *parts)


def src_file(root):
    return os.path.join(root, "crates", "fs-components-widgets", "src", "badge.rs")


def regenerate(root):
    subprocess.run([sys.executable, "-I", c(root, "render"), root], check=True)


def doc_mutation(fn):
    """Mutations of the TOML files must keep the generated documents current, or the wrong check would fire."""
    def run(root):
        fn(root)
        regenerate(root)
    return run


MUTATIONS = (
    Mutation("new class in a .class call", lambda r: append(src_file(r), '\nfn _m() { let _ = div().class("fs-mutant-a"); }\n'), "`fs-mutant-a` is used"),
    Mutation("new id without prefix", lambda r: append(src_file(r), '\nfn _m() { let _ = div().id("mutant-b"); }\n'), "`mutant-b` is used"),
    Mutation("new name only in a CSS string", lambda r: append(src_file(r), '\nconst _S: &str = ".fs-mutant-c { color: red; }";\n'), "`fs-mutant-c` is used"),
    Mutation("name with several classes in one call", lambda r: append(src_file(r), '\nfn _m() { let _ = div().class("fs-badge fs-mutant-d"); }\n'), "`fs-mutant-d` is used"),
    Mutation("a mapping line removed", doc_mutation(lambda r: edit(c(r, "mapping", "widgets.toml"), '"fs-badge-dot" = "badge.dot"\n', "")), "`fs-badge-dot` is used"),
    Mutation("stale mapping entry", doc_mutation(lambda r: edit(c(r, "mapping", "widgets.toml"), "[map.mapped]\n", '[map.mapped]\n"fs-gone-x" = "badge.dot"\n')), "no source uses it"),
    Mutation("stale exception", lambda r: edit(c(r, "exceptions.toml"), "[unmapped]\n", '[unmapped]\n"fs-gone-y" = "x"\n'), "not used in the source any more"),
    Mutation("test-only name used in production", doc_mutation(lambda r: edit(c(r, "mapping", "test-only.toml"), "names = [", 'names = [\n  "fs-badge",')), "is used in production code"),
    Mutation("block without parts", doc_mutation(lambda r: edit(c(r, "blocks", "general.toml"), 'parts = ["icon", "label"]', "parts = []")), "has no parts"),
    Mutation("block without states", doc_mutation(lambda r: edit(c(r, "blocks", "general.toml"), 'states = ["hover", "focus", "cursor", "pressed", "disabled", "busy"]', "states = []")), "has no states"),
    Mutation("block without German summary", doc_mutation(lambda r: edit(c(r, "blocks", "general.toml"), 'summary_de = "Ein drückbares', 'summary_x = "Ein drückbares')), "lacks summary or summary_de"),
    Mutation("part name with capitals and underscore", doc_mutation(lambda r: edit(c(r, "blocks", "general.toml"), 'parts = ["icon", "label"]', 'parts = ["icon", "Label_X"]')), "`Label_X` of block `button` is not [a-z0-9-]"),
    Mutation("duplicate part", doc_mutation(lambda r: edit(c(r, "blocks", "general.toml"), 'parts = ["icon", "label"]', 'parts = ["icon", "label", "icon"]')), "lists part `icon` twice"),
    Mutation("duplicate state", doc_mutation(lambda r: edit(c(r, "blocks", "general.toml"), 'states = ["hover", "focus", "cursor"', 'states = ["hover", "hover", "focus", "cursor"')), "lists state `hover` twice"),
    Mutation("duplicate block in another file", doc_mutation(lambda r: append(c(r, "blocks", "frames.toml"), '\n[block.button]\nnaming = "own"\nreason = "x"\nscope = "general"\nsummary = "x"\nsummary_de = "x"\nparts = ["a"]\nstates = ["b"]\n')), "block `button` is already defined"),
    Mutation("name mapped in two files", doc_mutation(lambda r: append(c(r, "mapping", "zz-extra.toml"), '[map.remove]\n"fs-badge" = "badge.root"\n')), "is mapped twice"),
    Mutation("mapping to a part that does not exist", doc_mutation(lambda r: edit(c(r, "mapping", "widgets.toml"), '"fs-badge-dot" = "badge.dot"', '"fs-badge-dot" = "badge.nothing"')), "`nothing` is not a part of `badge`"),
    Mutation("mapping to an unknown block", doc_mutation(lambda r: edit(c(r, "mapping", "widgets.toml"), '"fs-badge-dot" = "badge.dot"', '"fs-badge-dot" = "badgee.dot"')), "unknown block `badgee`"),
    Mutation("own name without a reason", doc_mutation(lambda r: edit(c(r, "blocks", "frames.toml"), 'reason = "The ARIA role is complementary', 'reasonx = "The ARIA role is complementary')), "has an own name and no reason"),
    Mutation("aria name that is no role", doc_mutation(lambda r: edit(c(r, "blocks", "general.toml"), 'ref = "textbox"', 'ref = "textfield"')), "is not a aria name"),
    Mutation("block name differs from its role", doc_mutation(lambda r: edit(c(r, "blocks", "general.toml"), "[block.textbox]", "[block.text-box]")), "the names must be equal"),
    Mutation("mapped name also on an exception list", doc_mutation(lambda r: edit(c(r, "exceptions.toml"), "[unmapped]\n", '[unmapped]\n"fs-badge" = "x"\n')), "is mapped and also an exception"),
    Mutation("mapping to a state that does not exist", doc_mutation(lambda r: edit(c(r, "mapping", "widgets.toml"), '"fs-badge-dot" = "badge.dot"', '"fs-badge-dot" = "badge.dot:sleeping"')), "`sleeping` is not a state of `badge`"),
    Mutation("mapping to a variant that does not exist", doc_mutation(lambda r: edit(c(r, "mapping", "widgets.toml"), '"fs-badge-dot" = "badge.dot"', '"fs-badge-dot" = "badge.dot/huge"')), "`huge` is not a variant of `badge`"),
    Mutation("generated document edited by hand", lambda r: append(c(r, "BLOCKS.md"), "\nhand edit\n"), "contract/BLOCKS.md is out of date"),
)


def copy_repo(root, dest):
    ignore = shutil.ignore_patterns("target", ".git", "temp", "node_modules")
    shutil.copytree(os.path.join(root, "crates"), os.path.join(dest, "crates"), ignore=ignore)
    shutil.copytree(os.path.join(root, "contract"), os.path.join(dest, "contract"), ignore=shutil.ignore_patterns("__pycache__"))


def run_check(root):
    done = subprocess.run([sys.executable, "-I", c(root, "check"), root], capture_output=True, text=True)
    return done.returncode, done.stdout + done.stderr


def selftest(root):
    """Return the list of failures of the selftest itself (empty = every check proved that it can fail)."""
    failures = []
    with tempfile.TemporaryDirectory(prefix="contract-selftest-") as tmp:
        base = os.path.join(tmp, "base")
        copy_repo(root, base)
        code, out = run_check(base)
        if code != 0:
            return [f"the unmodified copy does not pass:\n{out}"]
        for mutation in MUTATIONS:
            work = os.path.join(tmp, re.sub(r"\W+", "-", mutation.name))
            shutil.copytree(base, work)
            mutation.apply(work)
            code, out = run_check(work)
            if code == 0:
                failures.append(f"{mutation.name}: the check still passes")
            elif mutation.expect not in out:
                failures.append(f"{mutation.name}: failed, but not with `{mutation.expect}`:\n{out[:600]}")
            else:
                print(f"ok   {mutation.name}")
        failures += hardcoded_selftest(base, tmp)
    return failures


def hardcoded_selftest(base, tmp):
    """`contract/hardcoded` must notice a new fixed value (check fails until regenerated, then the table has it)."""
    out = []
    script = c(base, "hardcoded")
    if subprocess.run([sys.executable, "-I", script, base, "--check"], capture_output=True).returncode != 0:
        return ["HARDCODED-VALUES.md is out of date in the unmodified copy"]
    work = os.path.join(tmp, "hardcoded")
    shutil.copytree(base, work)
    append(os.path.join(work, "crates", "fs-components-widgets", "src", "zz_mutant.rs"), '\nconst _S: &str = ".fs-badge { padding: 7px; border-radius: 5px; color: #123456; }";\n')
    if subprocess.run([sys.executable, "-I", c(work, "hardcoded"), work, "--check"], capture_output=True).returncode == 0:
        out.append("hardcoded: a new fixed value was not noticed")
    subprocess.run([sys.executable, "-I", c(work, "hardcoded"), work], capture_output=True, check=True)
    table = open(c(work, "HARDCODED-VALUES.md"), encoding="utf-8").read()
    for needle in ("#123456", "`7px`", "`5px`"):
        if needle not in table:
            out.append(f"hardcoded: {needle} is not in the regenerated table")
    if not out:
        print("ok   hardcoded: a new colour, padding and radius appear in HARDCODED-VALUES.md")
    return out
