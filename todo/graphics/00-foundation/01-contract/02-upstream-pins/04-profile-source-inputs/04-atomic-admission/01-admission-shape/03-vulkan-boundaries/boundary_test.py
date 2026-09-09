#!/usr/bin/env python3
"""Hermetic hostile checks for unresolved Vulkan pre-admission boundaries."""

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
BOUNDARIES = HERE / "boundaries.json"
SOURCE_MAP = HERE.parent / "02-source-map/source_map.json"


def module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    value = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = value
    spec.loader.exec_module(value)
    return value


CONTRACT = module("f024_vulkan_boundaries", HERE / "boundary_contract.py")


class BoundaryTests(unittest.TestCase):
    def copied(self, source: Path = BOUNDARIES) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory()
        target = Path(temporary.name) / source.name
        shutil.copyfile(source, target)
        return temporary, target

    def change(self, path: Path, edit) -> None:
        value = json.loads(path.read_text(encoding="utf-8"))
        edit(value)
        path.write_text(json.dumps(value), encoding="utf-8")

    def rejected(self, edit, message: str) -> None:
        temporary, path = self.copied()
        with temporary:
            self.change(path, edit)
            with self.assertRaisesRegex(CONTRACT.BoundaryError, message):
                CONTRACT.validate(path)

    def test_exact_two_roots_stay_unadmitted_with_only_observation_counts(self) -> None:
        records = CONTRACT.validate()
        self.assertEqual([(record.required_input_id, record.state, record.observation_count) for record in records], [
            ("vulkan-14-spec", "unadmitted", 73), ("vulkan-cts-mustpass", "unadmitted", 98)])
        self.assertFalse(any(record.state == "admitted" for record in records))

    def test_each_root_rejects_fabricated_member_or_inventory_fields(self) -> None:
        for index in range(2):
            for name, claimed in (("members", []), ("core_members", []), ("excluded_members", []),
                                  ("member_count", 0), ("local_cache", "invented.cache")):
                self.rejected(lambda value, index=index, name=name, claimed=claimed:
                              value["boundaries"][index].update({name: claimed}), "unexpected schema")

    def test_duplicate_json_field_cannot_choose_a_different_admission_state(self) -> None:
        temporary, path = self.copied()
        with temporary:
            text = path.read_text(encoding="utf-8").replace('"state": "unadmitted",', '"state": "unadmitted", "state": "admitted",', 1)
            path.write_text(text, encoding="utf-8")
            with self.assertRaisesRegex(CONTRACT.BoundaryError, "duplicate JSON"):
                CONTRACT.validate(path)

    def test_state_shape_or_root_fallback_cannot_claim_admission(self) -> None:
        for index in range(2):
            self.rejected(lambda value, index=index: value["boundaries"][index].update(state="admitted"), "pre-admission")
            self.rejected(lambda value, index=index: value["boundaries"][index].update(shape="complete-single-source"), "pre-admission")
            self.rejected(lambda value, index=index: value["boundaries"][index].update(root_fallback="allowed"), "pre-admission")

    def test_missing_or_swapped_audited_blockers_fail(self) -> None:
        self.rejected(lambda value: value["boundaries"][0].update(blocker=""), "audited root")
        self.rejected(lambda value: value["boundaries"][1].update(blocker=value["boundaries"][0]["blocker"]), "audited root")

    def test_docs_requires_all_generated_scope_and_configuration_boundaries(self) -> None:
        self.rejected(lambda value: value["boundaries"][0]["admission_requirements"].pop(), "admission requirement")
        self.rejected(lambda value: value["boundaries"][0]["scope_exclusions"]["video"].clear(), "scope exclusion")
        self.rejected(lambda value: value["boundaries"][0]["observation"].update(boundary="core-only"), "audited root")

    def test_vcts_rejects_scope_expansion_and_oversize_weakening(self) -> None:
        self.rejected(lambda value: value["boundaries"][1]["scope_exclusions"]["extensions"].pop(), "scope exclusion")
        self.rejected(lambda value: value["boundaries"][1]["scope_exclusions"]["extensions"].append("vk-default/api.txt"), "scope exclusion")
        self.rejected(lambda value: value["boundaries"][1]["oversize_inspection"].update(inspected_over_limit_member_count=13), "oversize")
        self.rejected(lambda value: value["boundaries"][1]["oversize_inspection"].update(member_identities="pinned"), "oversize")
        self.rejected(lambda value: value["boundaries"][1]["oversize_inspection"].update(max_input_bytes=1), "oversize")

    def test_stale_source_map_cannot_bypass_the_boundary_record(self) -> None:
        temporary, boundary_path = self.copied()
        with temporary:
            source_map = Path(temporary.name) / "source_map.json"
            shutil.copyfile(SOURCE_MAP, source_map)
            self.change(source_map, lambda value: value["shapes"][4].update(members=[]))
            with self.assertRaisesRegex(CONTRACT.BoundaryError, "source map failed"):
                CONTRACT.validate(boundary_path, source_map)

    def test_cli_reports_only_unadmitted_unresolved_roots(self) -> None:
        result = subprocess.run([sys.executable, str(HERE / "boundary_contract.py"), str(BOUNDARIES)], capture_output=True,
                                text=True, env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), "BOUNDARIES: 2 unresolved Vulkan roots, 0 admitted closures")


if __name__ == "__main__":
    unittest.main()
