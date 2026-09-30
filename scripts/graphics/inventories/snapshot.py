"""Reuse source proofs inside one batch only while every admitted input stays fixed."""

from __future__ import annotations

import copy
import hashlib
from pathlib import Path

from .source import reject


class SourceSnapshot:
    def __init__(self, repo: Path, cache_root: Path):
        self.roots = (repo / "todo/graphics/00-foundation/01-contract",
                      repo / "scripts/graphics/inventories", cache_root)
        self.before = self.fingerprint()
        self.proof = None

    def fingerprint(self):
        entries = []
        for index, root in enumerate(self.roots):
            if root.is_symlink():
                reject("source proof root must not be a symlink")
            for path in sorted(root.rglob("*")):
                if "__pycache__" in path.parts:
                    continue
                if path.is_symlink():
                    reject("source proof input must not be a symlink")
                if path.is_file():
                    entries.append((index, str(path.relative_to(root)), hashlib.sha256(path.read_bytes()).hexdigest()))
        return tuple(entries)

    def remember(self, source, raw, domain, grammar, ledger):
        self.proof = copy.deepcopy((source, raw, domain, grammar, ledger))

    def reuse(self, engine, source, raw):
        expected, prior_raw, domain, grammar, ledger = self.proof
        if not engine.exact(expected, source) or prior_raw != raw:
            reject("batch source proof does not match current admitted bytes")
        return copy.deepcopy((domain, grammar, ledger))

    def finish(self):
        if self.before != self.fingerprint():
            reject("source inputs changed during the batch; no result can be published")
