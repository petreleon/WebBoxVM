#!/usr/bin/env python3
"""Hermetic checks for the F03.1 profile scope and input-sufficiency gate."""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from validate_profile_scope import MANIFEST, REQUIREMENTS_PATH, SCOPE, ScopeError, inventory, validate
from matrix_contract import validate_matrix
from profile_contract import PROFILES, document, profile_blocker, source_gaps

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[5]
GAPS = (
    "opengl-46-core-spec", "opengl-cts-manifest", "gles-32-spec", "gles-cts-manifest",
    "vulkan-14-spec", "vulkan-cts-mustpass",
)


class ProfileScopeTests(unittest.TestCase):
    def copied(self) -> tuple[tempfile.TemporaryDirectory[str], Path, Path]:
        temporary = tempfile.TemporaryDirectory(dir=ROOT)
        root = Path(temporary.name)
        scope, requirements = root / "scope.json", root / "requirements.json"
        shutil.copyfile(SCOPE, scope)
        shutil.copyfile(REQUIREMENTS_PATH, requirements)
        return temporary, scope, requirements

    def change(self, path: Path, edit) -> None:
        value = json.loads(path.read_text(encoding="utf-8"))
        edit(value)
        path.write_text(json.dumps(value), encoding="utf-8")

    def test_current_scope_has_exact_visible_gaps(self) -> None:
        self.assertEqual(validate(), GAPS)

    def test_stale_scope_lock_is_rejected(self) -> None:
        temporary, scope, requirements = self.copied()
        with temporary:
            self.change(scope, lambda value: value.update(inventory_sha256="0" * 64))
            with self.assertRaisesRegex(ScopeError, "stale inventory"):
                validate(scope, requirements, MANIFEST)

    def test_supported_profile_is_rejected_while_inventory_is_incomplete(self) -> None:
        temporary, scope, requirements = self.copied()
        with temporary:
            self.change(scope, lambda value: value["profiles"][0].update(status="supported"))
            with self.assertRaisesRegex(ScopeError, "blocked matrix state"):
                validate(scope, requirements, MANIFEST)

    def test_unknown_related_input_is_rejected(self) -> None:
        temporary, scope, requirements = self.copied()
        with temporary:
            self.change(requirements, lambda value: value["requirements"][0].update(related_input_ids=["unknown"]))
            with self.assertRaisesRegex(ScopeError, "unknown related"):
                validate(scope, requirements, MANIFEST)

    def test_duplicate_profile_role_is_rejected(self) -> None:
        temporary, scope, requirements = self.copied()
        with temporary:
            self.change(requirements, lambda value: value["requirements"].append(value["requirements"][0]))
            with self.assertRaisesRegex(ScopeError, "duplicate"):
                validate(scope, requirements, MANIFEST)

    def test_missing_profile_role_is_rejected(self) -> None:
        temporary, scope, requirements = self.copied()
        with temporary:
            self.change(requirements, lambda value: value["requirements"].pop())
            with self.assertRaisesRegex(ScopeError, "reviewed profile catalog"):
                validate(scope, requirements, MANIFEST)

    def test_existing_unrelated_input_cannot_discharge_a_future_source(self) -> None:
        temporary, scope, requirements = self.copied()
        with temporary:
            self.change(requirements, lambda value: value["requirements"][0].update(required_input_id="opengl-gles-registry"))
            with self.assertRaisesRegex(ScopeError, "reviewed profile catalog"):
                validate(scope, requirements, MANIFEST)

    def test_missing_inputs_require_the_inventory_blocker(self) -> None:
        temporary, scope, requirements = self.copied()
        with temporary:
            self.change(scope, lambda value: [item.update(blocker="matrix-incomplete") for item in value["profiles"]])
            with self.assertRaisesRegex(ScopeError, "inventory-sources-incomplete"):
                validate(scope, requirements, MANIFEST)

    def test_future_admission_moves_the_source_gate_to_matrix_work(self) -> None:
        revision, inputs = inventory(MANIFEST)
        profiles = {item[0] for item in PROFILES}
        self.assertEqual(source_gaps(document(REQUIREMENTS_PATH), revision, set(inputs) | set(GAPS), profiles), ())
        self.assertEqual(profile_blocker(()), "matrix-incomplete")

    def write_matrix(self, value: dict[str, object]) -> Path:
        temporary = tempfile.TemporaryDirectory(dir=ROOT)
        self.addCleanup(temporary.cleanup)
        path = Path(temporary.name) / "matrix.json"
        (path.parent / "evidence.md").write_text("# synthetic evidence\n", encoding="utf-8")
        path.write_text(json.dumps(value), encoding="utf-8")
        return path

    def matrix(self) -> tuple[dict[str, object], str, dict[str, dict[str, object]]]:
        revision, inputs = inventory(MANIFEST)
        inputs = dict(inputs)
        source = inputs["opengl-gles-registry"]
        test = {"id": "opengl-cts-manifest", "revision": "a" * 40, "sha256": "b" * 64}
        inputs[test["id"]] = test
        return {
            "schema": 1, "inventory_sha256": revision,
            "rows": [{
                "profile": "opengl-4.6-core", "requirement_kind": "command", "name": "glExample",
                "mandatory": True, "source_id": source["id"], "source_revision": source["revision"],
                "source_sha256": source["sha256"], "source_locator": "gl.xml#glExample",
                "condition": "always", "owner_task": "G01", "test_source_id": test["id"],
                "test_source_revision": test["revision"], "test_source_sha256": test["sha256"],
                "test_selector": "spec.example", "status": "blocked", "evidence": "evidence.md#blocked",
                "blocker": "implementation-pending",
            }],
        }, revision, inputs

    def test_matrix_contract_accepts_a_lock_bound_blocked_row(self) -> None:
        matrix, revision, inputs = self.matrix()
        path = self.write_matrix(matrix)
        self.assertEqual(validate_matrix(path, revision, inputs, {"opengl-4.6-core"}), 1)

    def test_matrix_contract_rejects_unbound_mandatory_source(self) -> None:
        matrix, revision, inputs = self.matrix()
        matrix["rows"][0]["test_source_sha256"] = "0" * 64
        path = self.write_matrix(matrix)
        with self.assertRaisesRegex(ScopeError, "mismatched test_source"):
            validate_matrix(path, revision, inputs, {"opengl-4.6-core"})

    def test_matrix_contract_rejects_missing_mandatory_owner(self) -> None:
        matrix, revision, inputs = self.matrix()
        matrix["rows"][0]["owner_task"] = ""
        path = self.write_matrix(matrix)
        with self.assertRaisesRegex(ScopeError, "empty owner_task"):
            validate_matrix(path, revision, inputs, {"opengl-4.6-core"})

    def test_matrix_contract_rejects_a_nonindependent_test_source(self) -> None:
        matrix, revision, inputs = self.matrix()
        row, source = matrix["rows"][0], inputs["opengl-gles-registry"]
        row.update(test_source_id=source["id"], test_source_revision=source["revision"], test_source_sha256=source["sha256"])
        with self.assertRaisesRegex(ScopeError, "unrelated normative"):
            validate_matrix(self.write_matrix(matrix), revision, inputs, {"opengl-4.6-core"})

    def test_matrix_contract_rejects_profile_cross_binding_and_placeholder_evidence(self) -> None:
        matrix, revision, inputs = self.matrix()
        row, source = matrix["rows"][0], inputs["webgpu-spec"]
        row.update(source_id=source["id"], source_revision=source["revision"], source_sha256=source["sha256"])
        with self.assertRaisesRegex(ScopeError, "unrelated normative"):
            validate_matrix(self.write_matrix(matrix), revision, inputs, {"opengl-4.6-core"})
        matrix, revision, inputs = self.matrix()
        matrix["rows"][0].update(status="supported", blocker="", evidence="none")
        with self.assertRaisesRegex(ScopeError, "empty evidence"):
            validate_matrix(self.write_matrix(matrix), revision, inputs, {"opengl-4.6-core"})

    def test_malformed_matrix_cli_is_fail_closed(self) -> None:
        path = self.write_matrix({})
        result = subprocess.run(
            [sys.executable, str(HERE / "validate_profile_scope.py"), "--matrix", str(path)],
            capture_output=True, text=True, check=False,
        )
        self.assertEqual(result.returncode, 2)
        self.assertIn("FAIL: matrix does not match schema version 1", result.stderr)
        decoy = ("import sys,types; m=types.ModuleType('matrix_contract'); m.validate_matrix=lambda *a: None; "
                 f"sys.modules['matrix_contract']=m; sys.argv=['gate','--matrix',{str(path)!r}]; "
                 "import validate_profile_scope; validate_profile_scope.main()")
        result = subprocess.run([sys.executable, "-c", decoy], cwd=HERE, capture_output=True, text=True, check=False)
        self.assertEqual(result.returncode, 2)
        self.assertIn("FAIL: matrix does not match schema version 1", result.stderr)


if __name__ == "__main__":
    unittest.main()
