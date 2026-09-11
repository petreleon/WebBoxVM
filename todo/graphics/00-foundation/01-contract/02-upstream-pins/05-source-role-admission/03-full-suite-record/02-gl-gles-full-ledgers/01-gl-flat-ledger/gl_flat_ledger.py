#!/usr/bin/env python3
"""Receipt the exact flat OpenGL CTS case sequence without executing it."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "01-canonical-full-suite-roots"))
import full_suite_roots as roots  # noqa: E402

ROOT_ID = "opengl-cts-gl46-main"
CASE_COUNT = 19714
CASE_SEQUENCE_SHA256 = "e28bbbbfd0f6c8d711554a01aa45819bdc7ca963c9426e997dfd1daaeb1d7b17"
NO_CLAIMS = {name: False for name in roots.CLAIMS}


class LedgerError(ValueError):
    """The local observation is not the exact pinned OpenGL flat sequence."""


def reject(message: str) -> None:
    raise LedgerError(message)


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def root_record() -> dict[str, object]:
    value = roots.catalog()
    roots.validate_full_suite_catalog(value)
    records = value["records"]
    record = next((item for item in records if item["id"] == ROOT_ID), None)
    if record is None or record["profile"] != "opengl-4.6-core":
        reject("OpenGL flat ledger has no canonical GL 4.6 root")
    return record


def parse_cases(data: bytes) -> tuple[str, ...]:
    if not data or not data.endswith(b"\n") or b"\r" in data:
        reject("GL root must be nonempty LF-terminated bytes")
    try:
        lines = tuple(data[:-1].decode("utf-8").split("\n"))
    except UnicodeDecodeError as error:
        raise LedgerError(f"GL root is not UTF-8: {error}") from error
    if (not lines or any(not line or line != line.strip() or "\x00" in line for line in lines)
            or len(set(lines)) != len(lines)):
        reject("GL root has blank, padded, NUL, or duplicate cases")
    return lines


def sequence_digest(cases: tuple[str, ...]) -> str:
    raw = "\n".join(cases).encode("utf-8") + b"\n"
    return hashlib.sha256(raw).hexdigest()


def source_identity(record: dict[str, object]) -> dict[str, object]:
    return {name: record[name] for name in ("id", "revision", "sha256", "bytes", "license", "attribution",
                                              "authority", "producer", "scope", "selector_path", "unfiltered")}


def ledger(data: bytes) -> dict[str, object]:
    record = root_record()
    if (len(data), hashlib.sha256(data).hexdigest()) != (record["bytes"], record["sha256"]):
        reject("GL root bytes do not match the pinned full-suite root")
    cases = parse_cases(data)
    digest = sequence_digest(cases)
    if (len(cases), digest) != (CASE_COUNT, CASE_SEQUENCE_SHA256):
        reject("GL case sequence differs from the reviewed exact root")
    body = {
        "schema": 1, "kind": "webboxvm-engineering-map", "authority": "WebBoxVM", "producer": "WebBoxVM",
        "claims": dict(NO_CLAIMS), "cts_executions": 0, "source_root": source_identity(record),
        "shape": "ordered-unique-flat-case-sequence", "case_count": len(cases), "case_sequence_sha256": digest,
    }
    return {**body, "ledger_sha256": hashlib.sha256(canonical(body)).hexdigest()}


def validate_ledger(value: object, data: bytes) -> None:
    if value != ledger(data):
        reject("local GL ledger differs from its exact source observation")


def fresh_cache(cache_root: Path):
    if not cache_root.is_absolute():
        reject("fresh cache root must be absolute")
    cache = roots.ExternalCache.from_path(cache_root, roots.repository_root(HERE))
    try:
        roots.require_fresh_cache(cache.root)
    except roots.FullSuiteError as error:
        reject(str(error))
    return cache


def refresh(cache_root: Path, timeout: float) -> dict[str, object]:
    if timeout <= 0:
        reject("timeout must be positive")
    record, cache = root_record(), fresh_cache(cache_root)
    try:
        path, reused = roots.fetch_to_cache(cache, roots.source_input(record), timeout)
        if reused:
            reject("fresh GL refresh unexpectedly reused a root")
        roots.verify_catalog({"schema": 1, "records": [record]}, cache.root)
    except (roots.ContractError, roots.RoleError, roots.FullSuiteError) as error:
        reject(str(error))
    return ledger(path.read_bytes())


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--timeout", type=float, default=30.0)
    args = parser.parse_args()
    try:
        print(json.dumps(refresh(args.cache_root, args.timeout), sort_keys=True))
    except LedgerError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
