#!/usr/bin/env python3
"""Focused positive and hostile checks for F03.2.2.5.1."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
import types
import unittest
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
PROBE = HERE / "opengl_command_domain_classification.py"
LIVE_ROOT = Path("/private/tmp/webboxvm-f0341.cqT6ZX")


def load():
    spec = importlib.util.spec_from_file_location("f032251_test_catalog", PROBE)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


CATALOG = load()


class CommandDomainClassificationTests(unittest.TestCase):
    def copied(self):
        temporary = tempfile.TemporaryDirectory()
        bundle = Path(temporary.name) / "bundle"
        bundle.mkdir()
        for name in CATALOG.ARTIFACTS.ALL_NAMES:
            shutil.copyfile(HERE / name, bundle / name)
        return temporary, bundle, bundle / CATALOG.CATALOG.name

    def read(self, path: Path):
        return json.loads(path.read_text(encoding="utf-8"))

    def write(self, path: Path, value) -> None:
        path.write_text(CATALOG.ARTIFACTS.serialized(value), encoding="utf-8")

    def rehash(self, path: Path, field: str):
        value = self.read(path)
        body = {key: item for key, item in value.items() if key != field}
        value[field] = hashlib.sha256(CATALOG.canonical(body)).hexdigest()
        self.write(path, value)
        return value

    def bind(self, root: Path, child: Path) -> None:
        root_value, child_value = self.read(root), self.read(child)
        for receipt in root_value["artifact_receipts"]:
            if receipt["file"] == child.name:
                receipt["artifact_sha256"] = child_value["artifact_sha256"]
                receipt["serialized_sha256"] = hashlib.sha256(child.read_bytes()).hexdigest()
        body = {key: item for key, item in root_value.items() if key != "classification_sha256"}
        root_value["classification_sha256"] = hashlib.sha256(CATALOG.canonical(body)).hexdigest()
        self.write(root, root_value)

    def reject_child(self, filename: str, edit, bind: bool = False) -> None:
        temporary, bundle, root = self.copied()
        with temporary:
            child = bundle / filename
            value = self.read(child)
            edit(value)
            self.write(child, value)
            self.rehash(child, "artifact_sha256")
            if bind:
                self.bind(root, child)
            with self.assertRaises(CATALOG.ClassificationError):
                CATALOG.validate(LIVE_ROOT, root)

    def test_closed_routes_baseline_exclusions_and_no_claim_state(self) -> None:
        value = CATALOG.validate(LIVE_ROOT)
        rows = value["command_families"] + value["non_command_families"]
        self.assertEqual(len(value["command_families"]), 28)
        self.assertEqual([row["source_order"] for row in value["command_families"]], list(range(1, 29)))
        self.assertEqual({row["route"] for row in rows}, {row["id"] for row in value["routes"]})
        excluded = [item for row in value["command_families"] for item in row.get("excluded_baseline_command_ids", [])]
        self.assertEqual(set(excluded), set(value["baseline"]["direct_creation_command_ids"]))
        self.assertEqual(value["index_omission_crosscheck"]["physical_page_range"], [801, 851])
        self.assertEqual(value["index_omission_crosscheck"]["checked_page_count"], 51)
        self.assertFalse(any(value["claims"].values()))
        self.assertEqual((value["cts_executions"], value["matrix_row_count"]), (0, 0))

    def test_missing_extra_tampered_rehashed_and_reordered_fragments_fail(self) -> None:
        temporary, bundle, root = self.copied()
        with temporary:
            (bundle / "opengl_command_domain_routes.json").unlink()
            with self.assertRaises(CATALOG.ClassificationError):
                CATALOG.validate(LIVE_ROOT, root)
        temporary, bundle, root = self.copied()
        with temporary:
            shutil.copyfile(bundle / "opengl_command_domain_routes.json", bundle / "opengl_command_domain_extra.json")
            with self.assertRaises(CATALOG.ClassificationError):
                CATALOG.validate(LIVE_ROOT, root)
        target = "opengl_command_domain_core_sync_families.json"
        self.reject_child(target, lambda value: value["command_families"][0].update(route="catchall"))
        self.reject_child(target, lambda value: value["command_families"].reverse(), bind=True)

    def test_rehashed_fragment_and_receipt_reordering_fail(self) -> None:
        target = "opengl_command_domain_resource_vertex_families.json"
        self.reject_child(target, lambda value: value["command_families"].pop())
        temporary, _, root = self.copied()
        with temporary:
            value = self.read(root)
            value["artifact_receipts"].reverse()
            self.write(root, value)
            self.rehash(root, "classification_sha256")
            with self.assertRaises(CATALOG.ClassificationError):
                CATALOG.validate(LIVE_ROOT, root)

    def test_anchor_promotion_type_alias_and_baseline_reinclusion_fail(self) -> None:
        target = "opengl_command_domain_core_sync_families.json"
        self.reject_child(target, lambda value: value["command_families"][0]["anchor"].update(physical_page=34), bind=True)
        self.reject_child(target, lambda value: value["command_families"][1].update(status="supported"), bind=True)
        self.reject_child(target, lambda value: value["command_families"][3].pop("excluded_baseline_command_ids"), bind=True)
        self.reject_child(target, lambda value: value["command_families"][3]["excluded_baseline_command_anchors"][0].update(physical_page=67), bind=True)
        temporary, _, root = self.copied()
        with temporary:
            value = self.read(root)
            value["matrix_row_count"] = True
            self.write(root, value)
            self.rehash(root, "classification_sha256")
            with self.assertRaises(CATALOG.ClassificationError):
                CATALOG.validate(LIVE_ROOT, root)

    def test_formal_anchor_and_full_index_crosscheck_are_active(self) -> None:
        original = CATALOG.RULES.page_text
        with patch.object(CATALOG.RULES, "page_text", side_effect=lambda raw, page: "" if page == 33 else original(raw, page)):
            with self.assertRaisesRegex(CATALOG.ClassificationError, "declaration anchor"):
                CATALOG.rendered_bundle(LIVE_ROOT)
        _, _, _, raw = CATALOG.source_input(LIVE_ROOT)
        short_index = types.SimpleNamespace(returncode=0, stdout=("x\f" * 50).encode("utf-8"))
        with patch.object(CATALOG.INDEX.subprocess, "run", return_value=short_index):
            with self.assertRaisesRegex(CATALOG.ClassificationError, "index crosscheck"):
                CATALOG.INDEX.verify(raw, CATALOG.RULES, CATALOG.CROSSCHECKS, CATALOG.reject)

    def test_unknown_index_witness_and_wrong_baseline_owner_fail_on_rerender(self) -> None:
        _, _, _, raw = CATALOG.source_input(LIVE_ROOT)
        command = ["pdftotext", "-f", "801", "-l", "851", "-layout", "-", "-"]
        actual = CATALOG.INDEX.subprocess.run(command, input=raw, capture_output=True, check=False)
        synthetic = types.SimpleNamespace(returncode=0, stdout=actual.stdout.replace(b"\f", b"\nCreateSynthetic, 999\f", 1))
        with patch.object(CATALOG.INDEX.subprocess, "run", return_value=synthetic):
            with self.assertRaisesRegex(CATALOG.ClassificationError, "unknown, missing, or unaccounted"):
                CATALOG.INDEX.verify(raw, CATALOG.RULES, CATALOG.CROSSCHECKS, CATALOG.reject)
        wrong = dict(CATALOG.RULES.BASELINE_OWNER, **{"command:glCreateBuffers": "program-pipeline"})
        with patch.object(CATALOG.RULES, "BASELINE_OWNER", wrong):
            with self.assertRaisesRegex(CATALOG.ClassificationError, "hard-coded baseline owner"):
                CATALOG.rendered_bundle(LIVE_ROOT)

    def test_private_load_and_cli_remain_isolated(self) -> None:
        code = "\n".join((
            "import importlib.util,sys,types", "from pathlib import Path",
            "sys.modules['opengl_command_domain_rules']=types.ModuleType('opengl_command_domain_rules')",
            f"spec=importlib.util.spec_from_file_location('probe',{str(PROBE)!r})",
            "module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)",
            f"assert module.validate(Path({str(LIVE_ROOT)!r}))['profile']=='opengl-4.6-core'",
        ))
        environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
        isolated = subprocess.run([sys.executable, "-c", code], capture_output=True, text=True, env=environment)
        self.assertEqual(isolated.returncode, 0, isolated.stderr)
        passed = subprocess.run([sys.executable, str(PROBE), "--cache-root", str(LIVE_ROOT)],
                                 capture_output=True, text=True, env=environment)
        self.assertEqual((passed.returncode, passed.stdout.strip()), (0, "PASS: 28 closed command families; matrix-incomplete"))


if __name__ == "__main__":
    unittest.main()
