#!/usr/bin/env python3
"""Hermetic positive and hostile tests for the aggregate-proof grammar."""

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


def module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    value = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = value
    spec.loader.exec_module(value)
    return value


CONTRACT = module("f024_admission_proof_contract", HERE / "admission_proof_contract.py")


class ContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        root = Path(self.temporary.name)
        self.contract = root / "contract.json"
        self.requirements = root / "requirements.json"
        shutil.copyfile(CONTRACT.CONTRACT, self.contract)
        shutil.copyfile(CONTRACT.REQUIREMENTS, self.requirements)

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def document(self, path: Path) -> dict[str, object]:
        return json.loads(path.read_text(encoding="utf-8"))

    def write(self, path: Path, value: dict[str, object]) -> None:
        path.write_text(json.dumps(value, sort_keys=True), encoding="utf-8")

    def seal(self, value: dict[str, object]) -> None:
        value["contract_sha256"] = CONTRACT.digest(value)

    def rejected_contract(self, edit, seal: bool = True) -> None:
        value = self.document(self.contract)
        edit(value)
        if seal:
            self.seal(value)
        self.write(self.contract, value)
        with self.assertRaises(CONTRACT.ContractError):
            CONTRACT.validate(self.contract, self.requirements)

    def rejected_requirements(self, edit) -> None:
        value = self.document(self.requirements)
        edit(value)
        self.write(self.requirements, value)
        with self.assertRaises(CONTRACT.ContractError):
            CONTRACT.validate(self.contract, self.requirements)

    def test_committed_contract_freezes_exactly_six_non_admitting_roles(self) -> None:
        value = CONTRACT.validate()
        self.assertEqual(tuple(row[:3] for row in value.roles), CONTRACT.CANONICAL)
        self.assertEqual(tuple(row[3] for row in value.roles), CONTRACT.ENVELOPES)
        self.assertEqual(value.states, tuple((field, False) for field in CONTRACT.STATE_FIELDS))
        command = subprocess.run([sys.executable, str(HERE / "admission_proof_contract.py")],
                                 capture_output=True, text=True,
                                 env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
        self.assertEqual(command.returncode, 0, command.stderr)
        self.assertIn("CONTRACT: 6 roles", command.stdout)

    def test_source_requirements_cannot_reorder_erode_or_change_schema(self) -> None:
        self.rejected_requirements(lambda value: value["requirements"].reverse())
        shutil.copyfile(CONTRACT.REQUIREMENTS, self.requirements)
        self.rejected_requirements(lambda value: value["requirements"][5].update(required_input_id="vulkan-14-spec"))
        shutil.copyfile(CONTRACT.REQUIREMENTS, self.requirements)
        self.rejected_requirements(lambda value: value["requirements"][4].update(related_input_ids=["vulkan-registry"]))
        shutil.copyfile(CONTRACT.REQUIREMENTS, self.requirements)
        self.rejected_requirements(lambda value: value.update(schema=2))

    def test_role_envelopes_cannot_substitute_docs_or_vcts(self) -> None:
        self.rejected_contract(lambda value: value["roles"].pop())
        shutil.copyfile(CONTRACT.CONTRACT, self.contract)
        self.rejected_contract(lambda value: value["roles"].append(dict(value["roles"][5])))
        shutil.copyfile(CONTRACT.CONTRACT, self.contract)
        self.rejected_contract(lambda value: value["roles"][4].update(envelope="v1-vcts-boundary-plus-v2-diagnostic"))
        shutil.copyfile(CONTRACT.CONTRACT, self.contract)
        self.rejected_contract(lambda value: value["roles"][5].update(required_input_id="vulkan-14-spec"))
        shutil.copyfile(CONTRACT.CONTRACT, self.contract)
        self.rejected_contract(lambda value: value["roles"].reverse())

    def test_contract_rejects_false_ready_states_and_extra_fields(self) -> None:
        self.rejected_contract(lambda value: value["states"].update(admission_eligible=True))
        shutil.copyfile(CONTRACT.CONTRACT, self.contract)
        self.rejected_contract(lambda value: value["states"].update(f03_ready=True))
        shutil.copyfile(CONTRACT.CONTRACT, self.contract)
        self.rejected_contract(lambda value: value.update(inventory_sha256="0" * 64))

    def test_contract_rejects_stale_lock_duplicate_keys_and_oversize_json(self) -> None:
        self.rejected_contract(lambda value: value.update(source_requirements_sha256="0" * 64))
        shutil.copyfile(CONTRACT.CONTRACT, self.contract)
        text = self.contract.read_text(encoding="utf-8").replace(
            '"kind": "aggregate-admission-proof-contract",',
            '"kind": "bad", "kind": "aggregate-admission-proof-contract",', 1)
        self.contract.write_text(text, encoding="utf-8")
        with self.assertRaises(CONTRACT.ContractError):
            CONTRACT.validate(self.contract, self.requirements)
        self.contract.write_bytes(b" " * (CONTRACT.MAX_BYTES + 1))
        with self.assertRaises(CONTRACT.ContractError):
            CONTRACT.validate(self.contract, self.requirements)


if __name__ == "__main__":
    unittest.main(verbosity=2)
