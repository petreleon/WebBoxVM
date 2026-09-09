#!/usr/bin/env python3
"""Hermetic F02.2.1 contract tests; F02.2.2 owns network fixtures."""

from __future__ import annotations

import hashlib
import tempfile
import unittest
from pathlib import Path

from source_cache import DenyRedirect, atomic_store, verify_payload
from source_model import ContractError, ExternalCache, MAX_INPUT_BYTES, SourceInput, load_manifest

HERE = Path(__file__).resolve().parent
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


class FetchContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        root = Path(self.temporary.name)
        self.cache = ExternalCache.from_path(root / "cache", root / "repository")
        self.source = SourceInput.from_manifest(entry())

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def test_committed_manifest_loads_without_network(self) -> None:
        self.assertEqual(len(load_manifest(MANIFEST)), 15)

    def test_manifest_rejects_missing_required_source_family(self) -> None:
        content = MANIFEST.read_text(encoding="utf-8").replace(' "piglit",', '', 1)
        invalid = Path(self.temporary.name) / "missing-family.toml"
        invalid.write_text(content, encoding="utf-8")
        with self.assertRaisesRegex(ContractError, "unapproved required source-family catalog"):
            load_manifest(invalid)
        self.assertFalse(self.cache.root.exists())

    def test_manifest_rejects_invented_source_family_catalog(self) -> None:
        content = MANIFEST.read_text(encoding="utf-8").replace('"piglit"', '"unapproved"')
        invalid = Path(self.temporary.name) / "invented-family.toml"
        invalid.write_text(content, encoding="utf-8")
        with self.assertRaisesRegex(ContractError, "unapproved required source-family catalog"):
            load_manifest(invalid)
        self.assertFalse(self.cache.root.exists())

    def test_manifest_rejects_duplicate_entry_source_family(self) -> None:
        content = MANIFEST.read_text(encoding="utf-8").replace(
            'source_family = "piglit"', 'source_family = "webgpu-cts"', 1)
        invalid = Path(self.temporary.name) / "duplicate-family.toml"
        invalid.write_text(content, encoding="utf-8")
        with self.assertRaisesRegex(ContractError, "incomplete source-family coverage"):
            load_manifest(invalid)
        self.assertFalse(self.cache.root.exists())

    def test_malformed_manifest_is_rejected_before_cache_creation(self) -> None:
        invalid = Path(self.temporary.name) / "malformed.toml"
        invalid.write_bytes(b"\xff")
        with self.assertRaisesRegex(ContractError, "manifest cannot be read"):
            load_manifest(invalid)
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
