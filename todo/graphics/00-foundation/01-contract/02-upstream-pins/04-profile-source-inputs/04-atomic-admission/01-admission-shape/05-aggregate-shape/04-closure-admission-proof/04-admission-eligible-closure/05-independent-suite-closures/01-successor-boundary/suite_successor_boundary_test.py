#!/usr/bin/env python3
"""Hermetic positive and hostile checks for the independent-suite policy boundary."""

import hashlib
import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent


def module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    value = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = value
    spec.loader.exec_module(value)
    return value


BOUNDARY = module("f025_successor_boundary", HERE / "suite_successor_boundary.py")


class BoundaryTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def copy(self, name: str) -> Path:
        target = Path(self.temporary.name) / name
        target.write_bytes(BOUNDARY.RECORD.read_bytes())
        return target

    def value(self, path: Path) -> dict[str, object]:
        return json.loads(path.read_text(encoding="utf-8"))

    def seal(self, value: dict[str, object]) -> None:
        value["policy_sha256"] = BOUNDARY.digest(value)

    def write(self, path: Path, value: dict[str, object]) -> None:
        path.write_text(json.dumps(value, sort_keys=True), encoding="utf-8")

    def reject(self, edit) -> None:
        path = self.copy("mutated.json")
        value = self.value(path)
        edit(value)
        self.seal(value)
        self.write(path, value)
        with self.assertRaisesRegex(BOUNDARY.BoundaryError, "exact blocked successor boundary"):
            BOUNDARY.validate(path)

    def test_committed_policy_is_read_only_and_reports_only_a_boundary(self) -> None:
        watched = (BOUNDARY.RECORD, BOUNDARY.CANDIDATES, BOUNDARY.CLOSURE, BOUNDARY.CONFIGURATIONS,
                   BOUNDARY.HANDOFF.HANDOFF, BOUNDARY.HANDOFF.TAXONOMY_RECEIPT, BOUNDARY.RECONCILER,
                   BOUNDARY.REQUIREMENTS, BOUNDARY.INVENTORY_LOCK)
        before = {path: hashlib.sha256(path.read_bytes()).hexdigest() for path in watched}
        value = BOUNDARY.validate()
        self.assertEqual(value["status"], "policy-only-unadmitted")
        self.assertEqual(value["effects"], {name: False for name in BOUNDARY.EFFECTS})
        self.assertEqual(value["gles"]["successor_closure_ready"], False)
        self.assertEqual(value["vcts"]["core_manifest_ready"], False)
        result = subprocess.run([sys.executable, "-B", str(HERE / "suite_successor_boundary.py")],
                                capture_output=True, text=True, env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), f"POLICY: policy-only-unadmitted {value['policy_sha256']}")
        self.assertEqual(before, {path: hashlib.sha256(path.read_bytes()).hexdigest() for path in watched})

    def test_resealed_promotion_scope_and_cap_mutations_are_rejected(self) -> None:
        cases = (
            lambda value: value["effects"].__setitem__("admitted", True),
            lambda value: value["effects"].__setitem__("conformant", 0),
            lambda value: value["gles"].__setitem__("successor_closure_ready", True),
            lambda value: value["gles"]["core_members"].pop(),
            lambda value: value["gles"].__setitem__("excluded_extension", []),
            lambda value: value["vcts"].__setitem__("selector_scope", "vulkan-1.4-core"),
            lambda value: value["vcts"].__setitem__("local_selector_allowed", True),
            lambda value: value["vcts"].__setitem__("taxonomy_is_conformance", True),
            lambda value: value["vcts"]["category_counts"].__setitem__("core", 92),
            lambda value: value["vcts"].__setitem__("over_f02_cap_member_count", 0),
            lambda value: value["vcts"].__setitem__("core_manifest_ready", True),
            lambda value: value["anchors"].__setitem__("gles_closure_document_sha256", "0" * 64),
        )
        for edit in cases:
            with self.subTest(edit=edit):
                self.reject(edit)

    def test_self_hash_duplicate_oversize_fifo_and_symlink_are_rejected(self) -> None:
        path = self.copy("selfhash.json")
        value = self.value(path)
        value["policy_sha256"] = "0" * 64
        self.write(path, value)
        with self.assertRaisesRegex(BOUNDARY.BoundaryError, "self-hash"):
            BOUNDARY.validate(path)
        root = Path(self.temporary.name)
        duplicate = root / "duplicate.json"
        duplicate.write_text('{"schema":1,"schema":1}', encoding="utf-8")
        with self.assertRaisesRegex(BOUNDARY.BoundaryError, "duplicate"):
            BOUNDARY.validate(duplicate)
        oversized = root / "oversized.json"
        oversized.write_bytes(b"x" * (BOUNDARY.MAX_BYTES + 1))
        with self.assertRaisesRegex(BOUNDARY.BoundaryError, "bounded size"):
            BOUNDARY.validate(oversized)
        fifo = root / "policy.fifo"
        os.mkfifo(fifo)
        with self.assertRaisesRegex(BOUNDARY.BoundaryError, "regular file"):
            BOUNDARY.validate(fifo)
        link = root / "policy-link.json"
        link.symlink_to(BOUNDARY.RECORD)
        with self.assertRaisesRegex(BOUNDARY.BoundaryError, "regular file"):
            BOUNDARY.validate(link)


if __name__ == "__main__":
    unittest.main(verbosity=2)
