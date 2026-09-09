#!/usr/bin/env python3
"""Validate F02.1's offline immutable graphics-input inventory."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import sys
import tempfile
import unittest
from pathlib import Path, PurePosixPath
from urllib.parse import urlsplit

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from inventory_layout import FAMILIES, INPUT_FIELDS as FIELDS, InventoryLayoutError, load_inventory, render_v2_lock

MANIFEST = HERE / "manifest.toml"
COMMIT = re.compile(r"^[0-9a-f]{40}$")
DIGEST = re.compile(r"^[0-9a-f]{64}$")
IDENTIFIER = re.compile(r"^[a-z0-9][a-z0-9-]*$")
MUTABLE = frozenset(("head", "latest", "main", "master", "stable", "trunk"))


def fail(message: str) -> None:
    raise ValueError(message)


def string(value: object, field: str) -> str:
    if not isinstance(value, str) or not value:
        fail(f"{field} must be a nonempty string")
    return value


def validate(entries: object) -> int:
    if not isinstance(entries, (list, tuple)) or not entries:
        fail("inputs must be a nonempty array")
    ids, families = set(), set()
    for index, entry in enumerate(entries):
        if not isinstance(entry, dict) or set(entry) != FIELDS:
            fail(f"input {index} does not match the maintained schema")
        identifier = string(entry["id"], "id")
        family = string(entry["source_family"], "source_family")
        revision = string(entry["revision"], "revision")
        digest = string(entry["sha256"], "sha256")
        if not IDENTIFIER.fullmatch(identifier) or identifier in ids:
            fail(f"input {index} has a duplicate or invalid id")
        if not COMMIT.fullmatch(revision):
            fail(f"input {identifier} has a non-commit revision")
        if not DIGEST.fullmatch(digest) or digest == "0" * 64:
            fail(f"input {identifier} has an invalid sha256")
        if not isinstance(entry["bytes"], int) or entry["bytes"] <= 0:
            fail(f"input {identifier} has invalid byte count")
        for field in ("license", "generated_code_role"):
            string(entry[field], field)
        provenance = urlsplit(string(entry["provenance"], "provenance"))
        if provenance.scheme != "https" or not provenance.netloc:
            fail(f"input {identifier} has invalid provenance URL")
        url = urlsplit(string(entry["immutable_url"], "immutable_url"))
        parts = tuple(part.lower() for part in url.path.split("/") if part)
        if url.scheme != "https" or not url.netloc or revision not in url.path:
            fail(f"input {identifier} has no immutable HTTPS source URL")
        if MUTABLE.intersection(parts):
            fail(f"input {identifier} has a mutable URL reference")
        cache = PurePosixPath(string(entry["local_cache"], "local_cache"))
        if cache.is_absolute() or ".." in cache.parts or cache.parts[:2] != ("webboxvm-graphics", "f02"):
            fail(f"input {identifier} has a repository-relative or unsafe cache")
        if identifier not in cache.parts or not str(cache).endswith(f"/{digest}.source"):
            fail(f"input {identifier} cache does not bind its byte identity")
        ids.add(identifier)
        families.add(family)
    if families != FAMILIES or len(ids) != len(entries):
        fail("source-family coverage is incomplete or duplicated")
    return len(entries)


def load(path: Path = MANIFEST) -> tuple[tuple[dict[str, object], ...], str]:
    try:
        inventory = load_inventory(path)
    except InventoryLayoutError as error:
        fail(str(error))
    return inventory.inputs, inventory.revision


class InventoryTests(unittest.TestCase):
    def setUp(self) -> None:
        self.entries, self.revision = load()

    def reject(self, change, message: str) -> None:
        entries = list(copy.deepcopy(self.entries))
        change(entries)
        with self.assertRaisesRegex(ValueError, message):
            validate(entries)

    def test_reviewed_inventory_is_valid(self) -> None:
        self.assertEqual(validate(self.entries), len(FAMILIES))
        self.assertRegex(self.revision, r"^[0-9a-f]{64}$")

    def test_missing_family_is_rejected(self) -> None:
        self.reject(lambda entries: entries.pop(), "source-family coverage")

    def test_bad_digest_is_rejected(self) -> None:
        self.reject(lambda entries: entries[0].update(sha256="0" * 64), "invalid sha256")

    def test_mutable_source_reference_is_rejected(self) -> None:
        def make_mutable(entries):
            entry = entries[0]
            entry["immutable_url"] = entry["immutable_url"].replace(entry["revision"], "main")
        self.reject(make_mutable, "immutable HTTPS source URL")

    def test_wgsl_grammar_is_separate_from_the_semantic_reference(self) -> None:
        entries = {entry["id"]: entry for entry in self.entries}
        reference, grammar = entries["wgsl-spec"], entries["wgsl-grammar-syntax"]
        self.assertEqual(reference["source_family"], "wgsl")
        self.assertEqual(reference["generated_code_role"], "WGSL emitter semantic reference; no generated code")
        self.assertEqual(grammar["source_family"], "wgsl-grammar")
        self.assertTrue(grammar["immutable_url"].endswith("/wgsl/syntax.bnf"))
        self.assertEqual(grammar["license"], "W3C Software and Document License (repo LICENSE.md; document)")
        self.assertEqual(grammar["generated_code_role"], "future WGSL grammar-input/parser-validation generator input; nonstandard BNF dialect")

    def v2_inventory(self) -> tuple[tempfile.TemporaryDirectory, Path]:
        temporary = tempfile.TemporaryDirectory(dir=HERE)
        manifest = Path(temporary.name) / "manifest.toml"
        part = manifest.parent / "inputs" / "part-0001.toml"
        part.parent.mkdir()
        manifest.write_text(
            "schema = 2\ncache_root = \"$XDG_CACHE_HOME\"\ncache_note = \"fixture\"\n"
            f"required_families = {json.dumps(sorted(FAMILIES))}\ninput_files = [\"inputs/part-0001.toml\"]\n",
            encoding="utf-8")
        lines: list[str] = []
        for entry in self.entries:
            lines.append("[[inputs]]")
            for field in sorted(FIELDS):
                value = entry[field] if field == "bytes" else json.dumps(entry[field])
                lines.append(f"{field} = {value}")
        part.write_text("\n".join(lines) + "\n", encoding="utf-8")
        manifest.with_name("inventory.lock").write_bytes(render_v2_lock(manifest))
        return temporary, manifest

    def test_v2_closure_loads_and_rejects_stale_bytes(self) -> None:
        temporary, manifest = self.v2_inventory()
        self.addCleanup(temporary.cleanup)
        entries, revision = load(manifest)
        self.assertEqual(validate(entries), len(FAMILIES))
        self.assertEqual(revision, hashlib.sha256(manifest.with_name("inventory.lock").read_bytes()).hexdigest())
        for name in ("manifest.toml", "inputs/part-0001.toml", "inventory.lock"):
            with self.subTest(name=name):
                target = manifest.parent / name
                original = target.read_bytes()
                target.write_bytes(original + b"# stale\n")
                with self.assertRaisesRegex(ValueError, "inventory.lock"):
                    load(manifest)
                target.write_bytes(original)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true", help="run hermetic structural tests")
    if parser.parse_args().self_test:
        unittest.main(argv=[sys.argv[0]])
        return
    entries, revision = load()
    print(f"PASS: {validate(entries)} immutable inputs; inventory sha256={revision}")


if __name__ == "__main__":
    main()
