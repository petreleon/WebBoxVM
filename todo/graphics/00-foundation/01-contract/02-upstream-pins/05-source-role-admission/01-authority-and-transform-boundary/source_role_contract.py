#!/usr/bin/env python3
"""Fail-closed F02.5 catalog graph validation."""

from __future__ import annotations

from source_role_records import MAX_LOCAL_BYTES, RoleError, digest, fields, identity, positive, record_shape, reject, upstream_identity


def input_ids(record: dict[str, object], records: dict[str, dict[str, object]]) -> list[str]:
    value = record["inputs"]
    if not isinstance(value, list) or not value:
        reject("transform inputs must be a nonempty list")
    result = []
    for item in value:
        kind = item.get("kind") if isinstance(item, dict) else None
        expected = frozenset(("id", "kind", "sha256", "revision")) if kind in {"upstream-source", "full-suite-root"} else frozenset(("id", "kind", "sha256"))
        item = fields(item, expected, "transform input")
        identifier = identity(item["id"], "transform input id")
        digest(item["sha256"], "transform input sha256")
        target = records.get(identifier)
        if identifier in result or target is None or target["kind"] != kind or target["sha256"] != item["sha256"]:
            reject("transform input is duplicated, unresolved, or mismatched")
        if kind in {"upstream-source", "full-suite-root"} and target["revision"] != item["revision"]:
            reject("transform input has a mismatched revision")
        result.append(identifier)
    return result


def bind_command(record: dict[str, object], inputs: list[str]) -> None:
    mode = record["scope"].removeprefix("webboxvm-")
    flags: tuple[str, ...] = ()
    if record["scope"] == "webboxvm-byte-preserving-shard":
        piece = record["shard"]
        flags = (f"--shard-index={piece['index']}", f"--shard-count={piece['count']}",
                 f"--shard-offset={piece['offset']}", f"--shard-bytes={record['bytes']}")
    expected = ["webboxvm-source-builder", f"--mode={mode}", *flags, *(f"@input:{item}" for item in inputs),
                f"@output:{record['id']}"]
    if record["command"] != expected:
        reject("command must bind its declared inputs and output canonically")


def member_anchor(record: dict[str, object], records: dict[str, dict[str, object]]) -> None:
    root = records.get(record["suite_root_id"])
    _member_revision, member_repo, _member_path = upstream_identity(record)
    if root is None or root["kind"] != "full-suite-root" or root["revision"] != record["revision"]:
        reject("suite member does not anchor to its full suite root")
    _root_revision, root_repo, _root_path = upstream_identity(root)
    if member_repo != root_repo:
        reject("suite member and full suite root use different repositories")


def shard(record: dict[str, object], records: dict[str, dict[str, object]], inputs: list[str]) -> None:
    value = fields(record["shard"], frozenset(("source_id", "source_sha256", "source_bytes", "index", "count", "offset", "reassembled_sha256")), "shard")
    source_id = identity(value["source_id"], "shard source_id")
    digest(value["source_sha256"], "shard source_sha256")
    digest(value["reassembled_sha256"], "shard reassembled_sha256")
    count, index, offset = positive(value["count"], "shard count"), value["index"], value["offset"]
    if not isinstance(index, int) or isinstance(index, bool) or not isinstance(offset, int) or isinstance(offset, bool) or not 0 <= index < count or offset < 0:
        reject("shard index or offset is invalid")
    source = records.get(source_id)
    if inputs != [source_id] or source is None or source["kind"] != "upstream-source" or source["scope"] != "suite-member":
        reject("shard must name one upstream suite member")
    if (source["sha256"], source["bytes"]) != (value["source_sha256"], positive(value["source_bytes"], "shard source_bytes")) or value["reassembled_sha256"] != source["sha256"]:
        reject("shard does not identify its whole upstream member")


def no_cycles(records: dict[str, dict[str, object]]) -> None:
    state: dict[str, int] = {}
    def visit(identifier: str) -> None:
        state[identifier] = 1
        for item in records[identifier].get("inputs", []):
            child = item["id"] if item["kind"] == "webboxvm-transform" else None
            if child and state.get(child) == 1:
                reject("transform input graph has a cycle")
            if child and state.get(child, 0) == 0:
                visit(child)
        state[identifier] = 2
    for identifier, record in records.items():
        if record["kind"] == "webboxvm-transform" and state.get(identifier, 0) == 0:
            visit(identifier)


def complete_shards(records: dict[str, dict[str, object]]) -> None:
    groups: dict[str, list[dict[str, object]]] = {}
    for record in records.values():
        if record["kind"] == "webboxvm-transform" and record["scope"] == "webboxvm-byte-preserving-shard":
            groups.setdefault(record["shard"]["source_id"], []).append(record)
    for pieces in groups.values():
        first, count, offset = pieces[0]["shard"], pieces[0]["shard"]["count"], 0
        if len(pieces) != count or [item["shard"]["index"] for item in pieces] != list(range(count)):
            reject("shard catalog is incomplete or reordered")
        for piece in pieces:
            if piece["shard"]["count"] != count or piece["shard"]["offset"] != offset:
                reject("shard catalog has inconsistent offsets")
            offset += piece["bytes"]
        if offset != first["source_bytes"]:
            reject("shard catalog does not cover the upstream member")


def validate_catalog(value: object) -> tuple[str, ...]:
    catalog = fields(value, frozenset(("schema", "records")), "catalog")
    if type(catalog["schema"]) is not int or catalog["schema"] != 1 or not isinstance(catalog["records"], list) or not catalog["records"]:
        reject("catalog must be schema version 1 with records")
    records: dict[str, dict[str, object]] = {}
    for record in catalog["records"]:
        identifier = record_shape(record)
        if identifier in records:
            reject("catalog has duplicate record ids")
        records[identifier] = record
    for record in records.values():
        if record["kind"] == "upstream-source" and record["scope"] == "suite-member":
            member_anchor(record, records)
        if record["kind"] == "webboxvm-transform":
            inputs = input_ids(record, records)
            if record["scope"] == "webboxvm-byte-preserving-shard":
                shard(record, records, inputs)
            bind_command(record, inputs)
    no_cycles(records)
    complete_shards(records)
    return tuple(records)
