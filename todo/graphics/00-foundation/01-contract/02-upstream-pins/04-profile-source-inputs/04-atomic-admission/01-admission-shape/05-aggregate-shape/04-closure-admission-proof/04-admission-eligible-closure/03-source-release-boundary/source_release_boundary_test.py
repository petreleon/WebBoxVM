#!/usr/bin/env python3
"""Focused hostile tests for the source/release truthfulness boundary."""

from __future__ import annotations

import copy
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import source_release_boundary as subject

HERE = Path(__file__).resolve().parent


def record() -> dict[str, object]:
    value = subject.document(subject.RECORD)
    value["boundary_sha256"] = ""
    value["boundary_sha256"] = subject.hashlib.sha256(subject.canonical(value)).hexdigest()
    return value


class SourceReleaseBoundaryTest(unittest.TestCase):
    def rejected(self, mutate, phrase: str) -> None:
        value = record()
        mutate(value)
        value["boundary_sha256"] = subject.hashlib.sha256(subject.canonical(value)).hexdigest()
        with self.assertRaisesRegex(subject.BoundaryError, phrase):
            subject.boundary_value(value)

    def test_current_draft_is_a_truthful_nonpromotion(self) -> None:
        self.assertEqual(subject.boundary(), record()["boundary_sha256"])

    def test_rejects_source_or_release_promotion(self) -> None:
        self.rejected(lambda value: value["future_source_contract"].__setitem__("source_sufficient", True), "weakens")
        self.rejected(lambda value: value["effects"].__setitem__("conformant", True), "promotion")
        self.rejected(lambda value: value["effects"].__setitem__("conformant", 0), "promotion")
        self.rejected(lambda value: value["release_evidence"].__setitem__("release_claim_permitted", True), "release claim")

    def test_rejects_substitution_of_registry_docs_or_local_selector(self) -> None:
        self.rejected(lambda value: value["vulkan_sources"]["vk_xml"].__setitem__("normative_docs_or_cts", True), "registry metadata")
        self.rejected(lambda value: value["vulkan_sources"]["generated_docs"].__setitem__("may_satisfy_source", True), "generated Docs")
        self.rejected(lambda value: value["vk_default_diagnostics"].__setitem__("local_filtering", "core-only"), "vk-default")

    def test_rejects_missing_role_category_or_anchor(self) -> None:
        self.rejected(lambda value: value["source_roles"].pop(), "every profile role")
        self.rejected(lambda value: value["vk_default_diagnostics"].__setitem__("categories", ["core"]), "vk-default")
        self.rejected(lambda value: value["vulkan_sources"].__setitem__("vcts_handoff_sha256", "1" * 64), "source identities")
        self.rejected(lambda value: value["vulkan_sources"]["vk_xml"].__setitem__("bytes", 1), "registry metadata")

    def test_bounded_reader_rejects_symlink_and_oversize(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = root / "target.json"
            target.write_text("{}")
            link = root / "link.json"
            os.symlink(target, link)
            with self.assertRaisesRegex(subject.BoundaryError, "regular file"):
                subject.document(link)
            large = root / "large.json"
            large.write_bytes(b" " * (subject.MAX_BYTES + 1))
            with self.assertRaisesRegex(subject.BoundaryError, "bounded size"):
                subject.document(large)

    def test_cli_is_read_only_and_rejects_bad_arity(self) -> None:
        before = subject.RECORD.read_bytes()
        result = subprocess.run([sys.executable, str(HERE / "source_release_boundary.py")], text=True, capture_output=True, check=False)
        self.assertEqual(result.returncode, 0)
        self.assertTrue(result.stdout.startswith("BOUNDARY: 6 roles; vk-default diagnostic-only;"))
        self.assertEqual(subject.RECORD.read_bytes(), before)
        malformed = subprocess.run([sys.executable, str(HERE / "source_release_boundary.py"), "one", "two"], text=True, capture_output=True, check=False)
        self.assertEqual(malformed.returncode, 2)
        self.assertIn("usage", malformed.stderr)


if __name__ == "__main__":
    unittest.main()
