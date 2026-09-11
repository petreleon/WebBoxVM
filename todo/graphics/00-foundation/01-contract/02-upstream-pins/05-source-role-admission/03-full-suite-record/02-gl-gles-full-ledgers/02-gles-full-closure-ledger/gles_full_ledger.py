#!/usr/bin/env python3
"""Receipt the complete GLES root closure without inferring extension semantics."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

import gles_ledger_records as records

NORMAL = "--deqp-screen-rotation=unspecified --deqp-surface-width=64 --deqp-surface-height=64 --deqp-base-seed=1 --deqp-watchdog=disable"
WIDE = "--deqp-screen-rotation=unspecified --deqp-surface-width=113 --deqp-surface-height=47 --deqp-base-seed=2 --deqp-watchdog=disable"
FBO_WIDTH = "--deqp-screen-rotation=unspecified --deqp-surface-width=64 --deqp-surface-height=-1 --deqp-base-seed=3 --deqp-gl-config-name=rgba8888d24s8 --deqp-surface-type=fbo --deqp-watchdog=disable"
FBO_HEIGHT = "--deqp-screen-rotation=unspecified --deqp-surface-width=-1 --deqp-surface-height=64 --deqp-base-seed=3 --deqp-gl-config-name=rgba8888d24s8 --deqp-surface-type=fbo --deqp-watchdog=disable"


def config(name: str, command: str = NORMAL, first: bool = True, label: str = "khr-main") -> tuple[str, str, str, str, bool]:
    return name, command, label, "any", first


CONFIGURATIONS = (
    config("gles2-khr-main.txt"), config("gles2-khr-main.txt", first=False),
    config("gles3-khr-main.txt"), config("gles3-khr-main.txt", first=False),
    config("gles31-khr-main.txt"), config("gles31-khr-main.txt", first=False),
    config("gles32-khr-main.txt"), config("gles32-khr-main.txt", WIDE),
    config("gles32-khr-main.txt", FBO_WIDTH), config("gles32-khr-main.txt", FBO_HEIGHT),
    config("gles32-khr-main.txt", first=False), config("gles32-khr-main.txt", WIDE, False),
    config("gles32-khr-glesext.txt", label="khr-glesext"),
)
ATTRIBUTES = frozenset(("caseListFile", "commandLine", "name", "os", "useForFirstEGLConfig"))


class LedgerError(ValueError):
    """The GLES full-root observation is incomplete, altered, or misclassified."""


def reject(message: str) -> None:
    raise LedgerError(message)


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def flat_cases(data: bytes) -> tuple[str, ...]:
    if not data or not data.endswith(b"\n") or b"\r" in data:
        reject("GLES list must be nonempty LF-terminated bytes")
    try:
        cases = tuple(data[:-1].decode("utf-8").split("\n"))
    except UnicodeDecodeError as error:
        raise LedgerError(f"GLES list is not UTF-8: {error}") from error
    if any(not item or item != item.strip() or "\x00" in item for item in cases) or len(set(cases)) != len(cases):
        reject("GLES list has blank, padded, NUL, or duplicate cases")
    return cases


def configurations(data: bytes) -> tuple[tuple[str, str, str, str, bool], ...]:
    upper = data.upper()
    if b"<!DOCTYPE" in upper or b"<!ENTITY" in upper:
        reject("GLES XML must not declare a DTD or entity")
    try:
        root = ET.fromstring(data)
    except ET.ParseError as error:
        raise LedgerError(f"GLES XML cannot be parsed: {error}") from error
    if root.tag != "Mustpass" or root.attrib != {"version": "main"} or len(root) != 1:
        reject("GLES XML root is not the reviewed Mustpass descriptor")
    package = root[0]
    if package.tag != "TestPackage" or package.attrib != {"name": "Khronos Mustpass ES"}:
        reject("GLES XML package is not the reviewed descriptor")
    result = []
    for node in package:
        if node.tag != "Configuration" or set(node.attrib) != ATTRIBUTES:
            reject("GLES XML configuration has an unexpected shape")
        first = {"True": True, "False": False}.get(node.attrib["useForFirstEGLConfig"])
        if first is None:
            reject("GLES XML configuration has an invalid first-config flag")
        result.append((node.attrib["caseListFile"], node.attrib["commandLine"], node.attrib["name"],
                       node.attrib["os"], first))
    if tuple(result) != CONFIGURATIONS:
        reject("GLES XML configurations differ from the exact root order")
    return tuple(result)


def identity(record: dict[str, object]) -> dict[str, object]:
    return {name: record[name] for name in ("id", "revision", "sha256", "bytes", "license", "attribution",
                                              "authority", "producer", "scope")}


def ledger(root_data: bytes, payloads: dict[str, bytes]) -> dict[str, object]:
    value = records.catalog()
    try:
        records.validate_catalog(value)
    except records.RecordError as error:
        reject(str(error))
    root, members = value["records"][0], value["records"][1:]
    if (len(root_data), hashlib.sha256(root_data).hexdigest()) != (root["bytes"], root["sha256"]):
        reject("GLES root bytes do not match the pinned full-suite root")
    configs = configurations(root_data)
    if set(payloads) != {record["id"] for record in members}:
        reject("GLES member payloads do not match the exact root closure")
    observed = []
    for record, (_name, digest, size, count) in zip(members, records.MEMBERS):
        data = payloads[record["id"]]
        if (len(data), hashlib.sha256(data).hexdigest()) != (size, digest):
            reject("GLES member bytes do not match the pinned closure")
        cases = flat_cases(data)
        if len(cases) != count:
            reject("GLES member case sequence differs from the reviewed closure")
        observed.append({**identity(record), "case_count": len(cases), "case_sequence_sha256": digest,
                         "root_reference": "present"})
    body = {"schema": 1, "kind": "webboxvm-engineering-map", "authority": "WebBoxVM", "producer": "WebBoxVM",
            "claims": dict(records.NO_CLAIMS), "cts_executions": 0, "source_root": identity(root), "members": observed,
            "configurations": [{"case_list_file": item[0], "command_line": item[1], "name": item[2],
                                "os": item[3], "use_for_first_egl_config": item[4]} for item in configs],
            "glesext_boundary": {"id": records.member_id("gles32-khr-glesext.txt"), "root_reference": "present",
                                 "semantics": "not-classified-by-this-ledger"}}
    return {**body, "ledger_sha256": hashlib.sha256(canonical(body)).hexdigest()}


def validate_ledger(value: object, root_data: bytes, payloads: dict[str, bytes]) -> None:
    if value != ledger(root_data, payloads):
        reject("GLES local ledger differs from its exact source observation")


def fresh_cache(cache_root: Path):
    if not cache_root.is_absolute():
        reject("fresh cache root must be absolute")
    cache = records.roots.ExternalCache.from_path(cache_root, records.roots.repository_root(Path(__file__).resolve()))
    try:
        records.roots.require_fresh_cache(cache.root)
    except records.roots.FullSuiteError as error:
        reject(str(error))
    return cache


def refresh(cache_root: Path, timeout: float) -> dict[str, object]:
    if timeout <= 0:
        reject("timeout must be positive")
    value, cache = records.catalog(), fresh_cache(cache_root)
    try:
        records.validate_catalog(value)
        paths = {}
        for record in value["records"]:
            source = records.roots.source_input(record) if record["kind"] == "full-suite-root" else records.member_input(record)
            path, reused = records.roots.fetch_to_cache(cache, source, timeout)
            if reused:
                reject("fresh GLES refresh unexpectedly reused a source")
            paths[record["id"]] = path
        records.roots.verify_catalog(value, cache.root)
    except (records.RecordError, records.roots.ContractError, records.roots.RoleError,
            records.roots.FullSuiteError) as error:
        reject(str(error))
    return ledger(paths[records.ROOT_ID].read_bytes(), {key: path.read_bytes() for key, path in paths.items()
                                                         if key != records.ROOT_ID})


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
