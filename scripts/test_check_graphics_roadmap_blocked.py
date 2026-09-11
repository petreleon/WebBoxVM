"""Regression fixtures for declared external roadmap blockers."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from test_check_graphics_roadmap import receipt, run, task, write


def blocked_receipt() -> str:
    return receipt().replace("Result: PASS", "Result: BLOCKED")


class BlockedRoadmapTests(unittest.TestCase):
    def fixture_root(self) -> tempfile.TemporaryDirectory[str]:
        return tempfile.TemporaryDirectory(prefix="webboxvm-blocked-roadmap-")

    def blocked_fixture(
        self, root: Path, metadata: str = "Status: blocked\nBlocked-by: external/fixture-authority",
        boxes: str = "- [ ] waiting for authority", evidence: str | None = None,
    ) -> None:
        write(root, "README.md", "# Fixture root\n\n- [ ] [phase](phase/README.md)\n")
        write(root, "phase/README.md", task("R01", "- [ ] [leaf](child/README.md)"))
        write(root, "phase/evidence.md", receipt())
        write(root, "phase/child/README.md", task("R01.1", boxes, metadata=metadata))
        write(root, "phase/child/evidence.md", evidence or blocked_receipt())

    def assert_error(self, result, expected: str) -> None:
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIn(expected, result.stderr)

    def test_valid_blocker_is_listed_separately_and_blocks_dependents(self) -> None:
        with self.fixture_root() as temporary:
            root = Path(temporary)
            self.blocked_fixture(root)
            write(root, "dependent.md", task("R02", "- [ ] dependent work", deps="R01.1"))
            write(root, "evidence.md", receipt())
            result = run(root)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("Ready: none", result.stdout)
            self.assertIn("Blocked: R01.1 (external/fixture-authority)", result.stdout)

    def test_blocker_requires_a_stable_external_identifier(self) -> None:
        for metadata in ("Status: blocked", "Status: blocked\nBlocked-by: external/Not-Stable"):
            with self.subTest(metadata=metadata), self.fixture_root() as temporary:
                root = Path(temporary)
                self.blocked_fixture(root, metadata=metadata)
                self.assert_error(run(root), "blocked task needs Blocked-by")

    def test_blocker_requires_a_local_blocked_receipt(self) -> None:
        with self.fixture_root() as temporary:
            root = Path(temporary)
            self.blocked_fixture(root, evidence=receipt())
            self.assert_error(run(root), "blocked task requires Result: BLOCKED")
        with self.fixture_root() as temporary:
            root = Path(temporary)
            self.blocked_fixture(root)
            leaf = root / "phase/child/README.md"
            leaf.write_text(leaf.read_text().replace("[receipt](evidence.md)", "[receipt](../evidence.md)"))
            self.assert_error(run(root), "blocked task needs a local evidence receipt")

    def test_blocker_must_remain_an_open_leaf_with_closed_dependencies(self) -> None:
        with self.fixture_root() as temporary:
            root = Path(temporary)
            self.blocked_fixture(root, boxes="- [x] wrongly complete")
            self.assert_error(run(root), "blocked task requires an incomplete leaf")
        with self.fixture_root() as temporary:
            root = Path(temporary)
            write(root, "README.md", task("R03", "- [ ] waiting", deps="R04",
                                             metadata="Status: blocked\nBlocked-by: external/fixture-authority"))
            write(root, "evidence.md", blocked_receipt())
            write(root, "prerequisite.md", task("R04", "- [ ] incomplete"))
            self.assert_error(run(root), "blocked task requires every dependency PASS-complete")


if __name__ == "__main__":
    unittest.main()
