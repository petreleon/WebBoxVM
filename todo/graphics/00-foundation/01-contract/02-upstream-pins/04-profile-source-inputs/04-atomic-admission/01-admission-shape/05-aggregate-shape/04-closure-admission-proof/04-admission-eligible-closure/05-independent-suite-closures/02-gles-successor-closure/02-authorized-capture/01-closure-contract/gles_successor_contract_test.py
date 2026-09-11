#!/usr/bin/env python3
"""Focused positive and hostile tests for the GLES successor closure contract."""

import hashlib, importlib.util, json, os, subprocess, sys, tempfile, unittest
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


CONTRACT = module("f025_gles_successor_contract_test", HERE / "gles_successor_contract.py")


class ContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def copy(self, name: str) -> Path:
        path = Path(self.temporary.name) / name
        path.write_bytes(CONTRACT.RECORD.read_bytes())
        return path

    def value(self, path: Path) -> dict[str, object]:
        return json.loads(path.read_text(encoding="utf-8"))

    def write(self, path: Path, value: dict[str, object]) -> None:
        value["contract_sha256"] = CONTRACT.digest(value)
        path.write_text(json.dumps(value, sort_keys=True), encoding="utf-8")

    def reject(self, edit) -> None:
        path = self.copy("mutated.json")
        value = self.value(path)
        edit(value)
        self.write(path, value)
        with self.assertRaisesRegex(CONTRACT.ContractError, "exact unadmitted GLES closure"):
            CONTRACT.validate(path)

    def test_committed_contract_is_read_only_and_design_only(self) -> None:
        watched = (CONTRACT.RECORD, CONTRACT.INTEGRATION.RECORD, CONTRACT.BOUNDARY.CLOSURE,
                   CONTRACT.BOUNDARY.CONFIGURATIONS, CONTRACT.BOUNDARY.CANDIDATES)
        before = {path: hashlib.sha256(path.read_bytes()).hexdigest() for path in watched}
        value = CONTRACT.validate()
        self.assertEqual(value["status"], "design-only-unadmitted")
        self.assertEqual(len(value["closure"]["core_members"]), 4)
        self.assertFalse(value["producer"]["producer_execution_required_for_this_capture"])
        self.assertFalse(value["producer"]["producer_execution_proved"])
        self.assertEqual(value["effects"], {name: False for name in CONTRACT.EFFECTS})
        result = subprocess.run([sys.executable, "-B", str(HERE / "gles_successor_contract.py")],
                                capture_output=True, text=True, env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), f"CONTRACT: design-only-unadmitted {value['contract_sha256']}")
        self.assertEqual(before, {path: hashlib.sha256(path.read_bytes()).hexdigest() for path in watched})

    def test_resealed_identity_scope_cache_and_promotion_are_rejected(self) -> None:
        cases = (
            lambda value: value["source"].__setitem__("revision", "0" * 40),
            lambda value: value["closure"]["core_members"].pop(),
            lambda value: value["closure"]["root"].__setitem__("bytes", 1),
            lambda value: value["source"].__setitem__("cache_template", "webboxvm-graphics/f02-successor/{id}/{sha256}.source"),
            lambda value: value["closure"]["successor_cache_paths"].pop("gles-cts-gles3-khr-main"),
            lambda value: value["producer"].__setitem__("producer_execution_proved", True),
            lambda value: value["producer"].__setitem__("output_attestation_present", True),
            lambda value: value["capture"].__setitem__("active_cache_freshness_proved", True),
            lambda value: value["predecessor"].__setitem__("active_mutation_permitted", True),
            lambda value: value["effects"].__setitem__("admitted", True),
        )
        for edit in cases:
            with self.subTest(edit=edit):
                self.reject(edit)

    def test_self_hash_duplicate_oversize_fifo_and_symlink_are_rejected(self) -> None:
        path = self.copy("selfhash.json")
        value = self.value(path)
        value["contract_sha256"] = "0" * 64
        path.write_text(json.dumps(value), encoding="utf-8")
        with self.assertRaisesRegex(CONTRACT.ContractError, "self-hash"):
            CONTRACT.validate(path)
        root = Path(self.temporary.name)
        duplicate = root / "duplicate.json"
        duplicate.write_text('{"schema":1,"schema":1}', encoding="utf-8")
        with self.assertRaisesRegex(CONTRACT.ContractError, "duplicate"):
            CONTRACT.validate(duplicate)
        oversized = root / "oversized.json"
        oversized.write_bytes(b"x" * (64 * 1024 + 1))
        with self.assertRaisesRegex(CONTRACT.ContractError, "bounded size"):
            CONTRACT.validate(oversized)
        fifo = root / "contract.fifo"
        os.mkfifo(fifo)
        with self.assertRaisesRegex(CONTRACT.ContractError, "regular file"):
            CONTRACT.validate(fifo)
        link = root / "contract-link.json"
        link.symlink_to(CONTRACT.RECORD)
        with self.assertRaisesRegex(CONTRACT.ContractError, "regular file"):
            CONTRACT.validate(link)


if __name__ == "__main__":
    unittest.main(verbosity=2)
