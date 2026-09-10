"""Parse the proof-only, syscall-normalized lineage event grammar."""

from __future__ import annotations

import json

from lineage_model import (
    CLOSE, CLOSE_FIELDS, DERIVED, EXEC, EXEC_FIELDS, EXIT, EXIT_FIELDS, FINAL, FINAL_FIELDS, KINDS, MAX_EVENTS,
    MAX_TRACE_BYTES, PROCESS, PROCESS_FIELDS, RAW, READ, READ_FIELDS, RENAME, RENAME_FIELDS, WRITE, WRITE_FIELDS,
    Event, reject,
)
from lineage_parse import canonical_member_id, derived_selector, digest, exact, identifier, member_bytes, positive, raw_selector, trace_path

FIELDS = {PROCESS: PROCESS_FIELDS, EXEC: EXEC_FIELDS, EXIT: EXIT_FIELDS, READ: READ_FIELDS, WRITE: WRITE_FIELDS,
          RENAME: RENAME_FIELDS, CLOSE: CLOSE_FIELDS, FINAL: FINAL_FIELDS}


def _process(value: dict[str, object]) -> None:
    identifier(value["process_id"], "lineage process id")
    identifier(value["instance_id"], "lineage process instance id")
    parent = value["parent_instance_id"]
    if parent is not None: identifier(parent, "lineage parent instance id")
    digest(value["argv_sha256"], "lineage process argv sha256")


def _exec(value: dict[str, object]) -> None:
    identifier(value["process_id"], "lineage exec process id")
    identifier(value["from_instance_id"], "lineage exec source instance id")
    identifier(value["to_instance_id"], "lineage exec target instance id")
    digest(value["argv_sha256"], "lineage exec argv sha256")


def _exit(value: dict[str, object]) -> None:
    identifier(value["process_id"], "lineage exit process id")
    identifier(value["instance_id"], "lineage exit instance id")


def _read(value: dict[str, object]) -> None:
    identifier(value["instance_id"], "lineage read instance id")
    source = value["source"]
    if source not in (RAW, DERIVED): reject("lineage read has an invalid source kind")
    selector = (raw_selector if source == RAW else derived_selector)(value["selector"], "lineage read selector")
    canonical_member_id(source, value["input_id"], selector, "lineage read")
    digest(value["sha256"], "lineage read sha256")
    member_bytes(value["bytes"], "lineage read bytes")


def _object(value: dict[str, object], label: str) -> None:
    identifier(value["instance_id"], f"lineage {label} instance id")
    identifier(value["object_id"], f"lineage {label} object id")


def _write(value: dict[str, object]) -> None:
    _object(value, "write"); trace_path(value["path"], "lineage write path")


def _rename(value: dict[str, object]) -> None:
    _object(value, "rename")
    trace_path(value["source_path"], "lineage rename source path")
    trace_path(value["target_path"], "lineage rename target path")


def _close(value: dict[str, object]) -> None:
    _object(value, "close")


def _final(value: dict[str, object]) -> None:
    _object(value, "final")
    selector = derived_selector(value["selector"], "lineage final selector")
    canonical_member_id(DERIVED, value["input_id"], selector, "lineage final")
    digest(value["sha256"], "lineage final sha256")
    member_bytes(value["bytes"], "lineage final bytes")


def _validate(value: object) -> Event:
    if not isinstance(value, dict) or value.get("kind") not in KINDS: reject("lineage event has an unknown kind")
    kind = value["kind"]
    row = exact(value, FIELDS[kind], "lineage event")
    sequence = positive(row["sequence"], "lineage event sequence", MAX_EVENTS * 8)
    {PROCESS: _process, EXEC: _exec, EXIT: _exit, READ: _read, WRITE: _write, RENAME: _rename,
     CLOSE: _close, FINAL: _final}[kind](row)
    return Event(kind, sequence, dict(row))


def parse(value: object) -> tuple[Event, ...]:
    if not isinstance(value, list) or not value or len(value) > MAX_EVENTS: reject("lineage trace has an invalid event count")
    total, previous, result = 0, 0, []
    for item in value:
        try: total += len(json.dumps(item, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode("utf-8"))
        except (TypeError, ValueError) as error: reject(f"lineage event cannot be encoded: {error}")
        event = _validate(item)
        if event.sequence <= previous: reject("lineage events are not strictly ordered")
        previous = event.sequence
        result.append(event)
    if total > MAX_TRACE_BYTES: reject("lineage trace exceeds its explicit byte bound")
    return tuple(result)
