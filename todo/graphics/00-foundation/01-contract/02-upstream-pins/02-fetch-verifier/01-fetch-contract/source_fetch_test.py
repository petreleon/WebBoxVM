#!/usr/bin/env python3
"""Hermetic F02.2.1 contract tests; F02.2.2 owns network fixtures."""

from __future__ import annotations

import hashlib
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "01-input-inventory"))

from source_cache import DenyRedirect, atomic_store, verify_payload
from source_model import ContractError, ExternalCache, MAX_INPUT_BYTES, REQUIRED_FAMILIES, SourceInput, load_manifest
from inventory_layout import render_v2_lock

MANIFEST = HERE.parents[1] / "01-input-inventory" / "manifest.toml"
PAYLOAD = b"F02.2.1 verified fixture\n"
REVISION = "a" * 40


def entry(**changes) -> dict[str, object]:
    digest = hashlib.sha256(PAYLOAD).hexdigest()
    value = {
        "id": "fixture", "source_family": "fixture",
        "immutable_url": f"https://raw.githubusercontent.com/example/fixture/{REVISION}/payload.bin",
        "revision": REVISION, "sha256": digest, "bytes": len(PAYLOAD),
        "license": "fixture", "local_cache": f"webboxvm-graphics/f02/fixture/{digest}.source",
        "generated_code_role": "fixture", "provenance": "https://example.invalid/provenance",
    }
    value.update(changes)
    return value


def v2_inventory(base: Path) -> Path:
    manifest, part = base / "manifest.toml", base / "inputs" / "part-0001.toml"
    part.parent.mkdir(parents=True)
    families = sorted(REQUIRED_FAMILIES)
    manifest.write_text(
        "schema = 2\ncache_root = \"$XDG_CACHE_HOME\"\ncache_note = \"fixture\"\n"
        + "required_families = [" + ", ".join(f'\"{family}\"' for family in families) + "]\n"
        + "input_files = [\"inputs/part-0001.toml\"]\n", encoding="utf-8")
    entries = []
    for number, family in enumerate(families):
        identifier, digest = f"fixture-{number}", f"{number + 1:064x}"
        entries.append(
            "[[inputs]]\n"
            f'id = "{identifier}"\nsource_family = "{family}"\n'
            f'immutable_url = "https://raw.githubusercontent.com/example/fixture/{REVISION}/payload-{number}"\n'
            f'revision = "{REVISION}"\nsha256 = "{digest}"\nbytes = 1\nlicense = "fixture"\n'
            f'local_cache = "webboxvm-graphics/f02/{identifier}/{digest}.source"\n'
            'generated_code_role = "fixture"\nprovenance = "https://example.invalid/provenance"\n')
    part.write_text("\n".join(entries), encoding="utf-8")
    manifest.with_name("inventory.lock").write_bytes(render_v2_lock(manifest))
    return manifest


class FetchContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(dir=HERE)
        self.addCleanup(self.temporary.cleanup)
        root = Path(self.temporary.name)
        self.cache = ExternalCache.from_path(root / "cache", root / "repository")
        self.source = SourceInput.from_manifest(entry())

    def invalid_manifest(self, name: str) -> Path:
        path = Path(self.temporary.name) / name / "manifest.toml"
        path.parent.mkdir()
        return path

    def test_committed_manifest_loads_without_network(self) -> None:
        self.assertEqual(len(load_manifest(MANIFEST)), 15)

    def test_legacy_v1_inventory_is_rejected_before_cache_actions(self) -> None:
        legacy = self.invalid_manifest("legacy-v1")
        legacy.write_text("schema = 1\n", encoding="utf-8")
        with self.assertRaisesRegex(ContractError, "explicit compatibility"):
            load_manifest(legacy)
        self.assertFalse(self.cache.root.exists())

    def test_manifest_rejects_missing_required_source_family(self) -> None:
        content = MANIFEST.read_text(encoding="utf-8").replace(' "piglit",', '', 1)
        invalid = self.invalid_manifest("missing-family")
        invalid.write_text(content, encoding="utf-8")
        with self.assertRaisesRegex(ContractError, "source-family catalog"):
            load_manifest(invalid)
        self.assertFalse(self.cache.root.exists())

    def test_manifest_rejects_invented_source_family_catalog(self) -> None:
        content = MANIFEST.read_text(encoding="utf-8").replace('"piglit"', '"unapproved"')
        invalid = self.invalid_manifest("invented-family")
        invalid.write_text(content, encoding="utf-8")
        with self.assertRaisesRegex(ContractError, "source-family catalog"):
            load_manifest(invalid)
        self.assertFalse(self.cache.root.exists())

    def test_manifest_rejects_duplicate_entry_source_family(self) -> None:
        invalid = v2_inventory(Path(self.temporary.name) / "duplicate-family")
        part = invalid.parent / "inputs/part-0001.toml"
        part.write_text(part.read_text(encoding="utf-8").replace(
            'source_family = "piglit"', 'source_family = "webgpu-cts"', 1), encoding="utf-8")
        with self.assertRaisesRegex(ContractError, "duplicate id or source family"):
            load_manifest(invalid)
        self.assertFalse(self.cache.root.exists())

    def test_malformed_manifest_is_rejected_before_cache_creation(self) -> None:
        invalid = self.invalid_manifest("malformed")
        invalid.write_bytes(b"\xff")
        with self.assertRaisesRegex(ContractError, "not UTF-8 TOML"):
            load_manifest(invalid)
        self.assertFalse(self.cache.root.exists())

    def test_stale_v2_closure_is_rejected_before_cache_actions(self) -> None:
        for number, name in enumerate(("manifest.toml", "inputs/part-0001.toml", "inventory.lock")):
            with self.subTest(name=name):
                manifest = v2_inventory(Path(self.temporary.name) / f"v2-{number}")
                self.assertEqual(len(load_manifest(manifest)), len(REQUIRED_FAMILIES))
                target = manifest.parent / name
                target.write_bytes(target.read_bytes() + b"# stale\n")
                with self.assertRaisesRegex(ContractError, "inventory.lock"):
                    load_manifest(manifest)
                self.assertFalse(self.cache.root.exists())

    def test_verified_payload_uses_declared_external_cache_path(self) -> None:
        target = atomic_store(self.cache, self.source, PAYLOAD)
        self.assertEqual(target, self.cache.target(self.source))
        self.assertEqual(target.read_bytes(), PAYLOAD)
        self.assertTrue(target.is_relative_to(self.cache.root))

    def test_bad_payload_never_creates_an_accepted_cache_entry(self) -> None:
        with self.assertRaisesRegex(ContractError, "byte count mismatch"):
            atomic_store(self.cache, self.source, PAYLOAD + b"!")
        self.assertFalse(self.cache.target(self.source).exists())

    def test_mutable_or_malformed_url_is_rejected_before_cache_use(self) -> None:
        urls = (
            "http://raw.githubusercontent.com/example/fixture/payload",
            "https://raw.githubusercontent.com/example/fixture/main/payload",
            f"https://raw.githubusercontent.com/example/fixture/{REVISION}/payload?branch=main",
            f"https://gitlab.freedesktop.org/example/fixture/-/raw/{'b' * 40}/payload",
        )
        for value in urls:
            with self.subTest(value=value), self.assertRaises(ContractError):
                SourceInput.from_manifest(entry(immutable_url=value))

    def test_unsafe_cache_root_or_name_is_rejected(self) -> None:
        root = Path(self.temporary.name)
        with self.assertRaisesRegex(ContractError, "outside the repository"):
            ExternalCache.from_path(root / "repository" / "cache", root / "repository")
        with self.assertRaisesRegex(ContractError, "unsafe local_cache"):
            SourceInput.from_manifest(entry(local_cache="../../escape.source"))

    def test_malformed_metadata_leaves_cache_root_absent(self) -> None:
        changes = ({"revision": "main"}, {"bytes": True}, {"bytes": MAX_INPUT_BYTES + 1})
        for change in changes:
            with self.subTest(change=change), self.assertRaises(ContractError):
                SourceInput.from_manifest(entry(**change))
        self.assertFalse(self.cache.root.exists())

    def test_corrupt_existing_cache_is_never_reused(self) -> None:
        target = self.cache.target(self.source)
        target.parent.mkdir(parents=True)
        target.write_bytes(b"bad")
        with self.assertRaisesRegex(ContractError, "byte count mismatch"):
            atomic_store(self.cache, self.source, PAYLOAD)
        self.assertEqual(target.read_bytes(), b"bad")

    def test_redirect_handler_fails_closed(self) -> None:
        with self.assertRaisesRegex(ContractError, "redirect denied"):
            DenyRedirect().redirect_request(None, None, 302, "Found", {}, "https://other.invalid/file")

    def test_valid_digest_still_rejects_wrong_bytes(self) -> None:
        with self.assertRaisesRegex(ContractError, "sha256 mismatch"):
            verify_payload(self.source, b"x" * len(PAYLOAD))


if __name__ == "__main__":
    unittest.main()
