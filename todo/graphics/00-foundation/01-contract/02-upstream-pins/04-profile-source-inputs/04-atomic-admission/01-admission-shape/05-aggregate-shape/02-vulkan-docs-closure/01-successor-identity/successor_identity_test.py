"""Hermetic hostile checks for the isolated successor identity fixture."""
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
FIXTURE = HERE / "successor_identity.fixture.json"
def module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    value = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = value
    spec.loader.exec_module(value)
    return value

CONTRACT = module("f024_successor_identity", HERE / "successor_identity_contract.py")

class SuccessorIdentityTests(unittest.TestCase):
    def copied(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory()
        target = Path(temporary.name) / "fixture.json"
        shutil.copyfile(FIXTURE, target)
        return temporary, target

    def change(self, path: Path, edit) -> None:
        value = json.loads(path.read_text(encoding="utf-8"))
        edit(value)
        path.write_text(json.dumps(value), encoding="utf-8")

    def rejected(self, edit, message: str) -> None:
        temporary, path = self.copied()
        with temporary:
            self.change(path, edit)
            with self.assertRaisesRegex(CONTRACT.IdentityError, message):
                CONTRACT.validate(path)

    def test_fixture_is_only_a_validated_unadmitted_shape_and_cli(self) -> None:
        result = CONTRACT.validate()
        self.assertEqual(result.member_ids, ("fixture-root", "fixture-registry", "fixture-generated"))
        self.assertEqual(result.generated_ids, ("fixture-generated",))
        state = (result.state, result.admitted, result.cutover_ready)
        self.assertEqual(state, ("fixture-validated", False, False))
        command = subprocess.run(
            [sys.executable, str(HERE / "successor_identity_contract.py"), str(FIXTURE)],
            capture_output=True, text=True,
            env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"),
        )
        self.assertEqual(command.returncode, 0, command.stderr)
        expected = "FIXTURE: fixture-validated, 3 members, 1 generated, 0 cutover-ready"
        self.assertEqual(command.stdout.strip(), expected)

    def test_raw_members_retain_f02_policy_and_successor_cache_boundary(self) -> None:
        bad_url = "https://raw.githubusercontent.com/example/fixture/main/root.adoc"
        self.rejected(lambda value: value["members"][0].update(immutable_url=bad_url), "URL policy")
        self.rejected(
            lambda value: value["members"][0].update(bytes=8 * 1024 * 1024 + 1),
            "byte limit",
        )
        self.rejected(
            lambda value: value["members"][0].update(
                local_cache="webboxvm-graphics/f02/fixture-root/x.source"
            ),
            "successor namespace",
        )

    def test_generated_members_cannot_take_raw_identity_or_missing_producers(self) -> None:
        self.rejected(
            lambda value: value["members"][2].update(immutable_url="https://example.invalid/fake"),
            "invalid schema",
        )
        self.rejected(
            lambda value: value["members"][2].update(producer_member_ids=["absent"]),
            "not earlier",
        )
        self.rejected(lambda value: value["members"][2].update(generation_id="absent"), "no generation")

    def test_generation_recipe_configuration_toolchain_and_output_tree_are_bound(self) -> None:
        self.rejected(
            lambda value: value["generations"][0]["recipe"].update(workdir="../escape"),
            "safe relative",
        )
        self.rejected(
            lambda value: value["generations"][0]["recipe"].update(argv=["../escape"]),
            "unsafe argv",
        )
        self.rejected(
            lambda value: value["generations"][0]["configuration"]["attributes"].reverse(),
            "not canonical",
        )
        self.rejected(
            lambda value: value["generations"][0]["configuration"]["attributes"].append(
                {"name": "VK_VERSION_1_0", "value": "other"}
            ),
            "not canonical",
        )
        self.rejected(
            lambda value: value["generations"][0]["toolchain"].update(sha256="0" * 64),
            "toolchain",
        )
        self.rejected(
            lambda value: value["generations"][0].update(output_tree_sha256="4" * 64),
            "deterministic",
        )

    def test_member_order_and_uniqueness_cannot_be_repaired_by_sorting(self) -> None:
        self.rejected(
            lambda value: value["members"].insert(0, value["members"].pop(1)),
            "root must",
        )
        self.rejected(
            lambda value: value["members"][2].update(selector="root.adoc"),
            "repeats a member selector",
        )
        self.rejected(
            lambda value: value["generations"][0].update(
                output_member_ids=["fixture-generated", "fixture-generated"]
            ),
            "duplicate values",
        )

    def test_root_and_typed_fixture_scope_are_required_but_not_factual_docs_scope(self) -> None:
        self.rejected(lambda value: value.update(root_member_id="fixture-registry"), "root must")
        self.rejected(
            lambda value: value["scope"].update(
                ordered_member_ids=["fixture-registry", "fixture-root", "fixture-generated"]
            ),
            "ordered member",
        )
        self.rejected(lambda value: value["scope"]["excluded_members"].pop("wsi"), "typed exclusions")
        self.rejected(
            lambda value: value["scope"]["excluded_members"].update(wsi=["x", "x"]),
            "unique strings",
        )
        self.rejected(lambda value: value["scope"].update(evidence_mode="actual-docs"), "evidence mode")

    def test_determinism_requires_two_equal_clean_runs(self) -> None:
        self.rejected(
            lambda value: value["generations"][0]["clean_run_tree_sha256s"].__setitem__(1, "5" * 64),
            "not deterministic",
        )

    def test_predecessor_adapter_is_read_only_and_duplicate_json_cannot_hide_state(self) -> None:
        temporary, fixture = self.copied()
        with temporary:
            audit = Path(temporary.name) / "audit.json"
            shutil.copyfile(CONTRACT.PREDECESSOR_PATHS["audit"], audit)
            audit.write_bytes(audit.read_bytes() + b"\n")
            paths = dict(CONTRACT.PREDECESSOR_PATHS, audit=audit)
            with self.assertRaisesRegex(CONTRACT.IdentityError, "stale or mixed"):
                CONTRACT.validate(fixture, paths)
        self.rejected(
            lambda value: value["predecessor_adapter"].update(cutover_id="forbidden"),
            "invalid schema",
        )
        self.rejected(lambda value: value.update(schema=True), "root schema")
        temporary, fixture = self.copied()
        with temporary:
            text = fixture.read_text(encoding="utf-8").replace(
                '"status": "fixture-only-unadmitted",',
                '"status": "admitted", "status": "fixture-only-unadmitted",',
                1,
            )
            fixture.write_text(text, encoding="utf-8")
            with self.assertRaisesRegex(CONTRACT.IdentityError, "duplicate JSON"):
                CONTRACT.validate(fixture)
        with self.assertRaises(TypeError):
            CONTRACT.FixtureClosure(CONTRACT.validate().cache_plan, admitted=True)

if __name__ == "__main__":
    unittest.main()
