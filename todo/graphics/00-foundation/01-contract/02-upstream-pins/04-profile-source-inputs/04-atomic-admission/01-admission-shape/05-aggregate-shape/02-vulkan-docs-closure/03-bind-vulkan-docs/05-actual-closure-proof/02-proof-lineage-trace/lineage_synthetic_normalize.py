#!/usr/bin/env python3
"""Normalize a collector-owned synthetic syscall wire trace.

Invariant: every live fd names one permitted path; every temporary writer has one owner and is closed before rename.
Each wire record changes O(1) pid, fd, or path state, bounded by the frozen event limit.
"""

from __future__ import annotations

from dataclasses import dataclass

from lineage_model import MAX_EVENTS, reject
from lineage_parse import derived_selector, digest, member_bytes, raw_selector, stable_member_id, trace_path

FIELDS = {
    "process": frozenset(("kind", "pid", "parent", "argv_sha256")),
    "exec": frozenset(("kind", "pid", "argv_sha256")), "exit": frozenset(("kind", "pid")),
    "open": frozenset(("kind", "pid", "fd", "path", "mode")),
    "read": frozenset(("kind", "pid", "fd")), "write": frozenset(("kind", "pid", "fd")),
    "close": frozenset(("kind", "pid", "fd")), "rename": frozenset(("kind", "pid", "old", "new")),
}


@dataclass
class Object:
    identifier: str
    owner: str
    path: str
    written: bool = False
    closed: bool = False
    finalized: bool = False


@dataclass
class Handle:
    mode: str
    path: str
    object_id: str | None = None


@dataclass
class Process:
    instance: str
    files: dict[int, Handle]


def row(value: object) -> dict[str, object]:
    kind = value.get("kind") if isinstance(value, dict) else None
    if not isinstance(kind, str) or kind not in FIELDS or set(value) != FIELDS[kind]:
        reject("synthetic collector wire record has an invalid schema")
    return value


def number(value: object, label: str, low: int, high: int) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or not low <= value <= high:
        reject(f"synthetic collector {label} is outside its bound")
    return value


def path(value: object, label: str) -> tuple[str, str]:
    if not isinstance(value, str) or not value.isascii() or "\x00" in value: reject(f"synthetic collector {label} is invalid")
    if value.startswith("/vulkan/"): return "raw", raw_selector(value[8:], label)
    if value.startswith("/work/temporary/"): return "temporary", trace_path("temporary/" + value[16:], label)
    if value.startswith("/work/generated/"): return "derived", derived_selector("generated/" + value[16:], label)
    reject(f"synthetic collector {label} escapes its roots")


def identity(values: object, selector: str) -> tuple[str, int]:
    if not isinstance(values, dict) or not isinstance(values.get(selector), dict) or set(values[selector]) != {"sha256", "bytes"}:
        reject("synthetic collector snapshot is missing or malformed")
    item = values[selector]
    return digest(item["sha256"], "synthetic collector snapshot sha256"), member_bytes(item["bytes"], "synthetic collector snapshot bytes")


def member(kind: str, selector: str) -> str:
    try: return stable_member_id(f"{kind}-source-input", selector)
    except ValueError as error: reject(str(error))


def normalize(wire: object, snapshots: object) -> list[dict[str, object]]:
    if not isinstance(wire, list) or not wire or len(wire) > MAX_EVENTS: reject("synthetic collector wire count is invalid")
    result: list[dict[str, object]] = []; pids: dict[int, Process] = {}; generations: dict[int, int] = {}
    objects: dict[str, Object] = {}; serial = 0

    def emit(kind: str, **value: object) -> None:
        if len(result) >= MAX_EVENTS: reject("synthetic collector normalized event count is invalid")
        result.append({"kind": kind, "sequence": len(result) + 1, **value})

    def current(value: dict[str, object]) -> tuple[int, Process]:
        pid = number(value["pid"], "pid", 1, 2**31 - 1); process = pids.get(pid)
        if process is None: reject("synthetic collector wire names an inactive pid")
        return pid, process

    for raw in wire:
        value = row(raw); kind = value["kind"]
        if kind == "process":
            pid = number(value["pid"], "pid", 1, 2**31 - 1); parent = value["parent"]
            if pid in pids or parent is not None and (not isinstance(parent, int) or isinstance(parent, bool) or parent not in pids): reject("synthetic collector process parent is invalid")
            if parent is not None and pids[parent].files: reject("synthetic collector fork has tracked live fds")
            generations[pid] = generations.get(pid, 0) + 1; instance = f"p-{pid}-{generations[pid]}"
            parent_instance = None if parent is None else pids[parent].instance
            pids[pid] = Process(instance, {}); emit("process", process_id=f"pid-{pid}", instance_id=instance,
                                                    parent_instance_id=parent_instance, argv_sha256=digest(value["argv_sha256"], "synthetic collector argv sha256"))
        elif kind == "exec":
            pid, process = current(value)
            if process.files: reject("synthetic collector exec has live fds")
            before = process.instance; generations[pid] += 1; process.instance = f"p-{pid}-{generations[pid]}"
            emit("exec", process_id=f"pid-{pid}", from_instance_id=before, to_instance_id=process.instance,
                 argv_sha256=digest(value["argv_sha256"], "synthetic collector argv sha256"))
        elif kind == "exit":
            pid, process = current(value)
            if process.files: reject("synthetic collector exit has live fds")
            emit("exit", process_id=f"pid-{pid}", instance_id=process.instance); pids.pop(pid)
        elif kind == "open":
            pid, process = current(value); fd = number(value["fd"], "fd", 0, 65535); mode = value["mode"]
            root, selected = path(value["path"], "open path")
            if (fd in process.files or not isinstance(mode, str) or mode not in {"read", "write"}
                    or (mode == "read") != (root in {"raw", "derived"}) or (mode == "write" and root != "temporary")):
                reject("synthetic collector open is outside its contract")
            serial += 1; object_id = f"o-{pid}-{fd}-{serial}" if mode == "write" else None
            process.files[fd] = Handle(mode, selected, object_id)
            if mode == "write":
                if selected in objects: reject("synthetic collector reopens a writer path")
                objects[selected] = Object(object_id, process.instance, selected)
        elif kind in {"read", "write", "close"}:
            _, process = current(value); fd = number(value["fd"], "fd", 0, 65535); handle = process.files.get(fd)
            if handle is None: reject("synthetic collector fd is not open")
            if kind == "read":
                root = "raw" if not handle.path.startswith("generated/") else "derived"
                if handle.mode != "read": reject("synthetic collector reads a writer fd")
                sha, size = identity(snapshots, handle.path); emit("read", instance_id=process.instance, input_id=member(root, handle.path),
                                                                   source=root, selector=handle.path, sha256=sha, bytes=size)
            elif kind == "write":
                if handle.mode != "write" or handle.object_id is None: reject("synthetic collector writes a reader fd")
                item = objects.get(handle.path)
                if item is None or item.owner != process.instance or item.closed: reject("synthetic collector writer state is invalid")
                item.written = True; emit("write", instance_id=process.instance, object_id=handle.object_id, path=handle.path)
            else:
                process.files.pop(fd)
                if handle.mode == "write":
                    item = objects.get(handle.path)
                    if item is None or not item.written or item.closed: reject("synthetic collector close has no writer")
                    item.closed = True; emit("close", instance_id=process.instance, object_id=handle.object_id)
        else:
            _, process = current(value); old_root, old = path(value["old"], "rename source"); new_root, new = path(value["new"], "rename target")
            item = objects.get(old)
            if old_root != "temporary" or new_root != "derived" or item is None or item.owner != process.instance or not item.closed or item.finalized or new in objects:
                reject("synthetic collector rename has invalid writer state")
            objects.pop(old); objects[new] = item; item.path = new; item.finalized = True
            emit("rename", instance_id=process.instance, object_id=item.identifier, source_path=old, target_path=new)
            sha, size = identity(snapshots, new); emit("final", instance_id=process.instance, object_id=item.identifier,
                                                        input_id=member("derived", new), selector=new, sha256=sha, bytes=size)
    if pids or any(not item.finalized for item in objects.values()): reject("synthetic collector wire did not terminate cleanly")
    return result
