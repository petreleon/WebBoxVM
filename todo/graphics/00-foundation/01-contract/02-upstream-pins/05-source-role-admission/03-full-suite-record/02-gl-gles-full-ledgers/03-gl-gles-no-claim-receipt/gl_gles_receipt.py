#!/usr/bin/env python3
"""Bind exact GL and GLES observations into one WebBoxVM no-claim receipt."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "01-gl-flat-ledger"))
sys.path.insert(0, str(HERE.parent / "02-gles-full-closure-ledger"))
import gl_flat_ledger as gl  # noqa: E402
import gles_full_ledger as gles  # noqa: E402

NO_CLAIMS = dict(gl.NO_CLAIMS)


class ReceiptError(ValueError):
    """The combined local receipt is incomplete, forged, or makes a forbidden claim."""


def reject(message: str) -> None:
    raise ReceiptError(message)


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def checked(value: object, root_id: str, member_ids: tuple[str, ...] = ()) -> dict[str, object]:
    if not isinstance(value, dict) or value.get("authority") != "WebBoxVM" or value.get("producer") != "WebBoxVM":
        reject("inner ledger is not a WebBoxVM observation")
    if value.get("claims") != NO_CLAIMS or value.get("cts_executions") != 0:
        reject("inner ledger makes a forbidden qualification claim")
    source = value.get("source_root")
    if not isinstance(source, dict) or source.get("id") != root_id:
        reject("inner ledger has an unexpected immutable root")
    if source.get("authority") != "Khronos" or source.get("producer") != "Khronos":
        reject("inner ledger loses the Khronos source authority")
    if member_ids and [item.get("id") for item in value.get("members", [])] != list(member_ids):
        reject("GLES receipt has an incomplete or reordered root closure")
    return value


def receipt(gl_data: bytes, gles_root: bytes, gles_payloads: dict[str, bytes]) -> dict[str, object]:
    try:
        gl_value = checked(gl.ledger(gl_data), gl.ROOT_ID)
        ids = tuple(gles.records.member_id(item[0]) for item in gles.records.MEMBERS)
        gles_value = checked(gles.ledger(gles_root, gles_payloads), gles.records.ROOT_ID, ids)
    except (gl.LedgerError, gles.LedgerError) as error:
        reject(str(error))
    body = {
        "schema": 1, "kind": "webboxvm-engineering-map", "authority": "WebBoxVM", "producer": "WebBoxVM",
        "claims": dict(NO_CLAIMS), "cts_executions": 0,
        "gl": {"source_root": gl_value["source_root"], "ledger_sha256": gl_value["ledger_sha256"],
               "case_count": gl_value["case_count"], "case_sequence_sha256": gl_value["case_sequence_sha256"]},
        "gles": {"source_root": gles_value["source_root"], "ledger_sha256": gles_value["ledger_sha256"],
                 "members": gles_value["members"], "configurations": gles_value["configurations"],
                 "glesext_boundary": gles_value["glesext_boundary"]},
    }
    return {**body, "receipt_sha256": hashlib.sha256(canonical(body)).hexdigest()}


def validate_receipt(value: object, gl_data: bytes, gles_root: bytes, gles_payloads: dict[str, bytes]) -> None:
    if value != receipt(gl_data, gles_root, gles_payloads):
        reject("combined GL/GLES receipt differs from its exact observations")


def external_cache(path: Path):
    try:
        return gl.roots.ExternalCache.from_path(path, gl.roots.repository_root(HERE))
    except gl.roots.ContractError as error:
        reject(str(error))


def cached_bytes(cache, source) -> bytes:
    try:
        resolved = cache.target(source)
    except gl.roots.ContractError as error:
        reject(str(error))
    raw = cache.root / source.local_cache
    current = cache.root
    for component in Path(source.local_cache).parts:
        current /= component
        if current.is_symlink():
            reject(f"cache source path for {source.identifier} must not traverse a symlink")
    if raw != resolved or not raw.is_file():
        reject(f"cache is missing a regular source file for {source.identifier}")
    return raw.read_bytes()


def receipt_from_caches(gl_cache: Path, gles_cache: Path) -> dict[str, object]:
    gl_cache, gles_cache = external_cache(gl_cache), external_cache(gles_cache)
    gl_record = gl.root_record()
    gl_data = cached_bytes(gl_cache, gl.roots.source_input(gl_record))
    records = gles.records.catalog()
    root, *members = records["records"]
    return receipt(gl_data, cached_bytes(gles_cache, gles.records.roots.source_input(root)),
                   {item["id"]: cached_bytes(gles_cache, gles.records.member_input(item)) for item in members})


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--gl-cache-root", type=Path, required=True)
    parser.add_argument("--gles-cache-root", type=Path, required=True)
    args = parser.parse_args()
    try:
        print(json.dumps(receipt_from_caches(args.gl_cache_root, args.gles_cache_root), sort_keys=True))
    except ReceiptError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
