"""Loading the contract from its TOML files. Pattern: Builder (files in, one immutable model out)."""
from __future__ import annotations

import glob
import os
import tomllib
from dataclasses import dataclass, field


@dataclass(frozen=True)
class Block:
    name: str
    file: str
    naming: str
    ref: str
    reason: str
    scope: str
    widget: str
    summary: str
    summary_de: str
    parts: tuple
    states: tuple
    variants: tuple


@dataclass(frozen=True)
class Entry:
    """One old name and where it goes. `origin` is `here`, `coming:<branch>` or `external:<repo>`."""
    name: str
    target: str
    status: str
    origin: str
    file: str


@dataclass
class Contract:
    root: str
    meta: dict
    blocks: dict = field(default_factory=dict)
    entries: list = field(default_factory=list)
    test_only: list = field(default_factory=list)
    exceptions: dict = field(default_factory=dict)
    problems: list = field(default_factory=list)

    def here(self):
        return [e for e in self.entries if e.origin == "here"]


def _read(path):
    with open(path, "rb") as handle:
        return tomllib.load(handle)


def _files(root, pattern):
    return sorted(glob.glob(os.path.join(root, "contract", pattern)))


def load(root):
    """Read `contract/contract.toml` and every file it lists. Problems of the files go to `problems`, never raise."""
    meta = _read(os.path.join(root, "contract", "contract.toml"))
    contract = Contract(root=root, meta=meta)
    for pattern in meta["contract"].get("block_files", []):
        for path in _files(root, pattern):
            _blocks(contract, path)
    for pattern in meta["contract"].get("mapping_files", []):
        for path in _files(root, pattern):
            _mapping(contract, path)
    for path in _files(root, meta["contract"].get("exception_file", "exceptions.toml")):
        contract.exceptions = _read(path)
    return contract


def _blocks(contract, path):
    rel = os.path.relpath(path, contract.root)
    for name, body in _read(path).get("block", {}).items():
        if name in contract.blocks:
            contract.problems.append(f"{rel}: block `{name}` is already defined in {contract.blocks[name].file}")
            continue
        contract.blocks[name] = Block(
            name=name, file=rel, naming=body.get("naming", ""), ref=body.get("ref", ""),
            reason=body.get("reason", ""), scope=body.get("scope", ""), widget=body.get("widget", ""),
            summary=body.get("summary", ""), summary_de=body.get("summary_de", ""),
            parts=tuple(body.get("parts", [])), states=tuple(body.get("states", [])),
            variants=tuple(body.get("variants", [])))


def _mapping(contract, path):
    rel = os.path.relpath(path, contract.root)
    data = _read(path)
    for status, names in data.get("map", {}).items():
        _add(contract, names, status, "here", rel)
    for branch, groups in data.get("coming", {}).items():
        for status, names in groups.items():
            _add(contract, names, status, f"coming:{branch}", rel)
    for repo, groups in data.get("external", {}).items():
        for status, names in groups.items():
            _add(contract, names, status, f"external:{repo}", rel)
    contract.test_only.extend(data.get("test", {}).get("names", []))


def _add(contract, names, status, origin, rel):
    for name, target in names.items():
        contract.entries.append(Entry(name, target, status, origin, rel))
