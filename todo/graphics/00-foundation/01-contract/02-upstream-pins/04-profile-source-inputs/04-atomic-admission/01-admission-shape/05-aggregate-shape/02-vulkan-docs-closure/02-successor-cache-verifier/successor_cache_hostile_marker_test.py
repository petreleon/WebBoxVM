"""Hostile completion-marker, stale-member, and read-only verification cases."""

from __future__ import annotations

import importlib.util
import json
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


CACHE = module("f024_successor_cache_hostile_marker", HERE / "successor_cache_contract.py")
FIXTURES = module("f024_successor_cache_fixture_hostile_marker", HERE / "successor_cache_fixture.py")
MARKER = module("f024_successor_cache_marker_hostile_marker", HERE / "successor_cache_marker.py")


class HostileMarkerTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.repository, self.external = CACHE.PROJECT_ROOT, self.root / "external"
        self.fixture = FIXTURES.build(self.root)
        self.raw_calls, self.generation_calls = [], []

    def raw(self, member) -> bytes:
        self.raw_calls.append(member.identifier)
        return self.fixture.raw[member.identifier]

    def generate(self, generation, producers, scratch: Path) -> tuple[tuple[str, bytes], ...]:
        self.generation_calls.append(generation.identifier)
        return tuple((identifier, self.fixture.generated[identifier]) for identifier in generation.output_ids)

    def stage(self):
        return CACHE.stage_fixture(self.fixture.path, self.external, self.repository, self.raw, self.generate)

    def plan(self):
        return CACHE._plan(self.fixture.path, CACHE.IDENTITY.PREDECESSOR_PATHS)

    def marker(self) -> Path:
        return MARKER.location(self.external, self.plan())

    def refuses(self, pattern: str) -> None:
        self.raw_calls.clear()
        self.generation_calls.clear()
        with self.assertRaisesRegex(CACHE.CacheError, pattern):
            CACHE.verify_fixture(self.fixture.path, self.external, self.repository)
        with self.assertRaisesRegex(CACHE.CacheError, pattern):
            self.stage()
        self.assertEqual((self.raw_calls, self.generation_calls), ([], []))

    def reseal(self, value: dict[str, object]) -> bytes:
        value["marker_sha256"] = MARKER.canonical(
            {key: item for key, item in value.items() if key != "marker_sha256"}
        )
        return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode()

    def test_missing_marker_is_not_a_hit_and_verify_does_not_create_a_root(self) -> None:
        with self.assertRaisesRegex(CACHE.CacheMiss, "marker is missing|lock is missing"):
            CACHE.verify_fixture(self.fixture.path, self.external, self.repository)
        self.assertFalse(self.external.exists())

    def test_deleted_generated_member_with_marker_refuses_repair(self) -> None:
        self.stage()
        target = self.external / self.plan().members[-1].cache_path
        target.unlink()
        self.refuses("is missing")
        self.assertTrue(self.marker().is_file())

    def test_malformed_self_resealed_and_recursive_markers_refuse_repair(self) -> None:
        self.stage()
        marker, original = self.marker(), self.marker().read_bytes()
        altered = json.loads(original)
        altered["members"].reverse()
        changed = json.loads(original)
        changed["predecessor"]["decision"] = "admitted"
        invalid = json.loads(original)
        invalid["status"] = "admitted"
        stale = json.loads(original)
        stale["logical_id"] = "tampered"
        cases = (
            (b'{"schema":1,"schema":1}', "duplicate JSON"),
            (self.reseal(altered), "exactly match"),
            (self.reseal(changed), "exactly match"),
            (self.reseal(invalid), "invalid state"),
            (json.dumps(stale).encode(), "stale self digest"),
            (b"\xff", "cannot be read"),
            (b" " * (MARKER.MARKER_LIMIT + 1), "exceeds its byte limit"),
            (("[" * 1100 + "0" + "]" * 1100).encode(), "invalid schema"),
        )
        for payload, pattern in cases:
            with self.subTest(pattern=pattern):
                marker.write_bytes(payload)
                self.refuses(pattern)
                marker.write_bytes(original)

    def test_valid_marker_from_a_different_closure_refuses_repair(self) -> None:
        self.stage()
        other = json.loads(self.fixture.path.read_text(encoding="utf-8"))
        other["members"][-1]["selector"] = "generated/different-registry.adoc"
        FIXTURES.generation(other)
        FIXTURES.seal(other)
        other_path = self.root / "different-closure.fixture.json"
        other_path.write_text(json.dumps(other), encoding="utf-8")
        self.marker().write_bytes(MARKER.bytes_for(CACHE._plan(other_path, CACHE.IDENTITY.PREDECESSOR_PATHS)))
        self.refuses("exactly match")

    def test_marker_symlink_refuses_repair(self) -> None:
        self.stage()
        marker = self.marker()
        outside = self.root / "outside-marker.json"
        outside.write_bytes(marker.read_bytes())
        marker.unlink()
        marker.symlink_to(outside)
        self.refuses("symlink or unsafe")

    def test_readonly_complete_cache_still_rehashes_with_a_shared_lock(self) -> None:
        self.stage()
        paths = tuple(self.external.rglob("*"))
        try:
            for path in paths:
                path.chmod(0o500 if path.is_dir() else 0o400)
            self.external.chmod(0o500)
            receipt = CACHE.verify_fixture(self.fixture.path, self.external, self.repository)
            self.assertTrue(receipt.reused)
        finally:
            self.external.chmod(0o700)
            for path in paths:
                path.chmod(0o700 if path.is_dir() else 0o600)


if __name__ == "__main__":
    unittest.main()
