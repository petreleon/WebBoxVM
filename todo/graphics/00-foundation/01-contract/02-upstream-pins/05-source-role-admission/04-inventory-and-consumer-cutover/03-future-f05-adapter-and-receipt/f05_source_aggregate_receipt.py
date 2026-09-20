#!/usr/bin/env python3
"""Capture a fresh selector cache and a no-claim F05 source-admission receipt."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path

import f05_source_adapter as adapter
from f05_selector_cache import F05SelectorCacheError, refresh_selector_cache, verify_selector_cache

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[7]
F03_RUNNER = REPO / "todo/graphics/00-foundation/01-contract/03-feature-matrix/01-profile-scope/validate_profile_scope.py"
NO_CLAIMS = {name: False for name in adapter.CLAIMS}


class AggregateReceiptError(ValueError):
    """The aggregate receipt is stale, qualifying, or missing a required boundary check."""


def reject(message: str) -> None:
    raise AggregateReceiptError(message)


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def _run_f03(arguments: list[str]) -> None:
    result = subprocess.run([sys.executable, str(F03_RUNNER), *arguments], cwd=F03_RUNNER.parent,
                            capture_output=True, text=True, check=False)
    if result.returncode != 0 or "source gate is complete; profile matrices remain blocked" not in result.stdout:
        reject("active F03 source gate did not retain the matrix-incomplete boundary")


def _f03_checks(admission: dict[str, object]) -> tuple[dict[str, object], dict[str, object]]:
    _run_f03([])
    with tempfile.TemporaryDirectory(prefix=".f05-matrix-", dir=HERE) as temporary:
        root = Path(temporary)
        (root / "evidence.md").write_text("# synthetic blocked schema probe\n", encoding="utf-8")
        rows = [{"profile": profile, "requirement_kind": "command", "name": f"{profile}-schema-probe",
                 "mandatory": True, "source_role": "normative-root", "source_locator": "sealed-source",
                 "condition": "always", "owner_task": "F03.2", "test_source_role": "full-suite-root",
                 "status": "blocked", "evidence": "evidence.md#blocked", "blocker": "matrix-incomplete"}
                for profile in ("opengl-4.6-core", "gles-3.2", "vulkan-1.4-core")]
        matrix = {"schema": 2, "source_contract_sha256": admission["source_contract_sha256"],
                  "inventory_lock_sha256": admission["inventory_lock_sha256"], "rows": rows}
        path = root / "matrix.json"
        path.write_text(json.dumps(matrix), encoding="utf-8")
        _run_f03(["--matrix", str(path)])
    return ({"source_gate": "complete", "profile_status": "blocked", "blocker": "matrix-incomplete"},
            {"mode": "schema-boundary-test", "imported_rows": 0, "profile_status": "blocked",
             "blocker": "matrix-incomplete"})


def _body(admission: dict[str, object], cache: dict[str, object], f03: dict[str, object],
          matrix: dict[str, object]) -> dict[str, object]:
    return {"schema": 1, "kind": "webboxvm-f05-source-aggregate-receipt", "authority": "WebBoxVM",
            "producer": "WebBoxVM", "claims": dict(NO_CLAIMS), "cts_executions": 0,
            "source_admission": copy.deepcopy(admission), "fresh_selector_cache": copy.deepcopy(cache),
            "f03_gate": f03, "matrix_role_resolution": matrix,
            "f05_boundary": {"profile_bound_registration": False, "profile_bound_check_count": 0},
            "states": {"mandatory_role_inventory_complete": True, "source_gate_complete": True,
                       "matrix_complete": False, "api_support": False, "conformance": False,
                       "certification": False, "profile_support": False, "performance": False,
                       "guest_api": False, "browser_execution": False}}


def seal(body: dict[str, object]) -> dict[str, object]:
    return {**body, "receipt_sha256": hashlib.sha256(canonical(body)).hexdigest()}


def _checked_body(value: object) -> dict[str, object]:
    fields = {"schema", "kind", "authority", "producer", "claims", "cts_executions", "source_admission",
              "fresh_selector_cache", "f03_gate", "matrix_role_resolution", "f05_boundary", "states",
              "receipt_sha256"}
    if not isinstance(value, dict) or set(value) != fields or value.get("schema") != 1:
        reject("aggregate receipt has an unexpected schema")
    body = {key: item for key, item in value.items() if key != "receipt_sha256"}
    if value["receipt_sha256"] != hashlib.sha256(canonical(body)).hexdigest() or len(canonical(value)) > 65536:
        reject("aggregate receipt has a stale self hash or exceeds the compact limit")
    if value["claims"] != NO_CLAIMS or value["cts_executions"] != 0:
        reject("aggregate receipt promotes a qualification claim or CTS execution")
    admission = adapter.admitted_source_contract()
    if value["source_admission"] != admission:
        reject("aggregate receipt does not bind the complete admitted source contract")
    cache = value["fresh_selector_cache"]
    bound = {item["record_id"] for item in admission["bindings"]}
    upstream = {item["id"] for item in admission["records"] if item["kind"] == "upstream-source"}
    if (not isinstance(cache, dict) or set(cache) != {"mode", "file_count", "source_contract_sha256",
                                                       "selector_record_ids", "release_proof_ids"}
            or cache.get("mode") != "fresh-refresh" or cache.get("file_count") != 9
            or cache.get("source_contract_sha256") != admission["source_contract_sha256"]
            or set(cache.get("selector_record_ids", [])) != bound | (upstream - bound)
            or not isinstance(cache.get("release_proof_ids"), list) or len(cache["release_proof_ids"]) != 2):
        reject("aggregate receipt has incomplete fresh selector-cache evidence")
    if value["f03_gate"] != {"source_gate": "complete", "profile_status": "blocked", "blocker": "matrix-incomplete"}:
        reject("aggregate receipt promotes the F03 source gate")
    if value["matrix_role_resolution"] != {"mode": "schema-boundary-test", "imported_rows": 0,
                                             "profile_status": "blocked", "blocker": "matrix-incomplete"}:
        reject("aggregate receipt misstates matrix coverage")
    if value["f05_boundary"] != {"profile_bound_registration": False, "profile_bound_check_count": 0}:
        reject("aggregate receipt fabricates an F05 profile registration")
    expected = {"mandatory_role_inventory_complete": True, "source_gate_complete": True, "matrix_complete": False,
                "api_support": False, "conformance": False, "certification": False, "profile_support": False,
                "performance": False, "guest_api": False, "browser_execution": False}
    if value["states"] != expected:
        reject("aggregate receipt has an invalid no-claim state")
    return copy.deepcopy(value)


def validate_receipt(value: object, cache_root: Path | None = None) -> dict[str, object]:
    receipt = _checked_body(value)
    try:
        if cache_root is not None and receipt["fresh_selector_cache"] != verify_selector_cache(cache_root):
            reject("aggregate receipt does not match the verified selector cache")
    except F05SelectorCacheError as error:
        reject(str(error))
    return receipt


def build_receipt(cache_root: Path, timeout: float = 60.0) -> dict[str, object]:
    admission = adapter.admitted_source_contract()
    f03, matrix = _f03_checks(admission)
    return validate_receipt(seal(_body(admission, refresh_selector_cache(cache_root, timeout), f03, matrix)), cache_root)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--selector-cache-root", type=Path, required=True)
    parser.add_argument("--timeout", type=float, default=60.0)
    parser.add_argument("--receipt", type=Path)
    args = parser.parse_args()
    try:
        value = (validate_receipt(json.loads(args.receipt.read_text(encoding="utf-8")), args.selector_cache_root)
                 if args.receipt else build_receipt(args.selector_cache_root, args.timeout))
        print(json.dumps(value, sort_keys=True, indent=2))
    except (AggregateReceiptError, F05SelectorCacheError, OSError, json.JSONDecodeError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
