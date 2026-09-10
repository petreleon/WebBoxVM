"""Fail-closed static metadata reifier for the pinned Vulkan Docs capture."""

from __future__ import annotations

import fnmatch
import hashlib
import re
import sys
import tomllib
from pathlib import Path, PurePosixPath

HERE = Path(__file__).resolve().parent
BASE = HERE.parent.parent
if str(HERE.parent) not in sys.path: sys.path.insert(0, str(HERE.parent))
for directory in (BASE / "02-actual-closure-identity", BASE / "03-capture-core-closure/01-observe-pinned-build-inputs",
                  BASE / "03-capture-core-closure/02-bind-core-input-scope"):
    sys.path.insert(0, str(directory))

from vulkan_docs_identity_build import GENERATION_ID, ROOT
from vulkan_docs_identity_members import DOCS_COMMIT, NAMESPACE
from vulkan_docs_identity_parse import F02
from vulkan_docs_member_ids import member_id as stable_member_id
from vulkan_docs_member_snapshot import SnapshotError, source_snapshot
from vulkan_docs_observer_model import PHASES
from vulkan_docs_observer_plan import SOURCE_TREE
from vulkan_docs_scope_bind import bind_capture

RAW_RECORD_FIELDS = frozenset(("kind", "selector", "revision", "immutable_url", "sha256", "bytes", "phase_roles"))
DERIVED_RECORD_FIELDS = frozenset(("kind", "selector", "generation_id", "sha256", "bytes", "phase_roles"))
RAW, DERIVED = "raw-observed-input", "derived-observed-input"
SPDX = re.compile(r"(?m)^.*SPDX-License-Identifier:\s*([^\r\n]+?)\s*$")
HEX = re.compile(r"^[0-9a-f]{64}$")


class MetadataError(ValueError): pass
class MetadataBlocked(MetadataError): pass
def reject(message: str) -> None: raise MetadataError(message)
def _selector(value: object) -> str:
    if not isinstance(value, str) or not value or not value.isascii() or "\\" in value:
        reject("metadata selector is invalid")
    parsed = PurePosixPath(value)
    if parsed.is_absolute() or str(parsed) != value or any(part in (".", "..") for part in parsed.parts):
        reject("metadata selector escapes its root")
    return value
def _digest(value: object) -> str:
    if not isinstance(value, str) or not HEX.fullmatch(value) or value == "0" * 64:
        reject("metadata digest is invalid")
    return value
def _phases(value: object) -> None:
    if not isinstance(value, list) or not value or any(not isinstance(item, str) or item not in PHASES for item in value) or len(set(value)) != len(value):
        reject("metadata phase roles are invalid")
def _payload(payloads: dict[str, bytes], selector: str) -> bytes:
    value = payloads.get(selector)
    if not isinstance(value, bytes) or not value:
        reject("metadata snapshot lacks a regular source member")
    return value
def _annotations(payloads: dict[str, bytes]) -> tuple[tuple[tuple[str, ...], str], ...]:
    try:
        value = tomllib.loads(_payload(payloads, "REUSE.toml").decode("utf-8"))
    except (UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
        reject(f"REUSE metadata cannot be read: {error}")
    rows = value.get("annotations", [])
    if not isinstance(rows, list): reject("REUSE annotations are invalid")
    result = []
    for row in rows:
        if not isinstance(row, dict) or row.get("precedence") != "aggregate":
            reject("REUSE annotation has no reviewed aggregate precedence")
        patterns = row.get("path")
        patterns = (patterns,) if isinstance(patterns, str) else tuple(patterns) if isinstance(patterns, list) else ()
        license_value = row.get("SPDX-License-Identifier")
        if not patterns or any(not isinstance(item, str) or not item.isascii() for item in patterns):
            reject("REUSE annotation paths are invalid")
        if not isinstance(license_value, str) or not license_value:
            reject("REUSE annotation lacks an SPDX license")
        result.append((patterns, license_value))
    return tuple(result)
def license_for(payloads: dict[str, bytes], selector: str, annotations: tuple[tuple[tuple[str, ...], str], ...]) -> str:
    text = _payload(payloads, selector).decode("utf-8", errors="ignore")
    headers = {item.strip() for item in SPDX.findall(text) if item.strip()}
    matches = {license_value for patterns, license_value in annotations if any(fnmatch.fnmatchcase(selector, pattern)
                                                                                for pattern in patterns)}
    if len(headers) > 1 or (headers and matches and headers != matches): reject("source license evidence is ambiguous")
    if headers: return next(iter(headers))
    if len(matches) != 1: reject("source license has no unique SPDX/REUSE authority")
    return next(iter(matches))
def member_id(kind: str, selector: str) -> str:
    try: return stable_member_id(kind, _selector(selector))
    except ValueError as error: reject(str(error))
def cache_name(kind: str, identifier: str, digest: str) -> str:
    details = {"raw-source-input": ("raw", "source"), "derived-source-input": ("derived", "derived")}.get(kind)
    if details is None: reject("metadata cache kind is invalid")
    return f"{NAMESPACE}/{details[0]}/{identifier}/{_digest(digest)}.{details[1]}"
def _raw_record(value: object) -> dict[str, object]:
    if not isinstance(value, dict) or set(value) != RAW_RECORD_FIELDS or value.get("kind") != RAW:
        reject("raw observed record has an invalid schema")
    selector = _selector(value.get("selector"))
    expected_url = f"https://raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/{DOCS_COMMIT}/{selector}"
    if value.get("revision") != DOCS_COMMIT or value.get("immutable_url") != expected_url:
        reject("raw observed record is not the pinned Vulkan Docs source")
    if not isinstance(value.get("bytes"), int) or isinstance(value["bytes"], bool) or not 0 < value["bytes"] <= F02.MAX_INPUT_BYTES:
        reject("raw observed record byte count is invalid")
    _digest(value.get("sha256")); _phases(value["phase_roles"])
    return value
def raw_member(record: object, payloads: dict[str, bytes], annotations: tuple[tuple[tuple[str, ...], str], ...]) -> dict[str, object]:
    value = _raw_record(record)
    selector, digest, count = value["selector"], value["sha256"], value["bytes"]
    assert isinstance(selector, str) and isinstance(digest, str) and isinstance(count, int)
    payload = _payload(payloads, selector)
    if (hashlib.sha256(payload).hexdigest(), len(payload)) != (digest, count):
        reject("raw observed record does not match its pinned source bytes")
    identifier = member_id("raw-source-input", selector)
    result = {"kind": "raw-source-input", "id": identifier, "source_family": "vulkan-docs",
              "immutable_url": value["immutable_url"], "revision": DOCS_COMMIT, "sha256": digest, "bytes": count,
              "license": license_for(payloads, selector, annotations),
              "local_cache": cache_name("raw-source-input", identifier, digest),
              "generated_code_role": "Docs source" if selector == "vkspec.adoc" else "captured raw Docs source input",
              "provenance": ROOT["provenance"] if selector == "vkspec.adoc" else f"captured pinned Vulkan-Docs source tree {DOCS_COMMIT}: {selector}",
              "selector": selector}
    if selector == "vkspec.adoc" and result != ROOT: reject("raw root metadata does not preserve the reviewed root identity")
    return result
def raw_members(observation: Path, artifact_root: Path, source: Path, run_id: str = "observer-a") -> tuple[dict[str, object], ...]:
    manifest = bind_capture(observation, artifact_root, run_id)
    if manifest.value["producer"]["source_tree_sha256"] != SOURCE_TREE[2]:
        reject("metadata capture does not retain the reviewed source-tree identity")
    try: payloads = source_snapshot(source)
    except SnapshotError as error: reject(str(error))
    annotations = _annotations(payloads)
    rows = tuple(raw_member(row, payloads, annotations) for row in manifest.value["raw_records"])
    if len({row["id"] for row in rows}) != len(rows) or len({row["local_cache"] for row in rows}) != len(rows):
        reject("raw metadata IDs or cache keys collide")
    return rows
def derived_requirements(records: object) -> tuple[dict[str, object], ...]:
    if not isinstance(records, list) or not records: reject("derived observed records are absent")
    result = []
    for row in records:
        if not isinstance(row, dict) or set(row) != DERIVED_RECORD_FIELDS or row.get("kind") != DERIVED:
            reject("derived observed record has an invalid schema")
        selector, digest = _selector(row.get("selector")), _digest(row.get("sha256")); _phases(row["phase_roles"])
        if row.get("generation_id") != GENERATION_ID or not isinstance(row.get("bytes"), int) or isinstance(row["bytes"], bool) or not 0 < row["bytes"] <= F02.MAX_INPUT_BYTES:
            reject("derived observed record has stale static identity")
        identifier = member_id("derived-source-input", selector)
        result.append({"kind": "derived-source-input", "id": identifier, "generation_id": GENERATION_ID,
                       "selector": selector, "sha256": digest, "bytes": row["bytes"],
                       "local_cache": cache_name("derived-source-input", identifier, digest)})
    if len({row["id"] for row in result}) != len(result): reject("derived metadata IDs collide")
    return tuple(result)
def require_derived_authority(requirements: tuple[dict[str, object], ...]) -> None:
    if not requirements: reject("derived metadata requirements are absent")
    raise MetadataBlocked("derived member license, role, provenance, and producer authority is absent")
def main() -> None:
    if len(sys.argv) != 4: raise SystemExit("usage: vulkan_docs_member_metadata.py OBSERVATION ARTIFACT_ROOT SOURCE_ROOT")
    observation, artifact_root, source = (Path(item) for item in sys.argv[1:])
    raw = raw_members(observation, artifact_root, source)
    requirements = derived_requirements(bind_capture(observation, artifact_root, "observer-a").value["derived_records"])
    print(f"STATIC-METADATA: {len(raw)} raw, {len(requirements)} derived")
    try: require_derived_authority(requirements)
    except MetadataBlocked as error:
        print(f"BLOCKED: {error}")
        raise SystemExit(2)


if __name__ == "__main__": main()
