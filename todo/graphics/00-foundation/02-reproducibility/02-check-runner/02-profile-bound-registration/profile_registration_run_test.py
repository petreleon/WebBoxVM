#!/usr/bin/env python3
"""Execution-boundary tests for F05 profile registration."""

from __future__ import annotations

import copy
import hashlib
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import profile_registration as registration
import profile_registration_sources as sources

HERE = Path(__file__).resolve().parent
RUNNER = HERE / "profile_registration_run.py"


def rehash(value: dict[str, object]) -> dict[str, object]:
    body = {key: item for key, item in value.items() if key != "catalog_sha256"}
    value["catalog_sha256"] = hashlib.sha256(registration.canonical(body)).hexdigest()
    return value


class RegistrationRunTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix=".f05-registration-test-", dir=HERE)
        self.work = Path(self.temporary.name)
        self.base = registration.document(registration.CATALOG)

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def catalog(self, value: dict[str, object]) -> Path:
        path = self.work / "catalog.json"
        path.write_text(json.dumps(rehash(value)), encoding="utf-8")
        return path

    def invoke(self, catalog: Path, name: str, cache: Path | None = None):
        result = self.work / f"{name}.json"
        command = [sys.executable, str(RUNNER), "--catalog", str(catalog), "--select", name, "--result", str(result)]
        if cache is not None:
            command.extend(("--selector-cache-root", str(cache)))
        process = subprocess.run(command, text=True, capture_output=True, check=False,
                                 env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
        return process, json.loads(result.read_text(encoding="utf-8"))

    def baseline(self, code: str, prerequisites: list[dict[str, object]] | None = None) -> dict[str, object]:
        value, check = copy.deepcopy(self.base), None
        check = value["checks"][1]
        check.update(command=[sys.executable, "-c", code], tools=[[sys.executable, "--version"]])
        if prerequisites is not None:
            check["prerequisites"] = prerequisites
        value["checks"] = [check]
        return value

    def implementation_with_missing_prerequisites(self, sentinel: Path) -> dict[str, object]:
        value, contract = copy.deepcopy(self.base), sources.admitted_contract()
        bindings = {(item["profile"], item["role"]): item for item in contract["bindings"]}
        check = value["checks"][1]
        check.update(id="vulkan-missing-probes", kind="implementation", profile="vulkan-1.4-core",
                     profile_effect="blocked-observation", lane="implementation", requires_selector_cache=False,
                     source={"roles": [bindings[("vulkan-1.4-core", "normative-root")],
                                       bindings[("vulkan-1.4-core", "full-suite-root")]]},
                     command=[sys.executable, "-c", f"from pathlib import Path; Path({str(sentinel)!r}).write_text('ran')"],
                     tools=[[sys.executable, "--version"]], prerequisites=[
                         {"kind": "browser", "value": "webboxvm-missing-browser"},
                         {"kind": "hardware", "value": "WEBBOXVM_F05_MISSING_HARDWARE"},
                         {"kind": "asset", "value": f"{self.work.name}/missing-guest-image"},
                         {"kind": "asset", "value": f"{self.work.name}/missing-cts-suite"},
                     ])
        value["checks"], value["states"]["profile_implementation_count"] = [check], 1
        return value

    def test_missing_or_invalid_external_cache_blocks_without_child(self) -> None:
        value, sentinel = copy.deepcopy(self.base), self.work / "ran"
        value["checks"] = [value["checks"][0]]
        value["checks"][0]["command"] = [sys.executable, "-c", f"from pathlib import Path; Path({str(sentinel)!r}).write_text('ran')"]
        catalog = self.catalog(value)
        for cache in (None, Path("relative-cache")):
            with self.subTest(cache=cache):
                process, result = self.invoke(catalog, "vulkan-registry-inventory-v2", cache)
                check = result["checks"][0]
                self.assertEqual((process.returncode, result["result"], check["result"], check["exit_status"]), (3, "BLOCKED", "BLOCKED", None))
                self.assertFalse(sentinel.exists())

    def test_missing_browser_hardware_guest_and_cts_stay_blocked(self) -> None:
        sentinel = self.work / "implementation-ran"
        process, result = self.invoke(self.catalog(self.implementation_with_missing_prerequisites(sentinel)), "vulkan-missing-probes")
        check = result["checks"][0]
        self.assertEqual((process.returncode, result["result"], check["result"], check["exit_status"]), (3, "BLOCKED", "BLOCKED", None))
        self.assertEqual([item["kind"] for item in check["prerequisites"]], ["browser", "hardware", "asset", "asset"])
        self.assertFalse(sentinel.exists())

    def test_observer_preserves_success_count_and_child_failure_streams(self) -> None:
        success = self.catalog(self.baseline("print('child-out')"))
        process, result = self.invoke(success, "make-test")
        self.assertEqual((process.returncode, result["result"], result["checks"][0]["observed_count"]), (0, "PASS", 1))
        self.assertIn("child-out\nWEBBOXVM_GRAPHICS_OBSERVED_COUNT=1", process.stdout)
        failed = self.catalog(self.baseline("import sys; print('child-out'); print('child-err', file=sys.stderr); raise SystemExit(17)"))
        process, result = self.invoke(failed, "make-test")
        check = result["checks"][0]
        self.assertEqual((process.returncode, result["result"], check["exit_status"]), (17, "FAIL", 17))
        self.assertEqual((check["output"]["stdout"], check["output"]["stderr"]), ("child-out\n", "child-err\n"))


if __name__ == "__main__":
    unittest.main()
