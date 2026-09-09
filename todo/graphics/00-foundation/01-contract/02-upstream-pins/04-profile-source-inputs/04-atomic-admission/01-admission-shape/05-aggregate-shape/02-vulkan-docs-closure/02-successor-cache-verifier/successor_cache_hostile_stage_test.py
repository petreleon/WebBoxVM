"""Hostile callback and filesystem cases for successor-cache staging."""

from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

HERE = Path(__file__).resolve().parent


def module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    value = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = value
    spec.loader.exec_module(value)
    return value


CACHE = module("f024_successor_cache_hostile_stage", HERE / "successor_cache_contract.py")
FIXTURES = module("f024_successor_cache_fixture_hostile_stage", HERE / "successor_cache_fixture.py")
MARKER = module("f024_successor_cache_marker_hostile_stage", HERE / "successor_cache_marker.py")


class HostileStageTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        root = Path(self.temporary.name).resolve()
        self.repository, self.external = CACHE.PROJECT_ROOT, root / "external"
        self.fixture = FIXTURES.build(root)
        self.raw_calls, self.generation_calls = [], []

    def raw(self, member) -> bytes:
        self.raw_calls.append(member.identifier)
        return self.fixture.raw[member.identifier]

    def generation(self, generation, producers, scratch: Path) -> tuple[tuple[str, bytes], ...]:
        self.generation_calls.append(generation.identifier)
        return tuple((identifier, self.fixture.generated[identifier]) for identifier in generation.output_ids)

    def stage(self, raw=None, generation=None, external=None, repository=None):
        return CACHE.stage_fixture(self.fixture.path, external or self.external,
                                   self.repository if repository is None else repository,
                                   self.raw if raw is None else raw,
                                   self.generation if generation is None else generation)

    def marker(self) -> Path:
        plan = CACHE._plan(self.fixture.path, CACHE.IDENTITY.PREDECESSOR_PATHS)
        return MARKER.location(self.external, plan)

    def test_invalid_fixture_or_provider_fails_before_cache_creation(self) -> None:
        value = json.loads(self.fixture.path.read_text(encoding="utf-8"))
        value["status"] = "admitted"
        self.fixture.path.write_text(json.dumps(value), encoding="utf-8")
        with self.assertRaisesRegex(CACHE.CacheError, "fixture validation"):
            self.stage()
        self.assertFalse(self.external.exists())
        self.fixture = FIXTURES.build(Path(self.temporary.name))
        with self.assertRaisesRegex(CACHE.CacheError, "provider is not callable"):
            self.stage(raw=object())
        self.assertFalse(self.external.exists())

    def test_inside_repository_root_and_root_or_component_symlinks_fail(self) -> None:
        with self.assertRaisesRegex(CACHE.CacheError, "filesystem root"):
            self.stage(external=Path("/"))
        with self.assertRaisesRegex(CACHE.CacheError, "outside the repository"):
            self.stage(external=self.repository / "cache")
        claimed = Path(self.temporary.name) / "claimed-repository"
        claimed.mkdir()
        inside = self.repository / f".successor-cache-test-{Path(self.temporary.name).name}"
        with self.assertRaisesRegex(CACHE.CacheError, "real project root"):
            self.stage(external=inside, repository=claimed)
        self.assertFalse(inside.exists())
        outside = Path(self.temporary.name) / "outside"
        outside.mkdir()
        self.external.symlink_to(outside, target_is_directory=True)
        with self.assertRaisesRegex(CACHE.CacheError, "root is a symlink"):
            self.stage()
        self.external.unlink()
        self.external.mkdir()
        (self.external / "webboxvm-graphics").symlink_to(outside, target_is_directory=True)
        with self.assertRaisesRegex(CACHE.CacheError, "symlink or unsafe"):
            self.stage()
        self.assertEqual((self.raw_calls, self.generation_calls), ([], []))

    def test_ancestor_symlink_cannot_redirect_the_external_cache_root(self) -> None:
        outside, alias = self.external.parent / "outside", self.external.parent / "alias"
        outside.mkdir()
        alias.symlink_to(outside, target_is_directory=True)
        with self.assertRaisesRegex(CACHE.CacheError, "symlink component"):
            self.stage(external=alias / "redirected-cache")
        self.assertFalse((outside / "redirected-cache").exists())

    def test_invalid_external_root_never_escapes_the_cache_error_contract(self) -> None:
        malformed = Path(f"{self.external}{chr(0)}path")
        with self.assertRaisesRegex(CACHE.CacheError, "unsafe path component"):
            self.stage(external=malformed)
        self.assertEqual((self.raw_calls, self.generation_calls), ([], []))

    def test_case_alias_inside_repository_is_rejected_when_supported(self) -> None:
        alias = self.repository.parent / self.repository.name.swapcase()
        if not alias.exists() or not alias.samefile(self.repository):
            self.skipTest("the test volume is case-sensitive")
        inside = alias / f".case-cache-{Path(self.temporary.name).name}"
        with self.assertRaisesRegex(CACHE.CacheError, "outside the repository"):
            self.stage(external=inside)
        self.assertFalse(inside.exists())

    def test_bad_raw_payload_or_existing_target_fails_without_marker(self) -> None:
        with self.assertRaisesRegex(CACHE.CacheError, "not bytes"):
            self.stage(raw=lambda member: "not-bytes")
        self.assertFalse(self.marker().exists())
        with self.assertRaisesRegex(CACHE.CacheError, "sha256 mismatch"):
            self.stage(raw=lambda member: b"x" * len(self.fixture.raw[member.identifier]))
        attempted = []
        def failing_raw(member):
            attempted.append(member.identifier)
            raise RuntimeError("offline")
        with self.assertRaisesRegex(CACHE.CacheError, "raw reader failed"):
            self.stage(raw=failing_raw)
        self.assertEqual(attempted, ["fixture-root"])
        plan = CACHE._plan(self.fixture.path, CACHE.IDENTITY.PREDECESSOR_PATHS)
        target = self.external / plan.members[0].cache_path
        target.parent.mkdir(parents=True)
        target.write_bytes(b"corrupt")
        with self.assertRaisesRegex(CACHE.CacheError, "byte count mismatch"):
            self.stage()
        self.assertFalse(self.marker().exists())
        self.assertEqual(self.generation_calls, [])

    def test_runner_reorder_missing_extra_or_wrong_bytes_never_completes(self) -> None:
        failures = (
            lambda generation, producers, scratch: (),
            lambda generation, producers, scratch: tuple(reversed([
                (identifier, self.fixture.generated[identifier]) for identifier in generation.output_ids
            ])),
            lambda generation, producers, scratch: tuple(
                (identifier, b"bad") for identifier in generation.output_ids
            ),
            lambda generation, producers, scratch: tuple(
                (identifier, "not-bytes") for identifier in generation.output_ids
            ),
            lambda generation, producers, scratch: tuple(
                (identifier, self.fixture.generated[identifier]) for identifier in generation.output_ids
            ) + (("extra", b"extra"),),
        )
        for callback in failures:
            with self.subTest(callback=callback):
                self.raw_calls.clear()
                self.generation_calls.clear()
                with self.assertRaises(CACHE.CacheError):
                    self.stage(generation=callback)
                self.assertFalse(self.marker().exists())

    def test_defensive_clean_run_divergence_gate_never_completes(self) -> None:
        plan = CACHE._plan(self.fixture.path, CACHE.IDENTITY.PREDECESSOR_PATHS)
        generation = plan.generations[0]
        first = tuple((identifier, self.fixture.generated[identifier]) for identifier in generation.output_ids)
        second = tuple((identifier, b"different") for identifier in generation.output_ids)
        with mock.patch.object(CACHE, "_run", side_effect=(first, second)):
            with self.assertRaisesRegex(CACHE.CacheError, "clean runs diverged"):
                self.stage()
        self.assertFalse(self.marker().exists())


if __name__ == "__main__":
    unittest.main()
