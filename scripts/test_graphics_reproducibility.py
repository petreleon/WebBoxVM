#!/usr/bin/env python3
"""CLI reproducibility proof for the checked graphics chunk fixture."""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CHUNKER = ROOT / "scripts/graphics_chunker.py"
CHECKER_FIXTURES = ROOT / "scripts/test_check_graphics_roadmap.py"
SPEC = ROOT / (
    "todo/graphics/00-foundation/02-reproducibility/03-file-layout/"
    "03-generation-and-checker/01-chunk-generator/fixture/input.json"
)
MANIFEST = ROOT / (
    "todo/graphics/00-foundation/01-contract/02-upstream-pins/"
    "01-input-inventory/manifest.toml"
)
EXPECTED_HASHES = {
    "chunk-0001.md": "9eb18a787a5a3e877317a43dd4839ac41d10d44e7598773ab625722a1a03e78b",
    "metadata.json": "641f57ff68a2740d68911d34a500b113bfab8899ede250ea60be8e70e9045526",
}


def invoke(script: Path, *arguments: object) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(script), *(str(argument) for argument in arguments)],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )


def run_chunker(mode: str, spec: Path, output: Path, lock: Path | None = None) -> subprocess.CompletedProcess[str]:
    flag, source = ("--source-manifest", MANIFEST) if lock is None else ("--inventory-lock", lock)
    return invoke(
        CHUNKER,
        mode,
        "--spec",
        spec,
        flag,
        source,
        "--output",
        output,
    )


def hashes(output: Path) -> dict[str, str]:
    return {
        path.name: hashlib.sha256(path.read_bytes()).hexdigest()
        for path in sorted(output.iterdir(), key=lambda path: path.name)
        if path.is_file()
    }


def write_v2_spec(path: Path, revision: str) -> None:
    document = json.loads(SPEC.read_text(encoding="utf-8"))
    document["schema"] = 2
    del document["manifest_revision"]
    document["inventory_sha256"] = revision
    path.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")


class ReproducibilityProofTests(unittest.TestCase):
    def assert_ok(self, result: subprocess.CompletedProcess[str]) -> None:
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_repeated_cli_generation_has_the_recorded_hashes(self) -> None:
        with tempfile.TemporaryDirectory(prefix="webboxvm-graphics-proof-") as temporary:
            root = Path(temporary)
            first, second = root / "first", root / "second"
            first_write = run_chunker("--write", SPEC, first)
            second_write = run_chunker("--write", SPEC, second)
            self.assert_ok(first_write)
            self.assert_ok(second_write)
            self.assertIn(EXPECTED_HASHES["metadata.json"], first_write.stdout)
            self.assertIn(EXPECTED_HASHES["metadata.json"], second_write.stdout)
            self.assertEqual(hashes(first), EXPECTED_HASHES)
            self.assertEqual(hashes(second), EXPECTED_HASHES)
            self.assertEqual(hashes(first), hashes(second))
            self.assertTrue(
                all(
                    len((first / name).read_text(encoding="utf-8").splitlines()) <= 180
                    for name in EXPECTED_HASHES
                    if name.startswith("chunk-")
                )
            )
            self.assert_ok(run_chunker("--check", SPEC, first))

    def test_stale_metadata_and_reordered_input_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory(prefix="webboxvm-graphics-proof-") as temporary:
            root = Path(temporary)
            output = root / "output"
            self.assert_ok(run_chunker("--write", SPEC, output))
            (output / "metadata.json").write_text("stale metadata\n", encoding="utf-8")
            stale = run_chunker("--check", SPEC, output)
            self.assertNotEqual(stale.returncode, 0, stale.stdout)
            self.assertIn("generated output is stale: metadata.json", stale.stderr)
            reordered = root / "reordered.json"
            document = json.loads(SPEC.read_text(encoding="utf-8"))
            document["records"].reverse()
            reordered.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")
            changed = run_chunker("--check", reordered, root / "other")
            self.assertNotEqual(changed.returncode, 0, changed.stdout)
            self.assertIn("records must be uniquely ordered by id", changed.stderr)

    def test_v2_raw_lock_cli_is_reproducible_and_rejects_stale_identity(self) -> None:
        with tempfile.TemporaryDirectory(prefix="webboxvm-graphics-v2-proof-") as temporary:
            root = Path(temporary)
            lock, spec = root / "inventory.lock", root / "input-v2.json"
            lock.write_bytes(b"raw lock only; the F02 loader owns its composition\n")
            write_v2_spec(spec, hashlib.sha256(lock.read_bytes()).hexdigest())
            first, second = root / "first", root / "second"
            self.assert_ok(run_chunker("--write", spec, first, lock))
            self.assert_ok(run_chunker("--write", spec, second, lock))
            self.assertEqual(hashes(first), hashes(second))
            metadata = json.loads((first / "metadata.json").read_text(encoding="utf-8"))
            self.assertEqual(metadata["schema"], 2)
            self.assertIn("inventory_sha256", metadata)
            self.assertNotIn("manifest_revision", metadata)
            self.assertIn("graphics-chunker.v2", (first / "chunk-0001.md").read_text(encoding="utf-8"))
            wrong = run_chunker("--check", spec, first)
            self.assertNotEqual(wrong.returncode, 0, wrong.stdout)
            self.assertIn("requires --inventory-lock", wrong.stderr)
            lock.write_bytes(b"stale raw lock\n")
            stale = run_chunker("--check", spec, first, lock)
            self.assertNotEqual(stale.returncode, 0, stale.stdout)
            self.assertIn("inventory lock identity does not match", stale.stderr)

    def test_real_roadmap_checker_fixture_suite_remains_green(self) -> None:
        result = invoke(CHECKER_FIXTURES)
        self.assert_ok(result)
        self.assertIn("Ran 5 tests", result.stderr)
        self.assertIn("OK", result.stderr)


if __name__ == "__main__":
    unittest.main()
