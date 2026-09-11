#!/usr/bin/env python3
"""Hermetic CLI tests for the profile-independent local check runner."""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts/graphics/run.py"


class GraphicsRunnerTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix=".graphics-runner-", dir=ROOT)
        self.work = Path(self.temporary.name)
        self.number = 0

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def check(self, name: str, code: str, **changes: object) -> dict[str, object]:
        value: dict[str, object] = {
            "name": name, "command": [sys.executable, "-c", code], "expected_count": 1,
            "artifacts": [], "tools": [[sys.executable, "--version"]], "prerequisites": [],
        }
        value.update(changes)
        return value

    def catalog(self, checks: list[dict[str, object]]) -> Path:
        path = self.work / f"catalog-{self.number}.json"
        self.number += 1
        path.write_text(json.dumps({"schema": 1, "checks": checks}), encoding="utf-8")
        return path

    def invoke(self, catalog: Path, *selection: str, environment: dict[str, str] | None = None):
        result = self.work / f"result-{self.number}.json"
        self.number += 1
        command = [sys.executable, str(RUNNER), "--root", str(ROOT), "--catalog", str(catalog)]
        for name in selection:
            command.extend(("--select", name))
        command.extend(("--result", str(result)))
        env = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
        if environment:
            env.update(environment)
        process = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, check=False, env=env)
        return process, json.loads(result.read_text(encoding="utf-8"))

    def test_success_records_identity_counts_tools_and_artifact_hash(self) -> None:
        relative = f"{self.work.name}/artifact.bin"
        code = f"from pathlib import Path; Path({relative!r}).write_bytes(b'fixture'); print('WEBBOXVM_GRAPHICS_OBSERVED_COUNT=2')"
        process, result = self.invoke(self.catalog([
            self.check("fixture-success", code, expected_count=2, artifacts=[relative])
        ]), "fixture-success")
        check = result["checks"][0]
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(result["result"], "PASS")
        self.assertRegex(result["revision"], r"^[0-9a-f]{40}$")
        self.assertRegex(result["dirty_diff_sha256"], r"^[0-9a-f]{64}$")
        self.assertEqual((check["expected_count"], check["observed_count"]), (2, 2))
        self.assertEqual(check["artifacts"][0]["sha256"], hashlib.sha256(b"fixture").hexdigest())
        self.assertEqual(check["tool_versions"][0]["exit_status"], 0)

    def test_failing_child_preserves_its_exit_status_and_streams(self) -> None:
        code = "import sys; print('child-out'); print('child-err', file=sys.stderr); print('WEBBOXVM_GRAPHICS_OBSERVED_COUNT=1'); raise SystemExit(17)"
        process, result = self.invoke(self.catalog([self.check("fixture-fail", code)]), "fixture-fail")
        self.assertEqual(process.returncode, 17)
        self.assertEqual(process.stdout, "child-out\nWEBBOXVM_GRAPHICS_OBSERVED_COUNT=1\n")
        self.assertEqual(process.stderr, "child-err\n")
        self.assertEqual(result["checks"][0]["result"], "FAIL")
        self.assertEqual(result["checks"][0]["exit_status"], 17)

    def test_empty_unknown_and_zero_count_selections_fail(self) -> None:
        catalog = self.catalog([self.check("zero", "print('WEBBOXVM_GRAPHICS_OBSERVED_COUNT=0')")])
        empty, empty_result = self.invoke(catalog, "")
        unknown, unknown_result = self.invoke(catalog, "absent")
        zero, zero_result = self.invoke(catalog, "zero")
        self.assertEqual((empty.returncode, unknown.returncode, zero.returncode), (2, 2, 1))
        self.assertEqual((empty_result["result"], unknown_result["result"], zero_result["result"]), ("INVALID", "INVALID", "FAIL"))
        self.assertIn("at least one", empty_result["errors"][0])
        self.assertIn("unknown", unknown_result["errors"][0])
        self.assertEqual(zero_result["checks"][0]["observed_count"], 0)

    def test_malformed_catalog_is_rejected_with_a_local_result(self) -> None:
        catalog = self.work / "malformed.json"
        catalog.write_text("{ not-json", encoding="utf-8")
        process, result = self.invoke(catalog, "anything")
        self.assertEqual(process.returncode, 2)
        self.assertEqual(result["result"], "INVALID")
        self.assertIn("cannot load catalog", result["errors"][0])

    def test_each_missing_prerequisite_is_blocked_without_running_the_child(self) -> None:
        kinds = [
            ("executable", {"kind": "executable", "value": "webboxvm-no-such-executable"}),
            ("asset", {"kind": "asset", "value": f"{self.work.name}/missing.asset"}),
            ("browser", {"kind": "browser", "value": "webboxvm-no-such-browser"}),
            ("hardware", {"kind": "hardware", "value": "WEBBOXVM_TEST_MISSING_HARDWARE"}),
            ("permission", {"kind": "permission", "value": f"{self.work.name}/missing.probe", "access": "read"}),
        ]
        checks = [self.check(name, "raise SystemExit(99)", prerequisites=[requirement]) for name, requirement in kinds]
        process, result = self.invoke(self.catalog(checks), *(name for name, _ in kinds), environment={"WEBBOXVM_TEST_MISSING_HARDWARE": ""})
        self.assertEqual(process.returncode, 3)
        self.assertEqual((process.stdout, process.stderr), ("", ""))
        self.assertEqual(result["result"], "BLOCKED")
        for check, (kind, _) in zip(result["checks"], kinds):
            self.assertEqual((check["result"], check["observed_count"], check["exit_status"]), ("BLOCKED", 0, None))
            self.assertEqual(check["prerequisites"][0]["kind"], kind)


if __name__ == "__main__":
    unittest.main()
