#!/usr/bin/env python3
"""Hermetic positive and hostile checks for the Vulkan 1.4 input audit."""

from __future__ import annotations

import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
PARENT = HERE.parent
CANDIDATES = HERE / "candidates.json"
REFERENCES = HERE / "mustpass_references.json"


def module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    value = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = value
    spec.loader.exec_module(value)
    return value


AUDIT = module("f024_vulkan_candidates", PARENT / "candidate_contract.py")
REFERENCE = module("f024_vulkan_references", HERE / "mustpass_reference_contract.py")


class VulkanAuditTests(unittest.TestCase):
    def copied(self) -> tuple[tempfile.TemporaryDirectory[str], Path, Path]:
        temporary = tempfile.TemporaryDirectory()
        candidate, references = Path(temporary.name) / "candidates.json", Path(temporary.name) / "references.json"
        shutil.copyfile(CANDIDATES, candidate)
        shutil.copyfile(REFERENCES, references)
        return temporary, candidate, references

    def change(self, path: Path, edit) -> None:
        value = json.loads(path.read_text(encoding="utf-8"))
        edit(value)
        path.write_text(json.dumps(value), encoding="utf-8")

    def test_reviewed_roots_and_direct_reference_list_are_rejected_exactly(self) -> None:
        self.assertEqual(AUDIT.validate(CANDIDATES, "vulkan-1.4-core"), ("rejected", "rejected"))
        self.assertEqual(REFERENCE.validate(REFERENCES, CANDIDATES, "vulkan-1.4-core"), 98)

    def test_mutable_or_substituted_spec_source_is_rejected(self) -> None:
        temporary, candidate, _ = self.copied()
        with temporary:
            self.change(candidate, lambda value: value["candidates"][0]["entry"].update(revision="a" * 40))
            with self.assertRaisesRegex(AUDIT.AuditError, "pinned raw source"):
                AUDIT.validate(candidate, "vulkan-1.4-core")
        temporary, candidate, _ = self.copied()
        with temporary:
            self.change(candidate, lambda value: value["candidates"][0]["entry"].update(source_family="alternate"))
            with self.assertRaisesRegex(AUDIT.AuditError, "reviewed exact"):
                AUDIT.validate(candidate, "vulkan-1.4-core")

    def test_every_reviewed_spec_metadata_field_is_bound_exactly(self) -> None:
        alternate = "a" * 64
        changes = (("immutable_url", "https://raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/"
                    "f84d432d5b8912362f96f581f29bbc4f3c8c7843/README.adoc", "selector does not name"),
                   ("sha256", alternate, "reviewed exact"), ("bytes", 8686, "reviewed exact"),
                   ("license", "unreviewed", "reviewed exact"),
                   ("local_cache", "webboxvm-graphics/f02/vulkan-14-spec/"
                    "069b7e6d6326969df7b4a86f189f7ba22359e93ca7aa76c23301504667d3c4b0.cache", "F02.2 policy"),
                   ("generated_code_role", "unreviewed", "reviewed exact"),
                   ("provenance", "https://example.invalid/review", "reviewed exact"))
        for field, replacement, message in changes:
            with self.subTest(field=field):
                temporary, candidate, _ = self.copied()
                with temporary:
                    def edit(value, field=field, replacement=replacement):
                        entry = value["candidates"][0]["entry"]
                        entry[field] = replacement
                        if field == "sha256":
                            entry["local_cache"] = f"webboxvm-graphics/f02/vulkan-14-spec/{replacement}.source"
                    self.change(candidate, edit)
                    with self.assertRaisesRegex(AUDIT.AuditError, message):
                        AUDIT.validate(candidate, "vulkan-1.4-core")

    def test_compound_roots_cannot_be_rewritten_as_complete_sources(self) -> None:
        temporary, candidate, _ = self.copied()
        with temporary:
            self.change(candidate, lambda value: value["candidates"][1].update(decision="accepted"))
            with self.assertRaisesRegex(AUDIT.AuditError, "reviewed exact selector"):
                AUDIT.validate(candidate, "vulkan-1.4-core")
        temporary, candidate, _ = self.copied()
        with temporary:
            self.change(candidate, lambda value: value["candidates"][1].update(selector_case_count=0))
            with self.assertRaisesRegex(AUDIT.AuditError, "selector count"):
                AUDIT.validate(candidate, "vulkan-1.4-core")

    def test_reference_list_must_reproduce_the_pinned_root(self) -> None:
        temporary, candidate, references = self.copied()
        with temporary:
            self.change(references, lambda value: value["direct_references"].reverse())
            with self.assertRaisesRegex(REFERENCE.ReferenceError, "reviewed root bytes"):
                REFERENCE.validate(references, candidate, "vulkan-1.4-core")
        temporary, candidate, references = self.copied()
        with temporary:
            self.change(references, lambda value: value["direct_references"].pop())
            with self.assertRaisesRegex(REFERENCE.ReferenceError, "incorrect count"):
                REFERENCE.validate(references, candidate, "vulkan-1.4-core")

    def test_reference_paths_and_types_fail_closed(self) -> None:
        temporary, candidate, references = self.copied()
        with temporary:
            self.change(references, lambda value: value["direct_references"].__setitem__(0, "../escape.txt"))
            with self.assertRaisesRegex(REFERENCE.ReferenceError, "path is unsafe"):
                REFERENCE.validate(references, candidate, "vulkan-1.4-core")
        temporary, candidate, references = self.copied()
        with temporary:
            self.change(references, lambda value: value.update(direct_reference_count=98.0))
            with self.assertRaisesRegex(REFERENCE.ReferenceError, "stale direct_reference_count"):
                REFERENCE.validate(references, candidate, "vulkan-1.4-core")
        temporary, candidate, references = self.copied()
        with temporary:
            self.change(references, lambda value: value["direct_references"].__setitem__(1, value["direct_references"][0]))
            with self.assertRaisesRegex(REFERENCE.ReferenceError, "repeat a path"):
                REFERENCE.validate(references, candidate, "vulkan-1.4-core")

    def test_reference_scope_boundary_cannot_be_narrowed_or_erased(self) -> None:
        changes = (lambda value: value.update(scope_boundary="core-only"),
                   lambda value: value["scope_boundary_examples"].pop())
        for edit in changes:
            with self.subTest(edit=edit):
                temporary, candidate, references = self.copied()
                with temporary:
                    self.change(references, edit)
                    with self.assertRaisesRegex(REFERENCE.ReferenceError, "core scope boundary"):
                        REFERENCE.validate(references, candidate, "vulkan-1.4-core")

    def test_reference_root_binding_and_schema_are_strict(self) -> None:
        temporary, candidate, references = self.copied()
        with temporary:
            self.change(references, lambda value: value.update(candidate_sha256="0" * 64))
            with self.assertRaisesRegex(REFERENCE.ReferenceError, "reviewed root"):
                REFERENCE.validate(references, candidate, "vulkan-1.4-core")
        temporary, candidate, references = self.copied()
        with temporary:
            self.change(references, lambda value: value.update(schema=True))
            with self.assertRaisesRegex(REFERENCE.ReferenceError, "schema version"):
                REFERENCE.validate(references, candidate, "vulkan-1.4-core")

    def test_invalid_candidate_is_wrapped_by_reference_contract(self) -> None:
        temporary, candidate, references = self.copied()
        with temporary:
            self.change(candidate, lambda value: value.update(schema=True))
            with self.assertRaisesRegex(REFERENCE.ReferenceError, "candidate contract failed"):
                REFERENCE.validate(references, candidate, "vulkan-1.4-core")

    def test_clis_report_rejected_root_and_reference_count(self) -> None:
        environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
        audit = subprocess.run([sys.executable, str(PARENT / "candidate_contract.py"), str(CANDIDATES), "vulkan-1.4-core"],
                               capture_output=True, text=True, env=environment)
        references = subprocess.run([sys.executable, str(HERE / "mustpass_reference_contract.py"), str(REFERENCES),
                                    str(CANDIDATES), "vulkan-1.4-core"], capture_output=True, text=True, env=environment)
        self.assertEqual(audit.returncode, 0, audit.stderr)
        self.assertEqual(audit.stdout.strip(), "AUDIT: vulkan-1.4-core rejected, rejected")
        self.assertEqual(references.returncode, 0, references.stderr)
        self.assertEqual(references.stdout.strip(), "REFERENCES: vulkan-1.4-core 98 direct must-pass selectors")


if __name__ == "__main__":
    unittest.main()
