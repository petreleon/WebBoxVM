#!/usr/bin/env python3
"""Validate the bounded, ordered V2 closure-ledger schema without fetching it."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path, PurePosixPath

HERE = Path(__file__).resolve().parent


def load_identity():
    spec = importlib.util.spec_from_file_location("f025_v2_canonical_identity", HERE / "canonical_suite_identity.py")
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load the reviewed V2 suite identity")
    value = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = value
    spec.loader.exec_module(value)
    return value


identity = load_identity()

HEX40 = re.compile(r"^[0-9a-f]{40}$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")
COMPONENT = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]*$")
PREFIX = ("external", "vulkancts", "mustpass", "main")
LEDGER_FIELDS = frozenset(("schema", "kind", "root_identity_sha256", "limits", "member_count",
                           "member_total_bytes", "ledger_sha256", "members"))
MEMBER_FIELDS = frozenset(("path", "parent_path", "revision", "blob_sha1", "sha256", "bytes"))
LIMITS = {"max_members": 128, "max_member_bytes": 64 * 1024 * 1024,
          "max_total_bytes": 512 * 1024 * 1024, "max_depth": 8, "max_ledger_json_bytes": 256 * 1024}


class LedgerError(ValueError):
    """A suite ledger is partial, unsafe, reordered, or outside the V2 resource policy."""


def reject(message: str) -> None:
    raise LedgerError(message)


def object_without_duplicates(pairs: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for key, item in pairs:
        if key in value:
            reject("ledger has a duplicate JSON key")
        value[key] = item
    return value


def document(path: Path) -> dict[str, object]:
    try:
        raw = path.read_bytes()
    except OSError as error:
        reject(f"ledger cannot be read: {error}")
    if len(raw) > LIMITS["max_ledger_json_bytes"]:
        reject("ledger exceeds its JSON byte limit")
    try:
        value = json.loads(raw.decode("utf-8"), object_pairs_hook=object_without_duplicates)
    except LedgerError:
        raise
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"ledger cannot be parsed: {error}")
    if not isinstance(value, dict):
        reject("ledger is not an object")
    return value


def digest(value: dict[str, object]) -> str:
    body = {key: value[key] for key in ("root_identity_sha256", "limits", "members")}
    return hashlib.sha256(json.dumps(body, sort_keys=True, separators=(",", ":")).encode("utf-8")).hexdigest()


def safe_path(value: object) -> str:
    if not isinstance(value, str) or not value:
        reject("ledger member path is empty")
    path = PurePosixPath(value)
    if (path.is_absolute() or str(path) != value or path.parts[:4] != PREFIX or path.suffix != ".txt"
            or any(not COMPONENT.fullmatch(part) for part in path.parts)):
        reject("ledger member path is unsafe or outside the VCTS mustpass root")
    return value


def member(value: object, seen: dict[str, int], commit: str, root_path: str) -> tuple[str, int]:
    if not isinstance(value, dict) or set(value) != MEMBER_FIELDS:
        reject("ledger member has an unexpected schema")
    path = safe_path(value["path"])
    if path == root_path:
        reject("ledger cannot repeat the separately pinned selector root")
    parent = value["parent_path"]
    if parent is not None and (not isinstance(parent, str) or parent not in seen):
        reject("ledger parent must precede its child")
    if path in seen:
        reject("ledger repeats a member path")
    revision, blob, sha256, byte_count = (value[key] for key in ("revision", "blob_sha1", "sha256", "bytes"))
    if (revision != commit or not all(isinstance(item, str) and pattern.fullmatch(item) and item != "0" * len(item)
                                      for item, pattern in ((blob, HEX40), (sha256, HEX64)))):
        reject("ledger member has a mixed revision or invalid blob identity")
    if type(byte_count) is not int or byte_count < 1 or byte_count > LIMITS["max_member_bytes"]:
        reject("ledger member has an invalid byte count")
    depth = 1 if parent is None else seen[parent] + 1
    if depth > LIMITS["max_depth"]:
        reject("ledger member exceeds the maximum recursive depth")
    seen[path] = depth
    return path, byte_count


@dataclass(frozen=True)
class SuiteLedger:
    member_count: int
    total_bytes: int
    digest: str


def validate(path: Path, identity_path: Path) -> SuiteLedger:
    try:
        root = identity.validate(identity_path)
    except identity.IdentityError as error:
        reject(f"ledger root identity failed: {error}")
    value = document(path)
    if (set(value) != LEDGER_FIELDS or type(value.get("schema")) is not int
            or value.get("schema") != 1 or value.get("kind") != "canonical-upstream-suite"):
        reject("ledger has an unexpected root schema")
    limits = value.get("limits")
    if (value.get("root_identity_sha256") != root.digest or not isinstance(limits, dict)
            or set(limits) != set(LIMITS) or any(type(limits[key]) is not int or limits[key] != LIMITS[key] for key in LIMITS)):
        reject("ledger does not bind the reviewed root and limits")
    members = value.get("members")
    if (not isinstance(members, list) or len(members) < root.direct_member_count
            or len(members) > LIMITS["max_members"]):
        reject("ledger has an invalid member list")
    seen: dict[str, int] = {}
    total = sum(member(item, seen, root.peeled_commit, root.root_path)[1] for item in members)
    direct = tuple(item["path"] for item in members[:root.direct_member_count] if isinstance(item, dict))
    if direct != root.direct_members or any(item["parent_path"] is not None for item in members[:root.direct_member_count]):
        reject("ledger direct members do not preserve the pinned root selector order")
    tail = members[root.direct_member_count:]
    if any(item["parent_path"] is None for item in tail if isinstance(item, dict)):
        reject("ledger recursive members must name a parent")
    if tuple((seen[item["path"]], item["parent_path"], item["path"]) for item in tail if isinstance(item, dict)) != tuple(sorted((seen[item["path"]], item["parent_path"], item["path"]) for item in tail if isinstance(item, dict))):
        reject("ledger recursive members are not in canonical order")
    if total > LIMITS["max_total_bytes"]:
        reject("ledger exceeds the aggregate byte limit")
    if (type(value.get("member_count")) is not int or type(value.get("member_total_bytes")) is not int
            or value.get("member_count") != len(members) or value.get("member_total_bytes") != total):
        reject("ledger has a stale member aggregate")
    claimed = value.get("ledger_sha256")
    if not isinstance(claimed, str) or not HEX64.fullmatch(claimed) or claimed != digest(value):
        reject("ledger has a stale canonical digest")
    return SuiteLedger(len(members), total, claimed)


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit("usage: canonical_suite_ledger.py IDENTITY.json LEDGER.json")
    try:
        ledger = validate(Path(sys.argv[2]), Path(sys.argv[1]))
    except LedgerError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"LEDGER: {ledger.member_count} members {ledger.total_bytes} bytes {ledger.digest}")


if __name__ == "__main__":
    main()
