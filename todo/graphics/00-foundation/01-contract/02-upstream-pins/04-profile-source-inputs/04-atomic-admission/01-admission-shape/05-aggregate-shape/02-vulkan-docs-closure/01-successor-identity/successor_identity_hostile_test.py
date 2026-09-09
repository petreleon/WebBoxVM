"""Resealed hostile cases that must still fail the successor fixture contract."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import shutil
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


CONTRACT = module("f024_successor_identity_hostile", HERE / "successor_identity_contract.py")


class ResealedHostileTests(unittest.TestCase):
    def copied(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory()
        path = Path(temporary.name) / "fixture.json"
        shutil.copyfile(FIXTURE, path)
        return temporary, path

    @staticmethod
    def digest(value: object) -> str:
        raw = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
        return hashlib.sha256(raw).hexdigest()

    def seal(self, value: dict[str, object]) -> None:
        value["closure_sha256"] = self.digest({
            key: item for key, item in value.items() if key != "closure_sha256"
        })

    def scope_seal(self, value: dict[str, object]) -> None:
        scope = value["scope"]
        self.assertIsInstance(scope, dict)
        scope["scope_sha256"] = self.digest({
            key: item for key, item in scope.items() if key != "scope_sha256"
        })

    def fixture(self, path: Path) -> dict[str, object]:
        return json.loads(path.read_text(encoding="utf-8"))

    def save(self, path: Path, value: dict[str, object]) -> None:
        path.write_text(json.dumps(value), encoding="utf-8")

    def resealed_rejection(self, edit, message: str, scope: bool = False) -> None:
        temporary, path = self.copied()
        with temporary:
            value = self.fixture(path)
            edit(value)
            if scope:
                self.scope_seal(value)
            self.seal(value)
            self.save(path, value)
            with self.assertRaisesRegex(CONTRACT.IdentityError, message):
                CONTRACT.validate(path)

    def bind(self, value: dict[str, object], paths: dict[str, Path]) -> None:
        adapter = value["predecessor_adapter"]
        self.assertIsInstance(adapter, dict)
        for field, name in (
            ("predecessor_rules_sha256", "rules"),
            ("predecessor_audit_sha256", "audit"),
            ("predecessor_boundary_sha256", "boundary"),
        ):
            adapter[field] = hashlib.sha256(paths[name].read_bytes()).hexdigest()
        self.seal(value)

    def test_resealed_selector_scope_and_logical_id_forgery_fail(self) -> None:
        self.resealed_rejection(
            lambda value: value["members"][0].update(selector="unrelated.adoc"),
            "does not name",
        )
        self.resealed_rejection(
            lambda value: value["members"][2].update(selector="."),
            "non-root relative file",
        )
        self.resealed_rejection(
            lambda value: value["members"][2].update(selector="\x00"),
            "safe relative path",
        )
        self.resealed_rejection(
            lambda value: value["members"][2].update(selector="renamed/specattribs.adoc"),
            "output tree is stale",
        )
        self.resealed_rejection(
            lambda value: value["scope"]["excluded_members"].update(wsi=["../escape"]),
            "safe relative path",
            scope=True,
        )
        self.resealed_rejection(
            lambda value: value.update(required_input_id="."), "invalid id"
        )
        self.resealed_rejection(
            lambda value: value["members"][2].pop("license"), "invalid schema"
        )

    def test_resealed_predecessor_rules_audit_and_boundary_must_remain_semantic(self) -> None:
        temporary, path = self.copied()
        with temporary:
            rules = Path(temporary.name) / "rules.json"
            rules.write_text('{"schema":1,"phase":"fabricated"}', encoding="utf-8")
            paths = dict(CONTRACT.PREDECESSOR_PATHS, rules=rules)
            value = self.fixture(path)
            self.bind(value, paths)
            self.save(path, value)
            with self.assertRaisesRegex(CONTRACT.IdentityError, "transition-design"):
                CONTRACT.validate(path, paths)
        temporary, path = self.copied()
        with temporary:
            rules = Path(temporary.name) / "rules.json"
            shutil.copyfile(CONTRACT.PREDECESSOR_PATHS["rules"], rules)
            source = json.loads(rules.read_text(encoding="utf-8"))
            source["atomic_cutover"]["mixed_phase_records"] = "allowed"
            rules.write_text(json.dumps(source), encoding="utf-8")
            paths = dict(CONTRACT.PREDECESSOR_PATHS, rules=rules)
            value = self.fixture(path)
            self.bind(value, paths)
            self.save(path, value)
            with self.assertRaisesRegex(CONTRACT.IdentityError, "exact V1"):
                CONTRACT.validate(path, paths)
        for file_name, edit, message in (
            ("audit", lambda value: value["candidates"][0].update(coverage="direct"), "predecessor audit"),
            ("boundary", lambda value: value["boundaries"][0].update(state="admitted"), "predecessor boundary"),
        ):
            temporary, path = self.copied()
            with temporary:
                predecessor = Path(temporary.name) / f"{file_name}.json"
                shutil.copyfile(CONTRACT.PREDECESSOR_PATHS[file_name], predecessor)
                source = json.loads(predecessor.read_text(encoding="utf-8"))
                edit(source)
                predecessor.write_text(json.dumps(source), encoding="utf-8")
                paths = dict(CONTRACT.PREDECESSOR_PATHS, **{file_name: predecessor})
                value = self.fixture(path)
                self.bind(value, paths)
                self.save(path, value)
                with self.assertRaisesRegex(CONTRACT.IdentityError, message):
                    CONTRACT.validate(path, paths)

    def test_recipe_retains_repeated_arguments_as_ordered_identity(self) -> None:
        temporary, path = self.copied()
        with temporary:
            value = self.fixture(path)
            recipe = value["generations"][0]["recipe"]
            self.assertIsInstance(recipe, dict)
            recipe["argv"] = ["python3", "--include", "core", "--include", "core"]
            recipe["sha256"] = self.digest({key: item for key, item in recipe.items() if key != "sha256"})
            self.seal(value)
            self.save(path, value)
            result = CONTRACT.validate(path)
            self.assertEqual(result.state, "fixture-validated")
            plan = result.cache_plan
            self.assertEqual(plan.members[2].license, "fixture-generated-license")
            self.assertEqual(plan.generations[0].recipe_digest, recipe["sha256"])
            self.assertEqual(plan.scope.scope_digest, value["scope"]["scope_sha256"])
            self.assertEqual(plan.predecessor.decision, "rejected")


if __name__ == "__main__":
    unittest.main()
