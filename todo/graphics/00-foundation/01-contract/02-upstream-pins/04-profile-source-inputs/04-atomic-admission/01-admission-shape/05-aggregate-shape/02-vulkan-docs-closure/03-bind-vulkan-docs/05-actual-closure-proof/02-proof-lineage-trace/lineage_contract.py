#!/usr/bin/env python3
"""Self-hash a proof-only lineage receipt without admitting a Docs closure."""

from __future__ import annotations

import sys
from pathlib import Path

from lineage_bind import bind
from lineage_capture import SealedScope, capture_scope, checked
from lineage_events import parse
from lineage_model import MAX_EVENTS, RECEIPT_FIELDS, TRACE_CONTRACT, TRACE_STATUS, Lineage, reject
from lineage_parse import canonical, digest, document, exact, identifier, positive


def rows(records: tuple[Lineage, ...]) -> list[dict[str, object]]:
    return [{"input_id": item.identifier, "selector": item.selector, "sha256": item.digest, "bytes": item.byte_count,
             "writer_instance_id": item.writer_instance, "producer_input_ids": list(item.producer_ids)} for item in records]


def event_rows(events) -> list[dict[str, object]]:
    return [event.value for event in events]


def _capture(observation: object, artifact_root: object, run_id: object) -> SealedScope:
    if isinstance(observation, SealedScope) or not isinstance(observation, Path) or not isinstance(artifact_root, Path):
        reject("lineage receipt requires capture paths, not a caller-supplied scope")
    return capture_scope(observation, artifact_root, run_id)


def _receipt_with_scope(value: object, trace_source: object, scope: SealedScope) -> dict[str, object]:
    scope = checked(scope)
    source, events = digest(trace_source, "lineage trace source sha256"), parse(value)
    lineage = rows(bind(events, scope))
    result = {"schema": 1, "contract": TRACE_CONTRACT, "status": TRACE_STATUS,
              "observation_sha256": scope.observation_sha256, "manifest_sha256": scope.manifest_sha256,
              "scope_identity_sha256": scope.scope_identity_sha256, "run_id": scope.run_id,
              "trace_source_sha256": source, "event_count": len(events),
              "events_sha256": canonical(event_rows(events), "webboxvm-graphics-lineage-events-v2"), "lineage": lineage,
              "lineage_sha256": canonical(lineage, "webboxvm-graphics-lineage-records-v2")}
    result["trace_sha256"] = canonical(result, "webboxvm-graphics-lineage-trace-receipt-v2")
    return result


def receipt(value: object, trace_source: object, observation: object, artifact_root: object, run_id: object) -> dict[str, object]:
    return _receipt_with_scope(value, trace_source, _capture(observation, artifact_root, run_id))


def _verify_with_scope(receipt_value: object, events_value: object, scope: SealedScope) -> dict[str, object]:
    scope = checked(scope)
    value = exact(receipt_value, RECEIPT_FIELDS, "lineage receipt")
    if tuple(value.get(name) for name in ("schema", "contract", "status")) != (1, TRACE_CONTRACT, TRACE_STATUS):
        reject("lineage receipt has an active or unknown state")
    for name in ("observation_sha256", "manifest_sha256", "scope_identity_sha256", "trace_source_sha256", "events_sha256",
                 "lineage_sha256", "trace_sha256"):
        digest(value.get(name), f"lineage receipt {name}")
    run_id = identifier(value.get("run_id"), "lineage receipt run id")
    if run_id != scope.run_id or tuple(value.get(name) for name in ("observation_sha256", "manifest_sha256", "scope_identity_sha256")) != (
            scope.observation_sha256, scope.manifest_sha256, scope.scope_identity_sha256):
        reject("lineage receipt does not bind its sealed capture")
    positive(value.get("event_count"), "lineage receipt event count", MAX_EVENTS)
    expected = _receipt_with_scope(events_value, value["trace_source_sha256"], scope)
    if value != expected: reject("lineage receipt does not exactly bind its events")
    return expected


def verify(receipt_value: object, events_value: object, observation: object, artifact_root: object, run_id: object) -> dict[str, object]:
    return _verify_with_scope(receipt_value, events_value, _capture(observation, artifact_root, run_id))


def main() -> None:
    if len(sys.argv) != 6: raise SystemExit("usage: lineage_contract.py EVENTS.json TRACE_SOURCE OBSERVATION ARTIFACT_ROOT RUN_ID")
    try:
        result = receipt(document(Path(sys.argv[1])), sys.argv[2], Path(sys.argv[3]), Path(sys.argv[4]), sys.argv[5])
    except Exception as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"LINEAGE: proof-only-unadmitted, {len(result['lineage'])} derived inputs, 0 cutover-ready")


if __name__ == "__main__": main()
