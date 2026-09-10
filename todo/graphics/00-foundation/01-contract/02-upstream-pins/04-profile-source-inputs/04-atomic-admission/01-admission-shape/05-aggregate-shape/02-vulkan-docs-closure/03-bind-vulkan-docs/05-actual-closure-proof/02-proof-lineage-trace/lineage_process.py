"""Fail-closed process-instance lifecycle for normalized lineage events."""

from __future__ import annotations

from lineage_model import ProcessState, reject


def active(states: dict[str, ProcessState], instance: object) -> ProcessState:
    result = states.get(instance)
    if result is None or not result.active:
        reject("lineage event names an inactive or unknown process instance")
    return result


def start(states: dict[str, ProcessState], pids: dict[str, str], value: dict[str, object]) -> None:
    instance, process_id, parent = value["instance_id"], value["process_id"], value["parent_instance_id"]
    if instance in states or process_id in pids:
        reject("lineage trace repeats an active process identity")
    inherited = [] if parent is None else list(active(states, parent).dependencies())
    states[instance] = ProcessState(process_id, inherited, [])
    pids[process_id] = instance


def execed(states: dict[str, ProcessState], pids: dict[str, str], value: dict[str, object]) -> None:
    before, after, process_id = value["from_instance_id"], value["to_instance_id"], value["process_id"]
    previous = active(states, before)
    if after in states or previous.process_id != process_id or pids.get(process_id) != before:
        reject("lineage exec has an ambiguous process identity")
    previous.active = False
    states[after] = ProcessState(process_id, list(previous.dependencies()), [])
    pids[process_id] = after


def exited(states: dict[str, ProcessState], pids: dict[str, str], value: dict[str, object]) -> None:
    instance, process_id = value["instance_id"], value["process_id"]
    current = active(states, instance)
    if current.process_id != process_id or pids.get(process_id) != instance:
        reject("lineage exit has an ambiguous process identity")
    current.active = False
    pids.pop(process_id)


def complete(pids: dict[str, str]) -> None:
    if pids: reject("lineage trace has active process instances at its end")
