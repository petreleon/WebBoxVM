#!/usr/bin/env python3
"""Hermetic checks that the transition design cannot admit today's source records."""

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
RULES = HERE / "post_cutover_rules.json"


def module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    value = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = value
    spec.loader.exec_module(value)
    return value


CONTRACT = module("f024_post_cutover", HERE / "post_cutover_contract.py")


class PostCutoverTests(unittest.TestCase):
    def copied(self, source: Path) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory()
        target = Path(temporary.name) / source.name
        shutil.copyfile(source, target)
        return temporary, target

    def change(self, path: Path, edit) -> None:
        value = json.loads(path.read_text(encoding="utf-8"))
        edit(value)
        path.write_text(json.dumps(value), encoding="utf-8")

    def rejected_rule(self, edit, message: str = "transition rules") -> None:
        temporary, path = self.copied(RULES)
        with temporary:
            self.change(path, edit)
            with self.assertRaisesRegex(CONTRACT.TransitionError, message):
                CONTRACT.validate(path)

    def rejected_map(self, edit) -> None:
        temporary, path = self.copied(CONTRACT.MAP.SOURCE_MAP)
        with temporary:
            self.change(path, edit)
            with self.assertRaisesRegex(CONTRACT.TransitionError, "source map failed"):
                CONTRACT.validate(source_map_path=path)

    def test_current_artifacts_derive_only_the_blocked_pre_admission_result(self) -> None:
        temporary, candidate = self.copied(CONTRACT.MAP.AUDITS["gles-3.2"])
        with temporary:
            audits, observed, source_seen = dict(CONTRACT.MAP.AUDITS, **{"gles-3.2": candidate}), [], []
            original, map_original = CONTRACT.BOUNDARY.validate, CONTRACT.MAP.validate
            def boundary(*args):
                observed.append(args[2])
                return original(*args)
            def source_map(*args):
                source_seen.append(args[1])
                return map_original(*args)
            CONTRACT.BOUNDARY.validate = boundary
            CONTRACT.MAP.validate = source_map
            try:
                result = CONTRACT.validate(audits=audits)
            finally:
                CONTRACT.BOUNDARY.validate = original
                CONTRACT.MAP.validate = map_original
        self.assertEqual((len(source_seen), len(observed)), (1, 1))
        self.assertIs(source_seen[0], observed[0])
        self.assertIsNot(observed[0], audits)
        self.assertEqual(result.required_input_ids, tuple(item[0] for item in CONTRACT.CURRENT))
        self.assertEqual(result.blockers, CONTRACT.BLOCKERS)
        self.assertEqual((result.state, result.cutover_ready), ("pre-admission", False))

    def test_policy_rejects_mixed_partial_or_stale_atomic_rules(self) -> None:
        self.rejected_rule(lambda value: value["atomic_cutover"].update(mixed_phase_records="allowed"), "only valid")
        self.rejected_rule(lambda value: value["atomic_cutover"].update(partial_source_set="allowed"), "only valid")
        self.rejected_rule(lambda value: value["atomic_cutover"].update(inventory_lock="current-lock-allowed"), "only valid")
        self.rejected_rule(lambda value: value["source_shapes"].pop(), "only valid")
        self.rejected_rule(lambda value: value["future_artifacts"][0].update(schema_version=2), "only valid")
        self.rejected_rule(lambda value: value["future_artifacts"][0].update(closure_cardinality="partial-allowed"), "only valid")
        self.rejected_rule(lambda value: value["future_artifacts"][0]["member_fields"].pop(), "only valid")
        self.rejected_rule(lambda value: value["future_artifacts"][0]["scope_identity_fields"].pop(), "only valid")
        self.rejected_rule(lambda value: value["future_artifacts"][0]["closure_fields"].pop(), "only valid")
        self.rejected_rule(lambda value: value["future_artifacts"][0]["closure_semantics"].pop("audit_join"), "only valid")
        self.rejected_rule(lambda value: value["future_artifacts"][0]["closure_semantics"].update(compound="one-member-allowed"), "only valid")
        self.rejected_rule(lambda value: value["future_artifacts"][0]["closure_semantics"].update(root_only="allowed"), "only valid")
        self.rejected_rule(lambda value: value["future_artifacts"][1]["consumer_binding_fields"].pop(), "only valid")
        self.rejected_rule(lambda value: value["future_artifacts"][1].update(cutover_id="missing"), "only valid")

    def test_source_map_requires_the_complete_ordered_six_input_set(self) -> None:
        self.rejected_map(lambda value: value["shapes"].pop())
        self.rejected_map(lambda value: value["shapes"].reverse())
        self.rejected_map(lambda value: value["shapes"][0].update(role="conformance-manifest"))
        self.rejected_map(lambda value: value["shapes"][3]["core_members"].pop())

    def test_root_only_substitution_and_vulkan_fallback_cannot_cross_the_boundary(self) -> None:
        temporary, source_map = self.copied(CONTRACT.MAP.SOURCE_MAP)
        with temporary:
            self.change(source_map, lambda value: value["shapes"][3].update(shape="complete-single-source"))
            with self.assertRaisesRegex(CONTRACT.TransitionError, "source map failed"):
                CONTRACT.validate(source_map_path=source_map)
        temporary, boundary = self.copied(CONTRACT.BOUNDARY.BOUNDARIES)
        with temporary:
            self.change(boundary, lambda value: value["boundaries"][1].update(root_fallback="allowed"))
            with self.assertRaisesRegex(CONTRACT.TransitionError, "Vulkan boundary failed"):
                CONTRACT.validate(boundary_path=boundary)

    def test_stale_candidate_lock_fails_before_any_transition_decision(self) -> None:
        temporary, candidate = self.copied(CONTRACT.MAP.AUDITS["gles-3.2"])
        with temporary:
            self.change(candidate, lambda value: value.update(inventory_sha256="0" * 64))
            audits = dict(CONTRACT.MAP.AUDITS, **{"gles-3.2": candidate})
            with self.assertRaisesRegex(CONTRACT.TransitionError, "source map failed"):
                CONTRACT.validate(audits=audits)
        with self.assertRaisesRegex(CONTRACT.TransitionError, "incomplete candidate-audit bundle"):
            CONTRACT.validate(audits={})

    def test_duplicate_json_fields_cannot_relabel_the_transition(self) -> None:
        temporary, path = self.copied(RULES)
        with temporary:
            text = path.read_text(encoding="utf-8").replace('"phase": "transition-design",', '"phase": "post-cutover", "phase": "transition-design",', 1)
            path.write_text(text, encoding="utf-8")
            with self.assertRaisesRegex(CONTRACT.TransitionError, "duplicate JSON"):
                CONTRACT.validate(path)
        temporary, source_map = self.copied(CONTRACT.MAP.SOURCE_MAP)
        with temporary:
            text = source_map.read_text(encoding="utf-8").replace('"state": "candidate-accepted",', '"state": "admitted", "state": "candidate-accepted",', 1)
            source_map.write_text(text, encoding="utf-8")
            with self.assertRaisesRegex(CONTRACT.TransitionError, "duplicate JSON"):
                CONTRACT.validate(source_map_path=source_map)
        temporary, boundary = self.copied(CONTRACT.BOUNDARY.BOUNDARIES)
        with temporary:
            text = boundary.read_text(encoding="utf-8").replace('"state": "unadmitted",', '"state": "admitted", "state": "unadmitted",', 1)
            boundary.write_text(text, encoding="utf-8")
            with self.assertRaisesRegex(CONTRACT.TransitionError, "duplicate JSON"):
                CONTRACT.validate(boundary_path=boundary)
        temporary, candidate = self.copied(CONTRACT.MAP.AUDITS["opengl-4.6-core"])
        with temporary:
            text = candidate.read_text(encoding="utf-8").replace('"decision": "accepted",', '"decision": "rejected", "decision": "accepted",', 1)
            candidate.write_text(text, encoding="utf-8")
            audits = dict(CONTRACT.MAP.AUDITS, **{"opengl-4.6-core": candidate})
            with self.assertRaisesRegex(CONTRACT.TransitionError, "duplicate JSON"):
                CONTRACT.validate(audits=audits)

    def test_result_constructor_cannot_receive_a_caller_controlled_state(self) -> None:
        with self.assertRaises(TypeError):
            CONTRACT.PreAdmission((), "revision", (), state="post-cutover")
        with self.assertRaises(TypeError):
            CONTRACT.PreAdmission((), "revision", (), cutover_ready=True)

    def test_cli_reports_no_cutover_ready_input(self) -> None:
        result = subprocess.run([sys.executable, str(HERE / "post_cutover_contract.py"), str(RULES)], capture_output=True,
                                text=True, env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), "TRANSITION: pre-admission, 6 required inputs, 0 cutover-ready")


if __name__ == "__main__":
    unittest.main()
