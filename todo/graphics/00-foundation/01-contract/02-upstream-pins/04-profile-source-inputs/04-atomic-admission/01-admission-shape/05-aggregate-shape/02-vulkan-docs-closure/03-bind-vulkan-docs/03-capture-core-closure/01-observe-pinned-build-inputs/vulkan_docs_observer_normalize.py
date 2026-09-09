"""Normalize a pinned observer replay into immutable bounded input records."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

from vulkan_docs_observer_events import content, include, starts
from vulkan_docs_observer_model import DERIVED, MAX_RECORDS, PHASES, RAW, reject
from vulkan_docs_observer_parse import canonical, jsonl, source_limit

MAX_RENDERED_MEMBER = 16 * 1024 * 1024


def file_digest(path: Path, label: str) -> tuple[str, int]:
    if path.is_symlink() or not path.is_file():
        reject(f"{label} is not a regular observed file")
    try:
        data = path.read_bytes()
    except OSError as error:
        reject(f"{label} cannot be read: {error}")
    return hashlib.sha256(data).hexdigest(), source_limit(len(data), f"{label} bytes")


def tree(root: Path) -> tuple[int, int, str]:
    rows = []
    for path in sorted(candidate for candidate in root.rglob("*") if candidate.is_file()):
        if path.is_symlink():
            reject("observed tree contains a symlink")
        try:
            data = path.read_bytes()
        except OSError as error:
            reject(f"observed tree member cannot be read: {error}")
        if len(data) > MAX_RENDERED_MEMBER:
            reject("observed tree member exceeds the rendered-output bound")
        rows.append({"selector": path.relative_to(root).as_posix(), "sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)})
    if not rows:
        reject("observed tree is empty")
    rendered = json.dumps(rows, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode("utf-8")
    return len(rows), sum(row["bytes"] for row in rows), hashlib.sha256(rendered).hexdigest()


def record_path(kind: str, selector: str, source: Path, generated: Path, observed: tuple[int, int]) -> tuple[str, int]:
    root, relative = (source, selector) if kind == RAW else (generated, selector.removeprefix("generated/"))
    path = root / relative
    try:
        path.resolve().relative_to(root.resolve())
    except ValueError:
        reject("observer selector escapes its mounted root")
    digest, count = file_digest(path, "observed input")
    if (count, path.stat().st_mtime_ns // 1_000_000_000) != (observed[0], observed[1] // 1_000_000_000):
        reject(f"observed input changed after its content read: {selector}")
    return digest, count


def normalize(source: Path, generated: Path, io_trace: Path, include_trace: Path) -> dict[str, object]:
    io_rows = jsonl(io_trace)
    process_phases = starts(io_rows)
    records: dict[tuple[str, str], tuple[set[str], set[tuple[int, int]]]] = {}
    ignored = 0
    for row in io_rows:
        if row.get("kind") == "start":
            continue
        member, role, observed = content(row, process_phases)
        if member is None:
            ignored += 1
            continue
        roles, states = records.setdefault(member, (set(), set()))
        roles.add(role)
        states.add(observed)
    if not records or len(records) > MAX_RECORDS:
        reject("observer has an invalid unique input count")
    includes = []
    for row in jsonl(include_trace):
        member, line = include(row)
        captured = records.get(member)
        if captured is None or "asciidoctor" not in captured[0]:
            reject("resolved include has no matching Asciidoctor content read")
        includes.append({"kind": member[0], "selector": member[1], "line": line})
    result = []
    for (kind, selector), (roles, states) in sorted(records.items()):
        if not roles <= PHASES or len(states) != 1:
            reject("observer record has an invalid phase or mutable input state")
        digest, count = record_path(kind, selector, source, generated, next(iter(states)))
        result.append({"kind": kind, "selector": selector, "sha256": digest, "bytes": count, "phase_roles": sorted(roles)})
    return {
        "records": result, "includes": includes, "raw_count": sum(row["kind"] == RAW for row in result),
        "derived_count": sum(row["kind"] == DERIVED for row in result), "ignored_runtime_reads": ignored,
        "phase_counts": {name: sum(name in row["phase_roles"] for row in result) for name in sorted(PHASES)},
        "input_manifest_sha256": canonical(result, "webboxvm-graphics-vulkan-docs-observed-input-manifest-v1"),
        "include_identity_sha256": canonical(includes, "webboxvm-graphics-vulkan-docs-resolved-includes-v1"),
    }
