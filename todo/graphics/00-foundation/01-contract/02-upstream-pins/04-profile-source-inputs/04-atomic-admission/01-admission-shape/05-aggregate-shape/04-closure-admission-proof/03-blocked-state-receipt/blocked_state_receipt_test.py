#!/usr/bin/env python3
"""Hermetic positive and hostile tests for the blocked-state receipt."""

from __future__ import annotations

import hashlib
import importlib.util
import json
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


RECEIPT = module("f024_blocked_state_receipt", HERE / "blocked_state_receipt.py")


class ReceiptTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def copy(self, name: str) -> Path:
        path = Path(self.temporary.name) / name
        shutil.copyfile(RECEIPT.RECEIPT, path)
        return path

    def document(self, path: Path) -> dict[str, object]:
        return json.loads(path.read_text(encoding="utf-8"))

    def write(self, path: Path, value: dict[str, object]) -> None:
        path.write_text(json.dumps(value, sort_keys=True), encoding="utf-8")

    def seal(self, value: dict[str, object]) -> None:
        value["receipt_sha256"] = RECEIPT.digest(value)

    def rejected(self, path: Path, **kwargs) -> None:
        with self.assertRaises(RECEIPT.ReceiptError):
            RECEIPT.validate(path, **kwargs)

    def test_committed_receipt_is_read_only_exact_and_blocked(self) -> None:
        manifest = RECEIPT.RECON.PRE.TRANSITION.MANIFEST
        inputs = (RECEIPT.RECON.PROOF.CONTRACT, RECEIPT.RECON.PROOF.REQUIREMENTS, manifest,
                  manifest.parent / "inventory.lock",
                  *sorted(manifest.parent.glob("inputs/*.toml")), *RECEIPT.RECON.V1_PATHS,
                  *RECEIPT.RECON.PRE.TRANSITION.MAP.AUDITS.values(), *RECEIPT.RECON.V2_PATHS,
                  RECEIPT.RECEIPT, RECEIPT.RECON.PROOF.REQUIREMENTS.parent / "profile_scope.json")
        before = {path: hashlib.sha256(path.read_bytes()).hexdigest() for path in inputs}
        expected = RECEIPT.build()
        self.assertTrue(RECEIPT.alike(RECEIPT.validate(), expected))
        command = subprocess.run([sys.executable, "-B", str(HERE / "blocked_state_receipt.py")],
                                 capture_output=True, text=True)
        self.assertEqual(command.returncode, 0, command.stderr)
        self.assertEqual(command.stdout.strip(), f"RECEIPT: blocked 3 blockers {expected['receipt_sha256']}")
        self.assertEqual(before, {path: hashlib.sha256(path.read_bytes()).hexdigest() for path in inputs})
        self.assertEqual((expected["profile"], expected["state"], expected["readiness"], expected["v2_state"]),
                         ("proof-of-correctly-blocked-admission-state", "blocked",
                          {key: False for key in RECEIPT.READINESS}, [False, False, False]))

    def test_schema_duplicate_oversize_and_raw_selfhash_are_rejected(self) -> None:
        path = self.copy("selfhash.json")
        value = self.document(path)
        value["receipt_sha256"] = "0" * 64
        self.write(path, value)
        self.rejected(path)
        path = Path(self.temporary.name) / "duplicate.json"
        path.write_text('{"schema":1,"schema":1}', encoding="utf-8")
        self.rejected(path)
        path = Path(self.temporary.name) / "oversize.json"
        path.write_bytes(b" " * (RECEIPT.MAX_RECEIPT_BYTES + 1))
        self.rejected(path)
        path = self.copy("extra.json")
        value = self.document(path)
        value["extra"] = True
        self.seal(value)
        self.write(path, value)
        self.rejected(path)

    def test_resealed_ready_scope_blocker_and_identity_mutations_are_rejected(self) -> None:
        cases = (lambda value: value["readiness"].__setitem__("admission_eligible", True),
                 lambda value: value["readiness"].__setitem__("f03_ready", 0),
                 lambda value: value.__setitem__("state", "admission_eligible"),
                 lambda value: value["blockers"].reverse(),
                 lambda value: value["first_blocker"].__setitem__(0, "vulkan-14-spec"),
                 lambda value: value.__setitem__("selector_scope", "vulkan-1.4-core"),
                 lambda value: value.__setitem__("docs_generated_artifacts", "admitted"),
                 lambda value: value["v2_state"].__setitem__(0, True),
                 lambda value: value["v1_digests"][0].__setitem__(1, "0" * 64),
                 lambda value: value["v1_v2_bridge"].__setitem__(1, "f" * 40),
                 lambda value: value["v2_digests"][0].__setitem__(1, "0" * 64))
        for index, edit in enumerate(cases):
            path = self.copy(f"resealed-{index}.json")
            value = self.document(path)
            edit(value)
            self.seal(value)
            self.write(path, value)
            self.rejected(path)

    def test_stale_native_v1_and_v2_inputs_cannot_seal_or_validate(self) -> None:
        root = Path(self.temporary.name)
        source_map = root / "source-map.json"
        shutil.copyfile(RECEIPT.RECON.V1_PATHS[1], source_map)
        source_map.write_text(source_map.read_text(encoding="utf-8") + "\n", encoding="utf-8")
        with self.assertRaisesRegex(RECEIPT.ReceiptError, "current evidence failed"):
            RECEIPT.build({"v1_paths": (RECEIPT.RECON.V1_PATHS[0], source_map, RECEIPT.RECON.V1_PATHS[2])})
        handoff = root / "handoff.json"
        shutil.copyfile(RECEIPT.RECON.V2.HANDOFF, handoff)
        value = self.document(handoff)
        value["admitted"] = True
        value["handoff_sha256"] = RECEIPT.RECON.V2.digest(value)
        self.write(handoff, value)
        with self.assertRaisesRegex(RECEIPT.ReceiptError, "current evidence failed"):
            RECEIPT.build({"v2_paths": (handoff, *RECEIPT.RECON.V2_PATHS[1:])})


if __name__ == "__main__":
    unittest.main(verbosity=2)
