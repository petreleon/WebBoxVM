"""Bind a normalized trace to conservative derived-input lineage."""

from __future__ import annotations

from lineage_capture import SealedScope, checked
from lineage_model import CLOSE, DERIVED, EXEC, EXIT, FINAL, PROCESS, RAW, READ, RENAME, WRITE, Lineage, ObjectState, ProcessState, reject
from lineage_parse import canonical_member_id
from lineage_process import active, complete, execed, exited, start


def _member(value: dict[str, object]) -> tuple[str, str, str, int]:
    canonical_member_id(value["source"], value["input_id"], value["selector"], "lineage read")
    return value["source"], value["selector"], value["sha256"], value["bytes"]


def _remember(members: dict[str, tuple[str, str, str, int]], identifier: str, value) -> None:
    previous = members.get(identifier)
    if previous is not None and previous != value: reject("lineage input id changes its observed identity")
    members[identifier] = value


def _write(objects: dict[str, ObjectState], paths: dict[str, str], states: dict[str, ProcessState], value: dict[str, object]) -> None:
    active(states, value["instance_id"])
    identifier, current, owner = value["object_id"], value["path"], value["instance_id"]
    existing = objects.get(identifier)
    if existing is None:
        if current in paths: reject("lineage trace has multiple writers for one path")
        existing = ObjectState(owner, current); objects[identifier] = existing; paths[current] = identifier
    if existing.owner != owner or existing.path != current or existing.closed or existing.finalized:
        reject("lineage write has an ambiguous, closed, or final object")
    existing.dirty = True


def _rename(objects: dict[str, ObjectState], paths: dict[str, str], states: dict[str, ProcessState], value: dict[str, object]) -> None:
    active(states, value["instance_id"])
    object_id, source, target = value["object_id"], value["source_path"], value["target_path"]
    item = objects.get(object_id)
    if item is None or item.owner != value["instance_id"] or item.finalized or not item.dirty:
        reject("lineage rename lacks one successful writer")
    if item.path != source or paths.get(source) != object_id or target in paths:
        reject("lineage rename has an unsafe object/path lineage")
    paths.pop(source); paths[target] = object_id; item.path = target


def _close(objects: dict[str, ObjectState], paths: dict[str, str], states: dict[str, ProcessState], value: dict[str, object]) -> None:
    active(states, value["instance_id"])
    item = objects.get(value["object_id"])
    if item is None or item.owner != value["instance_id"] or item.finalized or item.closed or not item.dirty or paths.get(item.path) != value["object_id"]:
        reject("lineage close lacks one stable writer object")
    item.closed = True


def _final(objects, paths, states, members, completed, records, value) -> None:
    process = active(states, value["instance_id"])
    object_id, input_id, selector = value["object_id"], value["input_id"], value["selector"]
    canonical_member_id(DERIVED, input_id, selector, "lineage final")
    item = objects.get(object_id)
    if (item is None or item.owner != value["instance_id"] or item.finalized or not item.dirty or not item.closed
            or item.path != selector or paths.get(selector) != object_id):
        reject("lineage final lacks one closed, exact writer object")
    identity = (DERIVED, selector, value["sha256"], value["bytes"])
    _remember(members, input_id, identity)
    if input_id in completed: reject("lineage trace finalizes one input more than once")
    producers = process.dependencies()
    if not producers or input_id in producers: reject("lineage final has missing or cyclic producer inputs")
    records.append(Lineage(input_id, selector, value["sha256"], value["bytes"], value["instance_id"], producers))
    completed[input_id] = records[-1]
    item.finalized = True


def _acyclic(records: tuple[Lineage, ...]) -> None:
    ids = {item.identifier for item in records}
    waiting = {item.identifier: {source for source in item.producer_ids if source in ids} for item in records}
    ready, visited = [identifier for identifier, sources in waiting.items() if not sources], set()
    while ready:
        current = ready.pop()
        if current in visited: continue
        visited.add(current)
        for identifier, sources in waiting.items():
            sources.discard(current)
            if not sources and identifier not in visited: ready.append(identifier)
    if len(visited) != len(records): reject("lineage producer graph contains a cycle")


def bind(events, scope: SealedScope) -> tuple[Lineage, ...]:
    scope = checked(scope)
    states: dict[str, ProcessState] = {}; pids: dict[str, str] = {}; objects: dict[str, ObjectState] = {}
    paths: dict[str, str] = {}; members: dict[str, tuple[str, str, str, int]] = {}; completed: dict[str, Lineage] = {}
    records: list[Lineage] = []; raw_reads: set[str] = set(); derived_reads: set[str] = set()
    for event in events:
        value = event.value
        if event.kind == PROCESS: start(states, pids, value)
        elif event.kind == EXEC: execed(states, pids, value)
        elif event.kind == EXIT: exited(states, pids, value)
        elif event.kind == READ:
            process, identity = active(states, value["instance_id"]), _member(value)
            if value["source"] == RAW:
                if not scope.contains(value["input_id"], *identity[1:]): reject("lineage raw read escapes its sealed scope")
                raw_reads.add(value["input_id"])
            else:
                prior = completed.get(value["input_id"])
                if prior is None or (prior.selector, prior.digest, prior.byte_count) != identity[1:]:
                    reject("lineage reads a derived input before its exact finalization")
                derived_reads.add(value["input_id"])
            _remember(members, value["input_id"], identity)
            if value["input_id"] not in process.direct and value["input_id"] not in process.inherited:
                process.direct.append(value["input_id"])
        elif event.kind == WRITE: _write(objects, paths, states, value)
        elif event.kind == RENAME: _rename(objects, paths, states, value)
        elif event.kind == CLOSE: _close(objects, paths, states, value)
        elif event.kind == FINAL: _final(objects, paths, states, members, completed, records, value)
    if raw_reads != {item[0] for item in scope.raw_inputs}: reject("lineage raw reads do not exactly cover the sealed scope")
    if not records or set(completed) != derived_reads: reject("lineage finals do not exactly cover observed derived inputs")
    complete(pids)
    result = tuple(records); _acyclic(result)
    return result
