#!/usr/bin/env python3
"""Hermetic positive and hostile checks for the VCTS upstream-selector blocker."""
import hashlib, importlib.util, json, os, subprocess, sys, tempfile, unittest
from pathlib import Path
from unittest import mock

HERE = Path(__file__).resolve().parent
TEMP_ROOT = Path("/private/tmp") if Path("/private/tmp").is_dir() else Path(tempfile.gettempdir()).resolve()
def module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None: raise RuntimeError(f"cannot load {path}")
    value = importlib.util.module_from_spec(spec); sys.modules[spec.name] = value; spec.loader.exec_module(value)
    return value
GATE = module("f025_vcts_selector_recheck", HERE / "upstream_selector_recheck.py")

class RecheckTests(unittest.TestCase):
    def setUp(self) -> None: self.temporary = tempfile.TemporaryDirectory(dir=TEMP_ROOT)
    def tearDown(self) -> None: self.temporary.cleanup()
    def copy(self, name: str) -> Path:
        path = Path(self.temporary.name) / name; path.write_bytes(GATE.RECORD.read_bytes()); return path
    def value(self, path: Path) -> dict[str, object]: return json.loads(path.read_text(encoding="utf-8"))
    def seal(self, value: dict[str, object]) -> None: value["recheck_sha256"] = GATE.digest(value)
    def write(self, path: Path, value: dict[str, object]) -> None:
        path.write_text(json.dumps(value, sort_keys=True), encoding="utf-8")
    def reject(self, edit) -> None:
        path = self.copy("mutated.json"); value = self.value(path); edit(value); self.seal(value); self.write(path, value)
        with self.assertRaisesRegex(GATE.RecheckError, "exact blocked upstream observation"): GATE.validate(path)

    def test_committed_observation_is_offline_read_only_and_blocked(self) -> None:
        watched = tuple(GATE.INPUTS.values())
        before = {path: hashlib.sha256(path.read_bytes()).hexdigest() for path in watched}
        value = GATE.validate()
        self.assertEqual(value, GATE.build())
        self.assertEqual(value["status"], "blocked-upstream-authority-missing")
        self.assertFalse(value["observation"]["explicit_vulkan_14_core_manifest_present"])
        self.assertEqual(value["effects"], {name: False for name in GATE.EFFECTS})
        self.assertTrue(all(item is False for item in value["prohibitions"].values()))
        source = (HERE / "upstream_selector_recheck.py").read_text(encoding="utf-8")
        self.assertFalse(any(name in source for name in ("urllib", "requests", "subprocess", "socket")))
        result = subprocess.run([sys.executable, "-B", str(HERE / "upstream_selector_recheck.py")], capture_output=True,
                                text=True, env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), f"BLOCKED: {GATE.BLOCKER} {value['recheck_sha256']}")
        self.assertEqual(before, {path: hashlib.sha256(path.read_bytes()).hexdigest() for path in watched})

    def test_resealed_observation_scope_and_blocker_mutations_are_rejected(self) -> None:
        cases = (
            lambda value: value["observation"].__setitem__("latest_vulkan_14_tag_at_observation", "vulkan-cts-1.4.7.0"),
            lambda value: value["observation"]["root_directory_files"].append("vulkan-1.4-core.txt"),
            lambda value: value["observation"]["vk_default"].__setitem__("includes_wsi", False),
            lambda value: value["observation"]["vk_default"].__setitem__("scope", "vulkan-1.4-core"),
            lambda value: value["observation"].__setitem__("explicit_vulkan_14_core_manifest_present", True),
            lambda value: value["blocker"].__setitem__("replaces_historical_global_blocker", True),
            lambda value: value["blocker"].__setitem__("id", "vcts-vk-default-compound-oversize-core-scope-unadmitted"),
        )
        for edit in cases:
            with self.subTest(edit=edit): self.reject(edit)

    def test_resealed_promotions_and_predecessor_identity_changes_are_rejected(self) -> None:
        promotions = ("local_core_selector_allowed", "taxonomy_as_conformance_allowed", "root_only_claim_allowed",
                      "opaque_member_splitting_allowed", "vcts_as_docs_allowed", "tag_alone_sufficient")
        for key in promotions:
            with self.subTest(key=key): self.reject(lambda value, key=key: value["prohibitions"].__setitem__(key, True))
        self.reject(lambda value: value["effects"].__setitem__("admitted", 0))
        self.reject(lambda value: value["predecessors"]["v2_handoff"].__setitem__("document_sha256", "0" * 64))
        self.reject(lambda value: value["predecessors"]["active_f03"].__setitem__("changed", True))
        baseline = GATE.V2.validate
        with mock.patch.object(GATE.V2, "validate", side_effect=lambda: {**baseline(), "member_count": 99}):
            with self.assertRaisesRegex(GATE.RecheckError, "exact blocked VCTS state"): GATE.build()
        raw_hashes = GATE.hashes
        with mock.patch.object(GATE, "hashes", side_effect=lambda: {**raw_hashes(), "active_f02_inventory_lock": "0" * 64}):
            with self.assertRaisesRegex(GATE.RecheckError, "stale raw identities"): GATE.build()

    def test_parser_and_filesystem_hostiles_are_rejected(self) -> None:
        path = self.copy("selfhash.json"); value = self.value(path); value["recheck_sha256"] = "0" * 64; self.write(path, value)
        with self.assertRaisesRegex(GATE.RecheckError, "self-hash"): GATE.validate(path)
        root = Path(self.temporary.name)
        duplicate = root / "duplicate.json"; duplicate.write_text('{"schema":1,"schema":1}', encoding="utf-8")
        with self.assertRaisesRegex(GATE.RecheckError, "duplicate"): GATE.validate(duplicate)
        nonfinite = root / "nonfinite.json"; nonfinite.write_text('{"schema":NaN}', encoding="utf-8")
        with self.assertRaisesRegex(GATE.RecheckError, "non-finite"): GATE.validate(nonfinite)
        oversized = root / "oversized.json"; oversized.write_bytes(b"x" * (GATE.MAX_BYTES + 1))
        with self.assertRaisesRegex(GATE.RecheckError, "bounded size"): GATE.validate(oversized)
        fifo = root / "recheck.fifo"; os.mkfifo(fifo)
        with self.assertRaisesRegex(GATE.RecheckError, "regular file"): GATE.validate(fifo)
        link = root / "recheck-link.json"; link.symlink_to(GATE.RECORD)
        with self.assertRaisesRegex(GATE.RecheckError, "safely opened"): GATE.validate(link)
        nested = root / "nested"; nested.mkdir(); (nested / "valid.json").write_bytes(GATE.RECORD.read_bytes())
        redirect = root / "redirect"; redirect.symlink_to(nested, target_is_directory=True)
        with self.assertRaisesRegex(GATE.RecheckError, "safely opened"): GATE.validate(redirect / "valid.json")

if __name__ == "__main__": unittest.main(verbosity=2)
