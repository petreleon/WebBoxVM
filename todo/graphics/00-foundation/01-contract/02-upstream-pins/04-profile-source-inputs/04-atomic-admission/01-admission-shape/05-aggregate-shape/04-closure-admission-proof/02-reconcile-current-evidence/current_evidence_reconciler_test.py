#!/usr/bin/env python3
"""Hermetic positive and hostile tests for current aggregate evidence reconciliation."""

from __future__ import annotations

from dataclasses import replace
import hashlib
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


def module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    value = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = value
    spec.loader.exec_module(value)
    return value


RECON = module("f024_current_evidence_reconciler", HERE / "current_evidence_reconciler.py")


class ReconcilerTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def document(self, path: Path) -> dict[str, object]:
        return json.loads(path.read_text(encoding="utf-8"))

    def write(self, path: Path, value: dict[str, object]) -> None:
        path.write_text(json.dumps(value, sort_keys=True), encoding="utf-8")

    def hash(self, path: Path) -> str:
        return hashlib.sha256(path.read_bytes()).hexdigest()

    def checked(self, **kwargs):
        return RECON.validate(**kwargs)

    def rejected(self, **kwargs) -> None:
        with self.assertRaises(RECON.ReconcileError):
            self.checked(**kwargs)

    def cache_digest(self, value: dict[str, object]) -> str:
        body = {key: item for key, item in value.items() if key != "receipt_sha256"}
        return hashlib.sha256(json.dumps(body, sort_keys=True, separators=(",", ":")).encode()).hexdigest()

    def seal(self, index: int, value: dict[str, object]) -> None:
        field, digest = (("handoff_sha256", RECON.V2.digest), ("identity_sha256", RECON.V2.capture.source.identity.digest),
                         ("plan_sha256", RECON.V2.capture.source.tree.digest), ("ledger_sha256", RECON.V2.capture.source.ledger.digest),
                         ("receipt_sha256", self.cache_digest), ("receipt_sha256", RECON.V2.capture.digest),
                         ("taxonomy_sha256", RECON.V2.taxonomy.digest))[index]
        value[field] = digest(value)

    def v2_case(self, index: int, edit) -> None:
        folder = Path(self.temporary.name) / f"v2-{index}"
        folder.mkdir()
        paths = tuple(folder / source.name for source in RECON.V2_PATHS)
        for source, target in zip(RECON.V2_PATHS, paths):
            shutil.copyfile(source, target)
        value = self.document(paths[index])
        edit(value)
        self.seal(index, value)
        self.write(paths[index], value)
        with self.assertRaisesRegex(RECON.ReconcileError, "^V2 evidence failed:"):
            RECON.validate(v2_paths=paths)

    def test_current_evidence_is_exactly_read_only_and_blocked(self) -> None:
        inputs = (RECON.PROOF.CONTRACT, RECON.PROOF.REQUIREMENTS, *RECON.V1_PATHS,
                  *RECON.PRE.TRANSITION.MAP.AUDITS.values(), *RECON.V2_PATHS,
                  RECON.PROOF.REQUIREMENTS.parent / "profile_scope.json")
        before = {path: self.hash(path) for path in inputs}
        result = self.checked()
        self.assertEqual(before, {path: self.hash(path) for path in inputs})
        self.assertEqual(tuple(row[:3] for row in result.roles), RECON.PROOF.CANONICAL)
        self.assertEqual((result.blockers, result.first_blocker), (RECON.PRE.BLOCKERS, RECON.PRE.BLOCKERS[0]))
        self.assertEqual(tuple(value for _, value in result.v1_digests), RECON.V1_LOCK)
        self.assertEqual(tuple(value for _, value in result.v2_digests), RECON.V2_LOCK)
        self.assertEqual(result.v1_v2_bridge, ("external/vulkancts/mustpass/main/vk-default.txt",
                                                 "f6a29701220f34dd1407513bfe80d74ca7b392ce",
                                                 "b689703bdc65a04764db3b9a8f6fe872b3fe94d0df68d78f6da6e5a06cfa9ed4"))
        self.assertEqual((result.state, result.admission_eligible, result.inventory_ready,
                          result.fresh_cache_ready, result.cutover_ready, result.f03_ready),
                         ("blocked", False, False, False, False, False))
        self.assertEqual((result.selector_scope, result.docs_generated_artifacts, result.v2_state),
                         (RECON.V2_HEADER[2], RECON.V2_HEADER[4], RECON.V2_STATE))
        command = subprocess.run([sys.executable, "-B", str(HERE / "current_evidence_reconciler.py")],
                                 capture_output=True, text=True,
                                 env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
        self.assertEqual(command.returncode, 0, command.stderr)
        self.assertEqual(command.stdout.strip(), "BLOCKED: 6 roles, first gles-cts-manifest requires-multifile-core-selector-closure")
        self.assertEqual(before, {path: self.hash(path) for path in inputs})

    def test_contract_and_v1_stale_eroded_or_falsely_admitted_inputs_cannot_reconcile(self) -> None:
        root = Path(self.temporary.name)
        contract = root / "contract.json"
        shutil.copyfile(RECON.PROOF.CONTRACT, contract)
        value = self.document(contract)
        value["roles"].reverse()
        value["contract_sha256"] = RECON.PROOF.digest(value)
        self.write(contract, value)
        self.rejected(contract_path=contract)
        audits = dict(RECON.PRE.TRANSITION.MAP.AUDITS)
        audit = root / "audit.json"
        shutil.copyfile(audits["gles-3.2"], audit)
        audit.write_text(audit.read_text(encoding="utf-8") + "\n", encoding="utf-8")
        audits["gles-3.2"] = audit
        RECON.PRE.validate(*RECON.V1_PATHS, audits)
        with self.assertRaisesRegex(RECON.ReconcileError, "immutable predecessor bundle"):
            self.checked(audits=audits)
        self.rejected(audits=dict(RECON.PRE.TRANSITION.MAP.AUDITS,
                                  **{"gles-3.2": RECON.PRE.TRANSITION.MAP.AUDITS["opengl-4.6-core"]}))
        source_map = root / "source-map.json"
        shutil.copyfile(RECON.PRE.TRANSITION.MAP.SOURCE_MAP, source_map)
        value = self.document(source_map)
        value["shapes"][4]["state"] = "candidate-accepted"
        self.write(source_map, value)
        self.rejected(v1_paths=(RECON.V1_PATHS[0], source_map, RECON.V1_PATHS[2]))
        value["shapes"][4]["state"] = "unadmitted"
        value["shapes"][3]["core_members"].pop()
        self.write(source_map, value)
        self.rejected(v1_paths=(RECON.V1_PATHS[0], source_map, RECON.V1_PATHS[2]))
        boundary = root / "boundaries.json"
        shutil.copyfile(RECON.V1_PATHS[2], boundary)
        value = self.document(boundary)
        value["boundaries"][1]["root_fallback"] = "allowed"
        self.write(boundary, value)
        self.rejected(v1_paths=(RECON.V1_PATHS[0], RECON.V1_PATHS[1], boundary))

    def test_all_v2_inputs_and_v1_substitution_cannot_reconcile(self) -> None:
        cases = ((0, lambda value: value.__setitem__("admitted", True)),
                 (1, lambda value: value.__setitem__("tag_name", "vulkan-cts-1.4.0")),
                 (2, lambda value: value["members"][0].__setitem__("blob_sha1", "f" * 40)),
                 (3, lambda value: value["members"][0].__setitem__("sha256", "f" * 64)),
                 (4, lambda value: value.__setitem__("admitted", True)),
                 (5, lambda value: value.__setitem__("admitted", True)),
                 (6, lambda value: value["rules"][0].__setitem__("category", "core")))
        for index, edit in cases:
            self.v2_case(index, edit)
        paths = list(RECON.V2_PATHS)
        paths[1] = RECON.SHAPE.parents[2] / "03-vulkan-input-audit/mustpass_references.json"
        self.rejected(v2_paths=tuple(paths))

    def test_bridge_and_incomplete_path_sets_cannot_reconcile(self) -> None:
        pre = RECON.PRE.validate()
        root = RECON.current_v2(RECON.V2_PATHS)[1]
        with self.assertRaisesRegex(RECON.ReconcileError, "selector, revision, or root identity"):
            RECON.bridge(RECON.V1_PATHS[1], RECON.PRE.TRANSITION.MAP.AUDITS["vulkan-1.4-core"],
                         replace(root, peeled_commit="f" * 40), pre.vulkan[-1].root_sha256)
        self.rejected(v1_paths=RECON.V1_PATHS[:2])
        self.rejected(v2_paths=RECON.V2_PATHS[:-1])


if __name__ == "__main__":
    unittest.main(verbosity=2)
