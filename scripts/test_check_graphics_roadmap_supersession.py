#!/usr/bin/env python3
"""Regression coverage for structured roadmap task succession."""

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

CHECKER = Path(__file__).with_name("check_graphics_roadmap.py")


def write(root, name, body):
    path = root / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(body, encoding="utf-8")


def receipt():
    return "Revision: fixture\nValidation: fixture\nResult: PASS\nArtifacts: fixture\nProfile: fixture\n"


def task(identifier, boxes, metadata=""):
    return f"""# Fixture {identifier}

Task: {identifier}
Depends: none
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


def run(root):
    return subprocess.run([sys.executable, str(CHECKER), str(root)], text=True,
                          capture_output=True, check=False)


class SupersessionTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="webboxvm-supersession-")
        self.root = Path(self.temporary.name)

    def tearDown(self):
        self.temporary.cleanup()

    def write_successor_fixture(self, replacement_done):
        root_mark, replacement_mark = ("x", "x") if replacement_done else (" ", " ")
        write(self.root, "README.md", f"# Root\n\n- [{root_mark}] [phase](phase/README.md)\n")
        write(self.root, "phase/README.md", task("R20", "- [x] [old](old/README.md)\n"
              f"- [{replacement_mark}] [new](new/README.md)"))
        write(self.root, "phase/evidence.md", receipt())
        write(self.root, "phase/old/README.md", task("R20.1", "- [ ] legacy",
              "Status: superseded\nSuperseded-by: R20.2"))
        write(self.root, "phase/old/evidence.md", receipt())
        write(self.root, "phase/new/README.md", task("R20.2", f"- [{replacement_mark}] replacement"))
        write(self.root, "phase/new/evidence.md", receipt())

    def test_parent_waits_for_its_sibling_successor(self):
        self.write_successor_fixture(False)
        pending = run(self.root)
        self.assertEqual(pending.returncode, 0, pending.stderr)
        self.assertIn("Ready: R20.2", pending.stdout)
        self.write_successor_fixture(True)
        finished = run(self.root)
        self.assertEqual(finished.returncode, 0, finished.stderr)
        self.assertIn("2 PASS-complete", finished.stdout)

    def test_successor_must_be_a_sibling_checkbox(self):
        write(self.root, "README.md", "# Root\n\n- [ ] [phase](phase/README.md)\n"
              "- [ ] [new](new/README.md)\n")
        write(self.root, "phase/README.md", task("R30", "- [x] [old](old/README.md)"))
        write(self.root, "phase/evidence.md", receipt())
        write(self.root, "phase/old/README.md", task("R30.1", "- [ ] legacy",
              "Status: superseded\nSuperseded-by: R30.2"))
        write(self.root, "phase/old/evidence.md", receipt())
        write(self.root, "new/README.md", task("R30.2", "- [ ] replacement"))
        write(self.root, "new/evidence.md", receipt())
        result = run(self.root)
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIn("superseded child successor must be a sibling checkbox", result.stderr)


if __name__ == "__main__":
    unittest.main()
