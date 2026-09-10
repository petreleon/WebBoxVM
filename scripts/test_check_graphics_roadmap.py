#!/usr/bin/env python3
"""Regression fixtures for the graphics-roadmap structural checker."""

from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

CHECKER = Path(__file__).with_name("check_graphics_roadmap.py")


def write(root: Path, relative: str, body: str) -> None:
    path = root / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(body, encoding="utf-8")


def receipt() -> str:
    return """Revision: fixture
Validation: fixture
Result: PASS
Artifacts: fixture
Profile: structural fixture
"""


def task(identifier: str, boxes: str, deps: str = "none", metadata: str = "") -> str:
    return f"""# Fixture {identifier}

Task: {identifier}
Depends: {deps}
{metadata}
Evidence: [receipt](evidence.md)

## Outcome

Fixture outcome.

## Starting points

- fixture

## Checklist

{boxes}

## Verification

- fixture verification.
"""


def run(root: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(CHECKER), str(root)], text=True, capture_output=True, check=False
    )


def valid_fixture(root: Path) -> None:
    write(root, "README.md", "# Fixture root\n\n- [x] [phase](phase/README.md)\n")
    write(root, "phase/README.md", task("R01", "- [x] [leaf](child/README.md)"))
    write(root, "phase/evidence.md", receipt())
    write(root, "phase/child/README.md", task("R01.1", "- [x] fixture complete"))
    write(root, "phase/child/evidence.md", receipt())


class RoadmapCheckerTests(unittest.TestCase):
    def fixture_root(self) -> tempfile.TemporaryDirectory[str]:
        return tempfile.TemporaryDirectory(prefix="webboxvm-roadmap-")

    def assert_first_error(self, prepare, expected: str) -> None:
        with self.fixture_root() as temporary:
            root = Path(temporary)
            prepare(root)
            result = run(root)
            self.assertNotEqual(result.returncode, 0, result.stdout)
            self.assertIn(expected, result.stderr.splitlines()[0])

    def test_valid_nested_child_lists_pass_the_real_entrypoint(self) -> None:
        with self.fixture_root() as temporary:
            root = Path(temporary)
            valid_fixture(root)
            result = run(root)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("Ready: none", result.stdout)

    def test_broken_link_is_the_first_diagnostic(self) -> None:
        self.assert_first_error(
            lambda root: write(root, "README.md", "# Fixture\n\n[missing](missing.md)\n"),
            "broken link: missing.md",
        )

    def test_nested_child_at_the_wrong_depth_is_rejected(self) -> None:
        def prepare(root: Path) -> None:
            write(root, "README.md", "# Fixture\n\n- [ ] [deep](deep/child/README.md)\n")
            write(root, "deep/child/README.md", task("R02", "- [ ] incomplete"))
            write(root, "deep/child/evidence.md", receipt())
        self.assert_first_error(prepare, "child list must live in an immediate subfolder")

    def test_parent_status_must_match_its_completed_child(self) -> None:
        def prepare(root: Path) -> None:
            write(root, "README.md", "# Fixture\n\n- [ ] [child](child/README.md)\n")
            write(root, "child/README.md", task("R03", "- [x] complete"))
            write(root, "child/evidence.md", receipt())
        self.assert_first_error(prepare, "checkbox does not match child completion")

    def test_181_line_roadmap_file_is_rejected(self) -> None:
        def prepare(root: Path) -> None:
            write(root, "README.md", "# Fixture\n")
            write(root, "notes.md", "line\n" * 181)
        self.assert_first_error(prepare, "notes.md: exceeds 180 physical lines")

    def test_python_cache_is_ignored_but_maintained_source_is_not(self) -> None:
        def prepare(root: Path) -> None:
            valid_fixture(root)
            write(root, "__pycache__/ignored.py", "line\n" * 181)
            write(root, "ignored.pyc", "line\n" * 181)
            write(root, "maintained.py", "line\n" * 181)
        self.assert_first_error(prepare, "maintained.py: exceeds 180 physical lines")

    def test_superseded_task_requires_an_active_successor(self) -> None:
        def prepare(root: Path) -> None:
            write(root, "README.md", task("R04", "- [ ] legacy", metadata="Status: superseded"))
            write(root, "evidence.md", receipt())
        self.assert_first_error(prepare, "superseded task needs Superseded-by")

    def test_inherited_supersession_is_closed_but_never_pass_complete(self) -> None:
        with self.fixture_root() as temporary:
            root = Path(temporary)
            write(root, "README.md", "# Fixture\n\n- [x] [old](old/README.md)\n- [x] [new](new/README.md)\n")
            write(root, "old/README.md", task("R05", "- [ ] [child](child/README.md)",
                                                metadata="Status: superseded\nSuperseded-by: R06"))
            write(root, "old/child/README.md", task("R05.1", "- [ ] legacy child"))
            write(root, "new/README.md", task("R06", "- [x] replacement"))
            write(root, "old/evidence.md", receipt())
            write(root, "old/child/evidence.md", receipt())
            write(root, "new/evidence.md", receipt())
            result = run(root)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("1 PASS-complete, 2 superseded", result.stdout)
            self.assertIn("Ready: none", result.stdout)

    def test_active_task_cannot_depend_on_a_superseded_task(self) -> None:
        def prepare(root: Path) -> None:
            write(root, "README.md", "# Fixture\n\n- [x] [old](old/README.md)\n- [ ] [new](new/README.md)\n- [x] [replacement](replacement/README.md)\n")
            write(root, "old/README.md", task("R07", "- [ ] legacy", metadata="Status: superseded\nSuperseded-by: R09"))
            write(root, "new/README.md", task("R08", "- [ ] work", deps="R07"))
            write(root, "replacement/README.md", task("R09", "- [x] replacement"))
            write(root, "old/evidence.md", receipt())
            write(root, "new/evidence.md", receipt())
            write(root, "replacement/evidence.md", receipt())
        self.assert_first_error(prepare, "active task depends on superseded R07")

    def test_superseded_checked_task_keeps_its_receipt_validation(self) -> None:
        def prepare(root: Path) -> None:
            write(root, "README.md", "# Fixture\n\n- [x] [old](old/README.md)\n- [x] [new](new/README.md)\n")
            write(root, "old/README.md", task("R10", "- [x] legacy", metadata="Status: superseded\nSuperseded-by: R11"))
            write(root, "old/evidence.md", "Result: BLOCKED\n")
            write(root, "new/README.md", task("R11", "- [x] replacement"))
            write(root, "new/evidence.md", receipt())
        self.assert_first_error(prepare, "missing concrete Revision")


if __name__ == "__main__":
    unittest.main()
