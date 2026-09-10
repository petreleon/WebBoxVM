#!/usr/bin/env python3
"""Build and validate ordered, diagnostic-only V2 coverage reports."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
MODES = ("core-readiness", "complete-suite-diagnostics")
CATEGORIES = ("core", "wsi", "video", "extension", "unknown")
CLAIM = "diagnostic-only-not-khronos-conformance-or-certification"
FIELDS = frozenset(("schema", "kind", "report_mode", "coverage_claim", "suite_identity_sha256",
                    "suite_revision", "suite_ledger_sha256", "taxonomy_sha256", "member_count", "members",
                    "summary", "status", "report_sha256"))
ROW_FIELDS = frozenset(("path", "blob_sha1", "member_sha256", "category", "rule_id", "source_locator",
                        "tests", "skips", "failures"))

def load() -> object:
    path = HERE / "coverage_taxonomy.py"
    spec = importlib.util.spec_from_file_location("f025_v2_coverage_taxonomy", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load V2 coverage taxonomy: {path}")
    value = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = value
    spec.loader.exec_module(value)
    return value


taxonomy = load()


class ReportError(ValueError):
    """A diagnostic coverage report is incomplete, stale, or falsely narrowed."""


def reject(message: str) -> None:
    raise ReportError(message)


def no_duplicates(pairs: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for key, item in pairs:
        if key in value:
            reject("coverage report has a duplicate JSON key")
        value[key] = item
    return value


def document(path: Path) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=no_duplicates)
    except ReportError:
        raise
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"coverage report cannot be read: {error}")
    if not isinstance(value, dict):
        reject("coverage report is not an object")
    return value


def digest(value: dict[str, object]) -> str:
    body = {key: item for key, item in value.items() if key != "report_sha256"}
    return hashlib.sha256(json.dumps(body, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def result(value: object) -> dict[str, int]:
    if not isinstance(value, dict) or set(value) != {"tests", "skips", "failures"}:
        reject("coverage result has an unexpected schema")
    if any(type(value[key]) is not int or value[key] < 0 for key in value):
        reject("coverage result has an invalid count")
    if value["skips"] + value["failures"] > value["tests"]:
        reject("coverage result exceeds its recorded tests")
    return {key: value[key] for key in value}


def summarize(rows: list[dict[str, object]]) -> dict[str, object]:
    categories = {name: {"members": 0, "tests": 0, "skips": 0, "failures": 0} for name in CATEGORIES}
    for row in rows:
        bucket = categories[str(row["category"])]
        bucket["members"] += 1
        for key in ("tests", "skips", "failures"):
            bucket[key] += int(row[key])
    return {"tests": sum(row["tests"] for row in rows), "skips": sum(row["skips"] for row in rows),
            "failures": sum(row["failures"] for row in rows), "categories": categories}


def status(mode: str, summary: dict[str, object], rows: list[dict[str, object]]) -> str:
    categories = summary["categories"]
    assert isinstance(categories, dict)
    core = categories["core"]
    assert isinstance(core, dict)
    if mode == "core-readiness":
        if core["members"] == 0: return "blocked-no-reviewed-core-members"
        if core["tests"] == 0:
            return "blocked-no-recorded-core-tests"
        if categories["unknown"]["members"]:
            return "blocked-unknown-members"
        if core["skips"] or core["failures"]:
            return "blocked-core-results"
        return "core-coverage-observed-not-conformance"
    if any(row["tests"] == 0 for row in rows):
        return "complete-suite-diagnostics-incomplete-no-recorded-member-tests"
    if summary["skips"] or summary["failures"]:
        return "complete-suite-diagnostics-with-nonpass-results"
    return "complete-suite-diagnostics-clean-not-conformance"


def build(taxonomy_path: Path, identity_path: Path, ledger_path: Path, mode: str,
          outcomes: dict[str, object]) -> dict[str, object]:
    if mode not in MODES:
        reject("coverage report has an unknown mode")
    if not isinstance(outcomes, dict):
        reject("coverage outcomes are not a path map")
    try:
        view = taxonomy.classify(taxonomy_path, identity_path, ledger_path)
    except taxonomy.TaxonomyError as error:
        reject(f"coverage classification failed: {error}")
    expected_paths = [row["path"] for row in view.members]
    if set(outcomes) != set(expected_paths) or len(outcomes) != len(expected_paths):
        reject("coverage outcomes silently filter, add, or duplicate suite members")
    rows: list[dict[str, object]] = []
    for classification in view.members:
        row: dict[str, object] = dict(classification)
        row.update(result(outcomes[classification["path"]]))
        rows.append(row)
    summary = summarize(rows)
    value: dict[str, object] = {
        "schema": 1, "kind": "vulkan-cts-coverage-report", "report_mode": mode, "coverage_claim": CLAIM,
        "suite_identity_sha256": view.identity_digest, "suite_revision": view.revision,
        "suite_ledger_sha256": view.ledger_digest, "taxonomy_sha256": view.taxonomy_digest,
        "member_count": len(rows), "members": rows, "summary": summary, "status": status(mode, summary, rows),
        "report_sha256": "",
    }
    value["report_sha256"] = digest(value)
    return value


def outcomes_from(value: dict[str, object]) -> dict[str, object]:
    rows = value.get("members")
    if not isinstance(rows, list):
        reject("coverage report has no member rows")
    outcomes: dict[str, object] = {}
    for row in rows:
        if not isinstance(row, dict) or set(row) != ROW_FIELDS or not isinstance(row.get("path"), str):
            reject("coverage report has an invalid member row")
        if row["path"] in outcomes:
            reject("coverage report duplicates a member")
        outcomes[row["path"]] = result({key: row[key] for key in ("tests", "skips", "failures")})
    return outcomes


def validate(report_path: Path, taxonomy_path: Path, identity_path: Path, ledger_path: Path) -> dict[str, object]:
    value = document(report_path)
    if (set(value) != FIELDS or type(value.get("schema")) is not int or value.get("schema") != 1
            or value.get("kind") != "vulkan-cts-coverage-report" or value.get("coverage_claim") != CLAIM):
        reject("coverage report has an unexpected schema or claim")
    expected = build(taxonomy_path, identity_path, ledger_path, str(value.get("report_mode")), outcomes_from(value))
    if value != expected or value.get("report_sha256") != digest(value):
        reject("coverage report is stale, reclassified, or silently filtered")
    return value


def main() -> None:
    if len(sys.argv) != 5:
        raise SystemExit("usage: coverage_report.py REPORT.json TAXONOMY.json IDENTITY.json LEDGER.json")
    try:
        report = validate(Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3]), Path(sys.argv[4]))
    except ReportError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"REPORT: {report['report_mode']} {report['member_count']} members {report['status']}")


if __name__ == "__main__":
    main()
