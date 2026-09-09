"""Positive isolated successor-cache staging and reuse tests."""

from __future__ import annotations

import importlib.util
import hashlib
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


CACHE = module("f024_successor_cache_positive", HERE / "successor_cache_contract.py")
FIXTURES = module("f024_successor_cache_fixture_positive", HERE / "successor_cache_fixture.py")
MARKER = module("f024_successor_cache_marker_positive", HERE / "successor_cache_marker.py")
INVENTORY = CACHE.PROJECT_ROOT / "todo/graphics/00-foundation/01-contract/02-upstream-pins/01-input-inventory"


class SuccessorCacheTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.repository, self.external = CACHE.PROJECT_ROOT, self.root / "external"
        self.fixture = FIXTURES.build(self.root)
        self.raw_calls, self.generation_calls = [], []
        self.events = []

    def raw(self, member) -> bytes:
        self.raw_calls.append(member.identifier)
        self.events.append(f"raw:{member.identifier}")
        return self.fixture.raw[member.identifier]

    def generate(self, generation, producers, scratch: Path) -> tuple[tuple[str, bytes], ...]:
        self.generation_calls.append((generation.identifier, producers, scratch))
        self.events.append(f"generation:{generation.identifier}")
        self.assertTrue(scratch.is_dir())
        return tuple((identifier, self.fixture.generated[identifier]) for identifier in generation.output_ids)

    def stage(self):
        return CACHE.stage_fixture(self.fixture.path, self.external, self.repository, self.raw, self.generate)

    def test_empty_external_root_stages_all_members_then_publishes_one_marker(self) -> None:
        result = self.stage()
        self.assertEqual((result.reused, result.admitted, result.cutover_ready), (False, False, False))
        self.assertEqual(
            result.member_ids,
            ("fixture-root", "fixture-registry", "fixture-generated", "fixture-generated-extra"),
        )
        self.assertEqual(self.raw_calls, ["fixture-root", "fixture-registry"])
        self.assertEqual(len(self.generation_calls), 2)
        self.assertNotEqual(self.generation_calls[0][2], self.generation_calls[1][2])
        plan = CACHE._plan(self.fixture.path, CACHE.IDENTITY.PREDECESSOR_PATHS)
        marker = MARKER.location(self.external, plan)
        self.assertTrue(marker.is_file())
        self.assertFalse((self.external / "webboxvm-graphics/f02").exists())
        for member in plan.members:
            target = self.external / member.cache_path
            self.assertTrue(target.is_file())
            self.assertEqual(target.suffix, ".source" if hasattr(member, "immutable_url") else ".derived")

    def test_complete_marker_rehashes_without_provider_callbacks(self) -> None:
        self.stage()
        self.raw_calls.clear()
        self.generation_calls.clear()
        verified = CACHE.verify_fixture(self.fixture.path, self.external, self.repository)
        self.assertTrue(verified.reused)
        self.assertEqual((self.raw_calls, self.generation_calls), ([], []))
        staged = self.stage()
        self.assertTrue(staged.reused)
        self.assertEqual((self.raw_calls, self.generation_calls), ([], []))

    def test_valid_individual_files_without_marker_are_not_a_complete_hit(self) -> None:
        self.stage()
        plan = CACHE._plan(self.fixture.path, CACHE.IDENTITY.PREDECESSOR_PATHS)
        MARKER.location(self.external, plan).unlink()
        self.raw_calls.clear()
        self.generation_calls.clear()
        result = self.stage()
        self.assertFalse(result.reused)
        self.assertEqual(self.raw_calls, [])
        self.assertEqual(len(self.generation_calls), 2)
        self.assertTrue(MARKER.location(self.external, plan).is_file())

    def test_independent_staging_emits_byte_identical_marker(self) -> None:
        self.stage()
        plan = CACHE._plan(self.fixture.path, CACHE.IDENTITY.PREDECESSOR_PATHS)
        first = MARKER.location(self.external, plan).read_bytes()
        other = self.root / "other-external"
        raw, generated = self.raw, self.generate
        CACHE.stage_fixture(self.fixture.path, other, self.repository, raw, generated)
        self.assertEqual(first, MARKER.location(other, plan).read_bytes())

    def test_generation_waits_for_a_later_valid_raw_producer(self) -> None:
        self.fixture = FIXTURES.build_interleaved(self.root)
        result = self.stage()
        plan = CACHE._plan(self.fixture.path, CACHE.IDENTITY.PREDECESSOR_PATHS)
        generation = plan.generations[0].identifier
        self.assertEqual(result.member_ids, plan.member_ids)
        self.assertEqual(
            self.events,
            [
                "raw:fixture-root", "raw:fixture-registry", "raw:fixture-late-raw",
                f"generation:{generation}", f"generation:{generation}",
            ],
        )

    def test_same_generation_output_producers_are_internal_to_one_clean_run(self) -> None:
        self.fixture = FIXTURES.build_internal_producer(self.root)
        self.stage()
        self.assertEqual(self.raw_calls, ["fixture-root", "fixture-registry"])
        self.assertEqual(len(self.generation_calls), 2)
        expected = (
            ("fixture-root", self.fixture.raw["fixture-root"]),
            ("fixture-registry", self.fixture.raw["fixture-registry"]),
        )
        self.assertEqual(self.generation_calls[0][1], expected)
        self.assertEqual(self.generation_calls[1][1], expected)

    def test_isolated_staging_preserves_active_inventory_and_v1_pre_admission(self) -> None:
        active = (
            INVENTORY / "manifest.toml", INVENTORY / "inventory.lock",
            *(INVENTORY / "inputs").glob("*.toml"),
            *CACHE.IDENTITY.PREDECESSOR_PATHS.values(),
        )
        before = {path: hashlib.sha256(path.read_bytes()).hexdigest() for path in active}
        self.stage()
        after = {path: hashlib.sha256(path.read_bytes()).hexdigest() for path in active}
        closure = CACHE.IDENTITY.validate(self.fixture.path)
        self.assertEqual(before, after)
        self.assertEqual((closure.admitted, closure.cutover_ready), (False, False))


if __name__ == "__main__":
    unittest.main()
