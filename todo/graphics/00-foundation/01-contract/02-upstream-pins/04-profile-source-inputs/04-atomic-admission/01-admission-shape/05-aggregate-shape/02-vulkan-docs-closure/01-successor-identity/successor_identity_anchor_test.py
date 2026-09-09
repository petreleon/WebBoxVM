"""Exact-predecessor anchor regressions for the successor-only fixture."""

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


CONTRACT = module("f024_successor_identity_anchor", HERE / "successor_identity_contract.py")


class AnchorRegressionTests(unittest.TestCase):
    def copied(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory()
        fixture = Path(temporary.name) / "fixture.json"
        shutil.copyfile(FIXTURE, fixture)
        return temporary, fixture

    @staticmethod
    def digest(value: object) -> str:
        raw = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
        return hashlib.sha256(raw).hexdigest()

    def seal(self, value: dict[str, object]) -> None:
        value["closure_sha256"] = self.digest({
            key: item for key, item in value.items() if key != "closure_sha256"
        })

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

    def test_resealed_fixture_cannot_claim_a_real_docs_requirement(self) -> None:
        temporary, fixture = self.copied()
        with temporary:
            value = json.loads(fixture.read_text(encoding="utf-8"))
            value.update(profile="vulkan-1.4-core", role="api-limit-format-spec",
                         required_input_id="vulkan-14-spec")
            for member in value["members"]:
                member["local_cache"] = member["local_cache"].replace(
                    "fixture-raw-generated-closure", "vulkan-14-spec"
                )
            self.seal(value)
            fixture.write_text(json.dumps(value), encoding="utf-8")
            with self.assertRaisesRegex(CONTRACT.IdentityError, "synthetic profile"):
                CONTRACT.validate(fixture)

    def test_resealed_boundary_cannot_erase_docs_exclusions(self) -> None:
        temporary, fixture = self.copied()
        with temporary:
            boundary = Path(temporary.name) / "boundary.json"
            shutil.copyfile(CONTRACT.PREDECESSOR_PATHS["boundary"], boundary)
            value = json.loads(boundary.read_text(encoding="utf-8"))
            value["boundaries"][0]["scope_exclusions"] = {
                "wsi": [], "video": [], "extensions": []
            }
            boundary.write_text(json.dumps(value), encoding="utf-8")
            paths = dict(CONTRACT.PREDECESSOR_PATHS, boundary=boundary)
            closure = json.loads(fixture.read_text(encoding="utf-8"))
            self.bind(closure, paths)
            fixture.write_text(json.dumps(closure), encoding="utf-8")
            with self.assertRaisesRegex(CONTRACT.IdentityError, "predecessor boundary"):
                CONTRACT.validate(fixture, paths)

    def test_resealed_audit_cannot_substitute_the_pinned_docs_root(self) -> None:
        temporary, fixture = self.copied()
        with temporary:
            audit = Path(temporary.name) / "audit.json"
            shutil.copyfile(CONTRACT.PREDECESSOR_PATHS["audit"], audit)
            value = json.loads(audit.read_text(encoding="utf-8"))
            value["inventory_sha256"] = "1" * 64
            value["candidates"][0]["entry"]["immutable_url"] = (
                "https://raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/"
                "f84d432d5b8912362f96f581f29bbc4f3c8c7843/README.adoc"
            )
            audit.write_text(json.dumps(value), encoding="utf-8")
            paths = dict(CONTRACT.PREDECESSOR_PATHS, audit=audit)
            closure = json.loads(fixture.read_text(encoding="utf-8"))
            self.bind(closure, paths)
            fixture.write_text(json.dumps(closure), encoding="utf-8")
            with self.assertRaisesRegex(CONTRACT.IdentityError, "predecessor audit"):
                CONTRACT.validate(fixture, paths)


if __name__ == "__main__":
    unittest.main()
