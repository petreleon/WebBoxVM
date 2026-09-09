#!/usr/bin/env python3
"""Hermetic positive and hostile checks for the six-record source shape map."""

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
MAP = HERE / "source_map.json"


def module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    value = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = value
    spec.loader.exec_module(value)
    return value


CONTRACT = module("f024_source_map", HERE / "source_map_contract.py")


class SourceMapTests(unittest.TestCase):
    def copied(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory()
        target = Path(temporary.name) / "source_map.json"
        shutil.copyfile(MAP, target)
        return temporary, target

    def change(self, path: Path, edit) -> None:
        value = json.loads(path.read_text(encoding="utf-8"))
        edit(value)
        path.write_text(json.dumps(value), encoding="utf-8")

    def rejected(self, edit, message: str = "source map") -> None:
        temporary, path = self.copied()
        with temporary:
            self.change(path, edit)
            with self.assertRaisesRegex(CONTRACT.SourceMapError, message):
                CONTRACT.validate(path)

    def test_exact_map_has_three_direct_one_closure_and_two_unresolved_roots(self) -> None:
        records = CONTRACT.validate()
        self.assertEqual([(record.required_input_id, record.shape, record.state) for record in records], [
            ("opengl-46-core-spec", "complete-single-source", "candidate-accepted"),
            ("opengl-cts-manifest", "complete-single-source", "candidate-accepted"),
            ("gles-32-spec", "complete-single-source", "candidate-accepted"),
            ("gles-cts-manifest", "bounded-unadmitted-closure", "unadmitted"),
            ("vulkan-14-spec", "unresolved-root", "unadmitted"),
            ("vulkan-cts-mustpass", "unresolved-root", "unadmitted"),
        ])
        self.assertEqual((records[3].member_count, records[3].configuration_count), (4, 12))
        self.assertEqual((records[4].observation_count, records[5].observation_count), (73, 98))
        self.assertFalse(any(record.state == "admitted" for record in records))

    def test_omission_extra_reorder_and_role_swap_fail_exact_six_requirement_mapping(self) -> None:
        self.rejected(lambda value: value.update(schema=True), "schema version")
        self.rejected(lambda value: value["shapes"].pop(), "exactly enumerate")
        self.rejected(lambda value: value["shapes"].append(value["shapes"][0]), "exactly enumerate")
        self.rejected(lambda value: value["shapes"].reverse(), "exactly enumerate")
        self.rejected(lambda value: value["shapes"][0].update(role="conformance-manifest"), "exactly enumerate")

    def test_stale_roots_and_retyped_complete_sources_are_rejected(self) -> None:
        self.rejected(lambda value: value["shapes"][3].update(root_sha256="0" * 64), "reviewed root")
        self.rejected(lambda value: value["shapes"][0].update(shape="bounded-unadmitted-closure"), "invalid admission shape")
        self.rejected(lambda value: value["shapes"][2].update(state="admitted"), "invalid admission shape")

    def test_stale_or_reclassified_audit_root_remains_authoritative(self) -> None:
        temporary, source_map = self.copied()
        with temporary:
            candidate = Path(temporary.name) / "candidates.json"
            shutil.copyfile(CONTRACT.AUDITS["gles-3.2"], candidate)
            self.change(candidate, lambda value: value["candidates"][1]["entry"].update(revision="a" * 40))
            audits = dict(CONTRACT.AUDITS, **{"gles-3.2": candidate})
            with self.assertRaisesRegex(CONTRACT.SourceMapError, "candidate audit failed"):
                CONTRACT.validate(source_map, audits)

    def test_gles_member_exclusion_and_configuration_bindings_are_exact(self) -> None:
        self.rejected(lambda value: value["shapes"][3]["core_members"].pop(), "exact members")
        self.rejected(lambda value: value["shapes"][3]["core_members"].append(value["shapes"][3]["excluded_members"][0]), "exact members")
        self.rejected(lambda value: value["shapes"][3].update(configuration_sha256="0" * 64), "stale configuration")

    def test_unresolved_vulkan_roots_cannot_gain_members_or_lose_their_boundaries(self) -> None:
        self.rejected(lambda value: value["shapes"][4].update(members=[]), "unexpected schema")
        self.rejected(lambda value: value["shapes"][4].update(shape="complete-single-source"), "invalid admission state")
        self.rejected(lambda value: value["shapes"][5].update(blocker=""), "invalid admission state")
        self.rejected(lambda value: value["shapes"][5]["observation"].update(count=0), "boundary observation")

    def test_injected_paths_must_be_complete_path_values(self) -> None:
        with self.assertRaisesRegex(CONTRACT.SourceMapError, "invalid auxiliary input path"):
            CONTRACT.validate(includes_path="not-a-path")
        with self.assertRaisesRegex(CONTRACT.SourceMapError, "incomplete candidate-audit path set"):
            CONTRACT.validate(audits=None)
        with self.assertRaisesRegex(CONTRACT.SourceMapError, "incomplete candidate-audit path set"):
            CONTRACT.validate(audits={"gles-3.2": CONTRACT.AUDITS["gles-3.2"]})

    def test_cli_reports_the_only_pre_admission_shape_counts(self) -> None:
        result = subprocess.run([sys.executable, str(HERE / "source_map_contract.py"), str(MAP)], capture_output=True,
                                text=True, env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), "MAP: 6 required inputs, 3 complete single-source, 1 unadmitted closure, 2 unresolved roots")


if __name__ == "__main__":
    unittest.main()
