#!/usr/bin/env python3
"""Hermetic checks for the only current aggregate: blocked pre-admission."""

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
RULES = HERE.parent.parent / "04-post-cutover-rules/post_cutover_rules.json"


def module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    value = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = value
    spec.loader.exec_module(value)
    return value


CONTRACT = module("f024_pre_admission_aggregate", HERE / "aggregate_contract.py")


class AggregateTests(unittest.TestCase):
    def copied(self, source: Path) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory()
        target = Path(temporary.name) / source.name
        shutil.copyfile(source, target)
        return temporary, target

    def change(self, path: Path, edit) -> None:
        value = json.loads(path.read_text(encoding="utf-8"))
        edit(value)
        path.write_text(json.dumps(value), encoding="utf-8")

    def rejected_map(self, edit) -> None:
        temporary, path = self.copied(CONTRACT.TRANSITION.MAP.SOURCE_MAP)
        with temporary:
            self.change(path, edit)
            with self.assertRaisesRegex(CONTRACT.AggregateError, "transition failed"):
                CONTRACT.validate(source_map_path=path)

    def test_current_artifacts_return_only_the_exact_blocked_aggregate(self) -> None:
        result = CONTRACT.validate()
        self.assertEqual(result.required_input_ids, tuple(item[0] for item in CONTRACT.TRANSITION.CURRENT))
        self.assertEqual(result.direct_candidate_ids, CONTRACT.DIRECT)
        self.assertEqual(result.blockers, CONTRACT.BLOCKERS)
        counts = (
            len(result.gles.core_members),
            len(result.gles.excluded_members),
            *result.gles.configuration_counts,
        )
        self.assertEqual(counts, CONTRACT.GLES_COUNTS)
        observed = [(item.required_input_id, item.observation_count) for item in result.vulkan]
        self.assertEqual(observed, list(CONTRACT.VULKAN))
        state = (result.state, result.admission_eligible, result.cutover_ready)
        self.assertEqual(state, ("pre-admission", False, False))
        command = subprocess.run(
            [sys.executable, str(HERE / "aggregate_contract.py"), str(RULES)],
            capture_output=True,
            text=True,
            env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"),
        )
        self.assertEqual(command.returncode, 0, command.stderr)
        expected = (
            "AGGREGATE: pre-admission, 6 required inputs, 3 blockers, "
            "0 cutover-ready"
        )
        self.assertEqual(command.stdout.strip(), expected)

    def test_source_map_requires_all_six_ordered_records(self) -> None:
        self.rejected_map(lambda value: value["shapes"].pop())
        self.rejected_map(lambda value: value["shapes"].reverse())
        self.rejected_map(lambda value: value["shapes"][0].update(role="conformance-manifest"))

    def test_compound_shapes_reject_root_only_or_member_erosion(self) -> None:
        self.rejected_map(lambda value: value["shapes"][3].update(shape="complete-single-source"))
        self.rejected_map(lambda value: value["shapes"][3]["core_members"].pop())
        self.rejected_map(lambda value: value["shapes"][4].update(members=[]))

    def test_vulkan_boundaries_reject_fallback_or_scope_weakening(self) -> None:
        temporary, path = self.copied(CONTRACT.TRANSITION.BOUNDARY.BOUNDARIES)
        with temporary:
            self.change(path, lambda value: value["boundaries"][0].update(root_fallback="allowed"))
            with self.assertRaisesRegex(CONTRACT.AggregateError, "transition failed"):
                CONTRACT.validate(boundary_path=path)
        temporary, path = self.copied(CONTRACT.TRANSITION.BOUNDARY.BOUNDARIES)
        with temporary:
            self.change(path, lambda value: value["boundaries"][1]["scope_exclusions"]["extensions"].pop())
            with self.assertRaisesRegex(CONTRACT.AggregateError, "transition failed"):
                CONTRACT.validate(boundary_path=path)

    def test_stale_or_partial_audit_bundle_cannot_return_an_aggregate(self) -> None:
        temporary, candidate = self.copied(CONTRACT.TRANSITION.MAP.AUDITS["gles-3.2"])
        with temporary:
            self.change(candidate, lambda value: value.update(inventory_sha256="0" * 64))
            audits = dict(CONTRACT.TRANSITION.MAP.AUDITS, **{"gles-3.2": candidate})
            with self.assertRaisesRegex(CONTRACT.AggregateError, "transition failed"):
                CONTRACT.validate(audits=audits)
        with self.assertRaisesRegex(CONTRACT.AggregateError, "transition failed"):
            CONTRACT.validate(audits={})

    def test_mixed_or_partial_transition_policy_cannot_return_an_aggregate(self) -> None:
        temporary, path = self.copied(RULES)
        with temporary:
            self.change(path, lambda value: value["atomic_cutover"].update(mixed_phase_records="allowed"))
            with self.assertRaisesRegex(CONTRACT.AggregateError, "transition failed"):
                CONTRACT.validate(rules_path=path)
        temporary, path = self.copied(RULES)
        with temporary:
            self.change(path, lambda value: value["atomic_cutover"].update(partial_source_set="allowed"))
            with self.assertRaisesRegex(CONTRACT.AggregateError, "transition failed"):
                CONTRACT.validate(rules_path=path)

    def test_injected_rule_map_boundary_and_audit_paths_reach_the_transition(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        with temporary:
            root = Path(temporary.name)
            names = ("rules.json", "map.json", "boundary.json", "candidate.json")
            rules, source_map, boundary, candidate = (root / name for name in names)
            copies = (
                (RULES, rules),
                (CONTRACT.TRANSITION.MAP.SOURCE_MAP, source_map),
                (CONTRACT.TRANSITION.BOUNDARY.BOUNDARIES, boundary),
                (CONTRACT.TRANSITION.MAP.AUDITS["gles-3.2"], candidate),
            )
            for source, target in copies:
                shutil.copyfile(source, target)
            audits, seen = dict(CONTRACT.TRANSITION.MAP.AUDITS, **{"gles-3.2": candidate}), []
            original = CONTRACT.TRANSITION.validate
            def transition(*args):
                seen.append(args)
                return original(*args)
            CONTRACT.TRANSITION.validate = transition
            try:
                CONTRACT.validate(rules, source_map, boundary, audits)
            finally:
                CONTRACT.TRANSITION.validate = original
        self.assertEqual(seen, [(rules, source_map, boundary, audits)])

    def test_duplicate_json_and_constructor_cannot_claim_a_ready_state(self) -> None:
        temporary, path = self.copied(CONTRACT.TRANSITION.MAP.SOURCE_MAP)
        with temporary:
            text = path.read_text(encoding="utf-8").replace(
                '"state": "candidate-accepted",',
                '"state": "admitted", "state": "candidate-accepted",',
                1,
            )
            path.write_text(text, encoding="utf-8")
            with self.assertRaisesRegex(CONTRACT.AggregateError, "transition failed"):
                CONTRACT.validate(source_map_path=path)
        with self.assertRaises(TypeError):
            CONTRACT.PreAdmissionAggregate((), (), (), "revision", None, (), state="post-cutover")


if __name__ == "__main__":
    unittest.main()
