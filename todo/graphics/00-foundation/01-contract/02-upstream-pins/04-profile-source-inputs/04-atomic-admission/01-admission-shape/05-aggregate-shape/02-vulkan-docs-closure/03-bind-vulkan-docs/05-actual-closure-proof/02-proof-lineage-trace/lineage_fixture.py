"""Small independent event traces and sealed scope fixtures for lineage tests."""

from __future__ import annotations

import copy
from dataclasses import replace

from lineage_capture import SealedScope
from lineage_contract import _receipt_with_scope

ROOT, LATE, SPEC, NEXT, ARGV, TRACE_SOURCE = "a" * 64, "f" * 64, "b" * 64, "c" * 64, "d" * 64, "e" * 64
LATE_ID = "raw-6c6174652e61646f63"
SPEC_ID, NEXT_ID = "derived-67656e6572617465642f73706563617474726962732e61646f63", "derived-67656e6572617465642f6e6578742e61646f63"
SCOPE = SealedScope("1" * 64, "2" * 64, "3" * 64, "observer-a", tuple(sorted((
    ("vulkan-14-spec", "vkspec.adoc", ROOT, 11), (LATE_ID, "late.adoc", LATE, 12),
))))


def scope() -> SealedScope:
    return SCOPE


def alternate_scope() -> SealedScope:
    return replace(SCOPE, manifest_sha256="4" * 64)


def events() -> list[dict[str, object]]:
    return [
        {"kind": "process", "sequence": 1, "process_id": "pid-make", "instance_id": "make", "parent_instance_id": None, "argv_sha256": ARGV},
        {"kind": "read", "sequence": 2, "instance_id": "make", "input_id": "vulkan-14-spec", "source": "raw", "selector": "vkspec.adoc", "sha256": ROOT, "bytes": 11},
        {"kind": "process", "sequence": 3, "process_id": "pid-shell", "instance_id": "shell", "parent_instance_id": "make", "argv_sha256": ARGV},
        {"kind": "read", "sequence": 4, "instance_id": "make", "input_id": LATE_ID, "source": "raw", "selector": "late.adoc", "sha256": LATE, "bytes": 12},
        {"kind": "write", "sequence": 5, "instance_id": "shell", "object_id": "object-spec", "path": "temporary/specattribs.tmp"},
        {"kind": "close", "sequence": 6, "instance_id": "shell", "object_id": "object-spec"},
        {"kind": "rename", "sequence": 7, "instance_id": "shell", "object_id": "object-spec", "source_path": "temporary/specattribs.tmp", "target_path": "generated/specattribs.adoc"},
        {"kind": "final", "sequence": 8, "instance_id": "shell", "object_id": "object-spec", "input_id": SPEC_ID, "selector": "generated/specattribs.adoc", "sha256": SPEC, "bytes": 19},
        {"kind": "read", "sequence": 9, "instance_id": "shell", "input_id": SPEC_ID, "source": "derived", "selector": "generated/specattribs.adoc", "sha256": SPEC, "bytes": 19},
        {"kind": "exec", "sequence": 10, "process_id": "pid-shell", "from_instance_id": "shell", "to_instance_id": "generator", "argv_sha256": ARGV},
        {"kind": "write", "sequence": 11, "instance_id": "generator", "object_id": "object-next", "path": "generated/next.adoc"},
        {"kind": "close", "sequence": 12, "instance_id": "generator", "object_id": "object-next"},
        {"kind": "final", "sequence": 13, "instance_id": "generator", "object_id": "object-next", "input_id": NEXT_ID, "selector": "generated/next.adoc", "sha256": NEXT, "bytes": 23},
        {"kind": "read", "sequence": 14, "instance_id": "generator", "input_id": NEXT_ID, "source": "derived", "selector": "generated/next.adoc", "sha256": NEXT, "bytes": 23},
        {"kind": "exit", "sequence": 15, "process_id": "pid-shell", "instance_id": "generator"},
        {"kind": "exit", "sequence": 16, "process_id": "pid-make", "instance_id": "make"},
    ]


def fresh_events() -> list[dict[str, object]]:
    return copy.deepcopy(events())


def sealed() -> dict[str, object]:
    return _receipt_with_scope(fresh_events(), TRACE_SOURCE, scope())
