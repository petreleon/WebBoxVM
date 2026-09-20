#!/usr/bin/env python3
"""Materialize source-sealed F05 registrations through the generic runner."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

import profile_registration as registration
import profile_registration_sources as sources

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[5]
GENERIC = REPO / "scripts/graphics/run.py"
OBSERVER = HERE.relative_to(REPO) / "profile_registration_observer.py"


def output() -> dict[str, str]:
    empty = hashlib.sha256(b"").hexdigest()
    return {"stdout": "", "stderr": "", "stdout_sha256": empty, "stderr_sha256": empty}


def projection(check: dict[str, object]) -> dict[str, object]:
    return {key: check[key] for key in ("id", "kind", "profile", "profile_effect", "source", "lane")}


def blocked(check: dict[str, object], reason: str) -> dict[str, object]:
    return {"name": check["id"], "command": check["command"], "tool_versions": [],
            "prerequisites": [{"kind": "external-selector-cache", "value": "selector-cache", "reason": reason}],
            "expected_count": check["expected_count"], "observed_count": 0, "exit_status": None,
            "artifacts": [], "output": output(), "errors": [], "result": "BLOCKED",
            "registration": projection(check)}


def select(value: dict[str, object], names: list[str]) -> list[dict[str, object]]:
    if not names or any(not name.strip() for name in names) or len(set(names)) != len(names):
        registration.reject("selection must name each check exactly once")
    indexed = {item["id"]: item for item in value["checks"]}
    missing = [name for name in names if name not in indexed]
    if missing:
        registration.reject("unknown registration selection: " + ", ".join(missing))
    return [indexed[name] for name in names]


def command(check: dict[str, object], cache_root: Path | None) -> list[str]:
    subject = list(check["command"])
    if check["requires_selector_cache"]:
        if cache_root is None:
            raise sources.SourceError("selector cache root was not supplied")
        sources.verify_registry_cache(cache_root, str(check["source_contract_sha256"]) if "source_contract_sha256" in check else "")
        if subject.count("{selector_cache_root}") != 1:
            registration.reject("selector-cache command template is malformed")
        subject = [str(cache_root) if item == "{selector_cache_root}" else item for item in subject]
    elif "{selector_cache_root}" in subject:
        registration.reject("non-cache registration contains a selector-cache placeholder")
    return [sys.executable, str(OBSERVER), "--count", str(check["expected_count"]), "--", *subject]


def generic_check(check: dict[str, object], cache_root: Path | None, contract_sha256: str) -> tuple[dict[str, object], int, str, str]:
    command_value = command({**check, "source_contract_sha256": contract_sha256}, cache_root)
    generic = {"schema": 1, "checks": [{"name": check["id"], "command": command_value,
               "expected_count": check["expected_count"], "artifacts": check["artifacts"],
               "tools": check["tools"], "prerequisites": check["prerequisites"]}]}
    with tempfile.TemporaryDirectory(prefix=".f05-registration-", dir=HERE) as temporary:
        root = Path(temporary)
        catalog_path, result_path = root / "catalog.json", root / "result.json"
        catalog_path.write_text(json.dumps(generic), encoding="utf-8")
        process = subprocess.run([sys.executable, str(GENERIC), "--root", str(REPO), "--catalog", str(catalog_path),
                                  "--select", str(check["id"]), "--result", str(result_path)], cwd=REPO,
                                 text=True, capture_output=True, check=False)
        result = json.loads(result_path.read_text(encoding="utf-8"))
    record = result["checks"][0]
    record["registration"] = projection(check)
    return record, process.returncode, process.stdout, process.stderr


def overall(checks: list[dict[str, object]]) -> str:
    values = [check["result"] for check in checks]
    return "FAIL" if "FAIL" in values else "BLOCKED" if "BLOCKED" in values else "PASS"


def receipt(value: dict[str, object], names: list[str], checks: list[dict[str, object]]) -> dict[str, object]:
    return {"schema": 1, "kind": "webboxvm-f05-profile-registration-result",
            "catalog_sha256": value["catalog_sha256"], "source_contract_sha256": value["source_contract_sha256"],
            "inventory_lock_sha256": value["inventory_lock_sha256"], "claims": value["claims"],
            "cts_executions": 0, "states": value["states"], "selection": names,
            "result": overall(checks), "checks": checks}


def write(path: Path, value: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--catalog", type=Path, default=registration.CATALOG)
    parser.add_argument("--select", action="append", default=[])
    parser.add_argument("--selector-cache-root", type=Path)
    parser.add_argument("--result", type=Path, required=True)
    args = parser.parse_args()
    try:
        value, selected = registration.load_catalog(args.catalog), None
        selected = select(value, args.select)
        checks: list[dict[str, object]] = []
        codes: list[int] = []
        streams: list[tuple[str, str]] = []
        for check in selected:
            try:
                record, code, stdout, stderr = generic_check(check, args.selector_cache_root, value["source_contract_sha256"])
            except sources.SourceError as error:
                record, code, stdout, stderr = blocked(check, str(error)), 3, "", ""
            checks.append(record)
            codes.append(code)
            streams.append((stdout, stderr))
        value_out = receipt(value, args.select, checks)
        write(args.result, value_out)
        for stdout, stderr in streams:
            sys.stdout.write(stdout)
            sys.stderr.write(stderr)
        if value_out["result"] == "PASS":
            raise SystemExit(0)
        if value_out["result"] == "BLOCKED":
            raise SystemExit(3)
        raise SystemExit(next((code for code in codes if code not in (0, 3)), 1))
    except (registration.RegistrationError, sources.SourceError, OSError, json.JSONDecodeError) as error:
        print(f"profile registration: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
