#!/usr/bin/env python3
"""Bind one replayable GLES capture without changing active consumers."""

from __future__ import annotations

import argparse, hashlib, importlib.util, json, os, stat, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CAPTURE_DIR = HERE.parents[1] / "02-capture-and-replay"
INTEGRATION_DIR = HERE.parents[2] / "01-successor-integration"
RECORD = HERE / "sealed_capture_binding.json"
MAX_BYTES = 16 * 1024


class BindingError(ValueError):
    """A capture binding is incomplete, stale, or falsely promoting."""


def reject(message: str) -> None:
    raise BindingError(message)


def reviewed(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load reviewed module {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


PLAN = reviewed("gles_capture_plan", CAPTURE_DIR / "gles_capture_plan.py")
MARKER = reviewed("gles_capture_marker", CAPTURE_DIR / "gles_capture_marker.py")
RUN = reviewed("gles_capture", CAPTURE_DIR / "gles_capture.py")
INTEGRATION = reviewed("f025_sealed_capture_integration", INTEGRATION_DIR / "multi_suite_successor_integration.py")


def pairs(rows: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for key, item in rows:
        if key in value:
            reject("capture binding has a duplicate JSON key")
        value[key] = item
    return value


def raw(path: Path) -> bytes:
    try:
        descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    except OSError as error:
        reject(f"capture binding cannot be safely read: {error}")
    try:
        info = os.fstat(descriptor)
        if not stat.S_ISREG(info.st_mode) or info.st_size > MAX_BYTES:
            reject("capture binding is not a bounded regular file")
        value = os.read(descriptor, MAX_BYTES + 1)
    finally:
        os.close(descriptor)
    if len(value) != info.st_size or len(value) > MAX_BYTES:
        reject("capture binding changed during bounded read")
    return value


def document(path: Path) -> dict[str, object]:
    try:
        value = json.loads(raw(path).decode("utf-8"), object_pairs_hook=pairs)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"capture binding is invalid JSON: {error}")
    if not isinstance(value, dict):
        reject("capture binding is not an object")
    return value


def digest(value: dict[str, object]) -> str:
    body = {key: item for key, item in value.items() if key != "binding_sha256"}
    return hashlib.sha256(json.dumps(body, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def alike(left: object, right: object) -> bool:
    if type(left) is not type(right):
        return False
    if isinstance(left, dict):
        return set(left) == set(right) and all(alike(left[key], right[key]) for key in left)
    if isinstance(left, list):
        return len(left) == len(right) and all(alike(one, two) for one, two in zip(left, right))
    return left == right


def inputs():
    try:
        plan, integration = PLAN.plan(), INTEGRATION.validate()
    except Exception as error:
        reject(f"capture predecessor is invalid: {error}")
    gles = integration.get("gles")
    expected = (plan.root.identifier, plan.root.sha256, "gles-cts", 4, 12, True, False, True)
    actual = (gles.get("candidate_id"), gles.get("candidate_sha256"), gles.get("family"),
              gles.get("core_member_count"), gles.get("configuration_count"),
              gles.get("extension_exclusion_required"), gles.get("successor_closure_ready"),
              gles.get("fresh_capture_required")) if isinstance(gles, dict) else ()
    if actual != expected or integration.get("effects") != {key: False for key in INTEGRATION.EFFECTS}:
        reject("capture predecessor does not retain the exact unadmitted GLES boundary")
    return plan, integration


def build() -> dict[str, object]:
    plan, integration = inputs()
    marker = MARKER.value(plan)
    members = marker.get("members")
    if (marker.get("status"), marker.get("producer_execution_proved"), marker.get("output_attestation_present"),
            marker.get("effects"), type(members), len(members) if isinstance(members, list) else -1) != (
                "captured-unadmitted", False, False, {key: False for key in PLAN.CONTRACT.EFFECTS}, list, 6):
        reject("capture marker does not retain the sealed unadmitted closure")
    value: dict[str, object] = {
        "schema": 1, "kind": "gles-sealed-capture-binding-v1", "status": "captured-unadmitted",
        "integration_sha256": integration["integration_sha256"],
        "capture": {"contract_sha256": plan.contract_sha256, "closure_sha256": plan.closure_sha256,
                    "configuration_document_sha256": plan.configuration_sha256,
                    "marker_relative_path": "/".join(MARKER.relative(plan)),
                    "marker_sha256": marker["marker_sha256"],
                    "marker_file_sha256": hashlib.sha256(json.dumps(marker, sort_keys=True, separators=(",", ":")).encode()).hexdigest(),
                    "members": members},
        "readiness": {key: False for key in PLAN.CONTRACT.EFFECTS}, "binding_sha256": "",
    }
    value["binding_sha256"] = digest(value)
    return value


def validate(path: Path = RECORD) -> dict[str, object]:
    value = document(path)
    if not isinstance(value.get("binding_sha256"), str) or value["binding_sha256"] != digest(value):
        reject("capture binding self-hash is invalid")
    if not alike(value, build()):
        reject("capture binding does not bind the exact sealed closure")
    return value


def replay(cache, path: Path = RECORD) -> dict[str, object]:
    value, plan = validate(path), PLAN.plan()
    try:
        RUN.replay(cache, plan)
        marker = MARKER.read(cache, plan)
        parent, name = MARKER.directory(cache, plan, False)
        try:
            marker_file_sha256 = hashlib.sha256(MARKER.raw(parent, name)).hexdigest()
        finally:
            os.close(parent)
    except Exception as error:
        reject(f"sealed capture cannot replay: {error}")
    capture = value["capture"]
    if (not isinstance(capture, dict) or marker.get("marker_sha256") != capture.get("marker_sha256")
            or marker_file_sha256 != capture.get("marker_file_sha256")):
        reject("replayed marker does not bind this capture")
    return value


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    action = parser.add_mutually_exclusive_group(required=True)
    action.add_argument("--build", action="store_true")
    action.add_argument("--verify", action="store_true")
    action.add_argument("--replay", action="store_true")
    parser.add_argument("--cache-root", type=Path)
    args = parser.parse_args()
    try:
        if args.replay and args.cache_root is None:
            reject("replay requires --cache-root")
        cache = RUN.ExternalCache.from_path(args.cache_root, RUN.repository_root(HERE)) if args.replay else None
        value = build() if args.build else replay(cache) if cache else validate()
    except BindingError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(json.dumps(value, indent=2, sort_keys=True) if args.build else f"BINDING: {value['status']} {value['binding_sha256']}")


if __name__ == "__main__":
    main()
