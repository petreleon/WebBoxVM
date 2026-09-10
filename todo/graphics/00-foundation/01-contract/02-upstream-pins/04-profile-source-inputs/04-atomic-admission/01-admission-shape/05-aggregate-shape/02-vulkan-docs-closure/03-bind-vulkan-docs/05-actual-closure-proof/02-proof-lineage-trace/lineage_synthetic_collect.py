#!/usr/bin/env python3
"""Consume fixture wire and exact post-exit snapshots solely from the collector pipe."""

from __future__ import annotations

import hashlib
import json
import re
from dataclasses import dataclass

from lineage_bind import bind
from lineage_capture import SealedScope, raw_member_id
from lineage_events import parse
from lineage_model import MAX_TRACE_BYTES, reject
from lineage_parse import nonfinite, pairs
from lineage_synthetic_normalize import normalize

TERMINAL = frozenset(("kind", "status"))
SNAPSHOT = frozenset(("kind", "path", "hex"))
HEX = re.compile(r"[0-9a-f]{6}")
KINDS = ("process", "exec", "open", "read", "close", "open", "write", "close", "rename", "open", "read", "close", "process", "exec", "exit", "exit")
ARGV = "797b8f932ac58c625cfb33c986ba1e4b5eb471598cb3847781a4eae40f8c21fb"


@dataclass(frozen=True)
class SyntheticObservation:
    contract: str
    status: str
    records: tuple


def snapshot(value: object) -> tuple[str, dict[str, object]]:
    if not isinstance(value, dict) or set(value) != SNAPSHOT or not isinstance(value.get("path"), str) or not isinstance(value.get("hex"), str):
        reject("synthetic collector snapshot row is malformed")
    selector = {"/vulkan/raw.adoc": "raw.adoc", "/work/generated/out.adoc": "generated/out.adoc"}.get(value["path"])
    if selector is None or not HEX.fullmatch(value["hex"]): reject("synthetic collector snapshot escapes its fixture paths")
    payload = bytes.fromhex(value["hex"])
    return selector, {"sha256": hashlib.sha256(payload).hexdigest(), "bytes": len(payload)}


def decoded(value: object) -> tuple[list[dict[str, object]], dict[str, dict[str, object]]]:
    if not isinstance(value, bytes) or not value or len(value) > MAX_TRACE_BYTES: reject("synthetic collector pipe size is invalid")
    rows = []
    for line in value.splitlines():
        try: row = json.loads(line.decode("utf-8"), object_pairs_hook=pairs, parse_constant=nonfinite)
        except (UnicodeDecodeError, json.JSONDecodeError, ValueError) as error: reject(f"synthetic collector pipe is not JSONL: {error}")
        if not isinstance(row, dict): reject("synthetic collector pipe row is not an object")
        rows.append(row)
    if len(rows) < 4 or any(row.get("kind") == "terminal" for row in rows[:-1]): reject("synthetic collector pipe lacks one terminal row")
    if set(rows[-1]) != TERMINAL or rows[-1] != {"kind": "terminal", "status": "observed-unadmitted"}:
        reject("synthetic collector did not complete an unadmitted fixture run")
    wire, snapshots, emitted = [], {}, False
    for row in rows[:-1]:
        if row.get("kind") == "snapshot":
            emitted = True; selector, identity = snapshot(row)
            if selector in snapshots: reject("synthetic collector repeats a snapshot")
            snapshots[selector] = identity
        elif emitted: reject("synthetic collector emits wire after a snapshot")
        else: wire.append(row)
    if set(snapshots) != {"raw.adoc", "generated/out.adoc"} or snapshots["raw.adoc"] != snapshots["generated/out.adoc"]:
        reject("synthetic collector snapshots do not prove the fixture copy")
    return wire, snapshots


def fixture_wire(wire: list[dict[str, object]]) -> None:
    if tuple(row.get("kind") for row in wire) != KINDS: reject("synthetic collector wire is not the one bounded fixture lifecycle")
    parent, child = wire[0], wire[12]
    if type(parent.get("pid")) is not int or parent["pid"] < 1 or parent.get("parent") is not None or child.get("parent") != parent["pid"]:
        reject("synthetic collector wire has an invalid fixture parent")
    if type(child.get("pid")) is not int or child["pid"] < 1 or child["pid"] == parent["pid"]:
        reject("synthetic collector wire has an invalid fixture child")
    if any(row.get("pid") != parent["pid"] for row in wire[1:12]) or any(row.get("pid") != child.get("pid") for row in wire[13:15]):
        reject("synthetic collector wire has an invalid fixture process order")
    if wire[15].get("pid") != parent["pid"]: reject("synthetic collector wire exits its fixture parent incorrectly")
    if any(row.get("argv_sha256") != ARGV for row in (wire[0], wire[1], wire[12], wire[13])):
        reject("synthetic collector wire has an invalid fixture sentinel")
    opens = ((2, 3, "/vulkan/raw.adoc", "read"), (5, 3, "/work/temporary/out.tmp", "write"), (9, 3, "/work/generated/out.adoc", "read"))
    if tuple((index, wire[index].get("fd"), wire[index].get("path"), wire[index].get("mode")) for index, _, _, _ in opens) != opens:
        reject("synthetic collector wire has a noncanonical fixture open")
    if any(wire[index].get("fd") != 3 for index in (3, 4, 6, 7, 10, 11)):
        reject("synthetic collector wire has a noncanonical fixture descriptor")
    if (wire[8].get("old"), wire[8].get("new")) != ("/work/temporary/out.tmp", "/work/generated/out.adoc"):
        reject("synthetic collector wire has a noncanonical fixture rename")


def fixture_scope(raw: dict[str, object]) -> SealedScope:
    anchor = lambda label: hashlib.sha256(label.encode("ascii")).hexdigest()
    return SealedScope(anchor("fixture-observation-v1"), anchor("fixture-manifest-v1"), anchor("fixture-scope-v1"), "fixture-ptrace-v1",
                       ((raw_member_id("raw.adoc"), "raw.adoc", raw["sha256"], raw["bytes"]),))


def collect(pipe: object):
    wire, snapshots = decoded(pipe)
    fixture_wire(wire)
    return SyntheticObservation("webboxvm-graphics-synthetic-ptrace-observation-v1", "observed-unadmitted",
                                bind(parse(normalize(wire, snapshots)), fixture_scope(snapshots["raw.adoc"])))
