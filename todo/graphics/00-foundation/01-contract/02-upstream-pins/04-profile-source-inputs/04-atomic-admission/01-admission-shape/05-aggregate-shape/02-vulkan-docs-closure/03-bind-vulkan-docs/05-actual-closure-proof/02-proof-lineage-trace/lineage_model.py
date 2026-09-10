"""Bounded vocabulary and state for proof-only derived-input lineage."""

from __future__ import annotations

from dataclasses import dataclass

RAW = "raw"
DERIVED = "derived"
PROCESS, EXEC, EXIT, READ, WRITE, RENAME, CLOSE, FINAL = "process", "exec", "exit", "read", "write", "rename", "close", "final"
KINDS = frozenset((PROCESS, EXEC, EXIT, READ, WRITE, RENAME, CLOSE, FINAL))
MAX_EVENTS = 200_000
MAX_TRACE_BYTES = 32 * 1024 * 1024
MAX_INPUT_BYTES = 8 * 1024 * 1024
TRACE_CONTRACT = "vulkan-docs-proof-lineage-trace-v2"
TRACE_STATUS = "proof-lineage-only-unadmitted"

PROCESS_FIELDS = frozenset(("kind", "sequence", "process_id", "instance_id", "parent_instance_id", "argv_sha256"))
EXEC_FIELDS = frozenset(("kind", "sequence", "process_id", "from_instance_id", "to_instance_id", "argv_sha256"))
EXIT_FIELDS = frozenset(("kind", "sequence", "process_id", "instance_id"))
READ_FIELDS = frozenset(("kind", "sequence", "instance_id", "input_id", "source", "selector", "sha256", "bytes"))
WRITE_FIELDS = frozenset(("kind", "sequence", "instance_id", "object_id", "path"))
RENAME_FIELDS = frozenset(("kind", "sequence", "instance_id", "object_id", "source_path", "target_path"))
CLOSE_FIELDS = frozenset(("kind", "sequence", "instance_id", "object_id"))
FINAL_FIELDS = frozenset(("kind", "sequence", "instance_id", "object_id", "input_id", "selector", "sha256", "bytes"))
RECEIPT_FIELDS = frozenset((
    "schema", "contract", "status", "observation_sha256", "manifest_sha256", "scope_identity_sha256", "run_id",
    "trace_source_sha256", "event_count", "events_sha256", "lineage", "lineage_sha256", "trace_sha256",
))


class LineageError(ValueError):
    """The proof trace is incomplete, ambiguous, or unsafe."""


def reject(message: str) -> None:
    raise LineageError(message)


@dataclass(frozen=True)
class Event:
    kind: str
    sequence: int
    value: dict[str, object]


@dataclass
class ProcessState:
    process_id: str
    inherited: list[str]
    direct: list[str]
    active: bool = True

    def dependencies(self) -> tuple[str, ...]:
        return tuple(dict.fromkeys((*self.inherited, *self.direct)))


@dataclass
class ObjectState:
    owner: str
    path: str
    dirty: bool = False
    closed: bool = False
    finalized: bool = False


@dataclass(frozen=True)
class Lineage:
    identifier: str
    selector: str
    digest: str
    byte_count: int
    writer_instance: str
    producer_ids: tuple[str, ...]
