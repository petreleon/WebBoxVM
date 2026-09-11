#!/usr/bin/env python3
"""Admit four immutable roots and one bounded WebBoxVM Vulkan definition."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import shutil
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROLE = HERE.parents[1] / "01-authority-and-transform-boundary"
ROOTS_DIR = HERE.parent / "01-normative-root-pins"
LOCAL = HERE.parent / "02-vulkan-local-definition"
sys.path[:0] = [str(ROLE), str(ROOTS_DIR), str(LOCAL)]
from normative_notices import NoticeError  # noqa: E402
from normative_roots import ROOTS, RootError, catalog as root_catalog, fetch_and_verify  # noqa: E402
from source_model import ContractError  # noqa: E402
from source_role_artifacts import verify_catalog  # noqa: E402
from source_role_contract import validate_catalog  # noqa: E402
from source_role_records import RoleError, artifact  # noqa: E402
import vulkan_definition_contract as definition  # noqa: E402

NO_CLAIMS = {"khronos_selector": False, "api_support": False, "conformance": False,
             "certification": False, "profile_support": False, "performance": False}
ROOT_IDS = tuple(item["id"] for item in ROOTS)


class AdmissionError(ValueError):
    """The local catalog has not earned source-consumer admission."""


def reject(message: str) -> None:
    raise AdmissionError(message)


def admission_catalog() -> dict[str, object]:
    return {"schema": 1, "records": [*copy.deepcopy(ROOTS), definition.transform()]}


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode() + b"\n"


def validate_admission(value: object) -> None:
    try:
        validate_catalog(value)
    except RoleError as error:
        reject(str(error))
    if value != admission_catalog():
        reject("admission catalog differs from its exact four roots and local transform")


def local_transform(value: dict[str, object]) -> dict[str, object]:
    return next(item for item in value["records"] if item["id"] == definition.OUTPUT_ID)


def admission_receipt(value: object) -> dict[str, object]:
    validate_admission(value)
    catalog = value
    output = local_transform(catalog)
    return {
        "schema": 1, "kind": "webboxvm-source-admission-receipt",
        "authority": "WebBoxVM", "producer": "WebBoxVM", "claims": dict(NO_CLAIMS),
        "cts_executions": 0, "source_ids": list(ROOT_IDS),
        "catalog_sha256": hashlib.sha256(canonical(catalog)).hexdigest(),
        "output": {"id": output["id"], "sha256": output["sha256"], "bytes": output["bytes"]},
    }


def validate_receipt(value: object, receipt: object) -> None:
    if receipt != admission_receipt(value):
        reject("receipt differs from the exact no-claim admission result")


def existing_root(value: Path) -> Path:
    if not value.is_absolute() or value.is_symlink() or not value.is_dir():
        reject("artifact root must be an absolute regular directory")
    return value.resolve()


def stage_builder(root: Path, value: dict[str, object]) -> Path:
    definition.validate_build(definition.build_record())
    transform = local_transform(value)
    destination = root / artifact(transform["builder"]["artifact"], "builder artifact")
    if destination.exists() or destination.is_symlink():
        reject("fresh artifact root already contains a builder artifact")
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(definition.BUILDER_PATH, destination)
    return destination


def verify_admission(value: object, artifact_root: Path) -> dict[str, object]:
    validate_admission(value)
    root = existing_root(artifact_root)
    try:
        verify_catalog(value, root)
    except RoleError as error:
        reject(str(error))
    transform = local_transform(value)
    staged = root / transform["builder"]["artifact"]
    expected = (definition.BUILDER_BYTES, definition.BUILDER_SHA256)
    if definition.fingerprint(staged, "staged builder") != expected:
        reject("staged builder differs from its recorded bytes and SHA-256")
    receipt = admission_receipt(value)
    validate_receipt(value, receipt)
    return receipt


def admit(cache_root: Path, timeout: float) -> dict[str, object]:
    if not cache_root.is_absolute():
        reject("fresh cache root must be absolute")
    try:
        fetched = fetch_and_verify(root_catalog(), cache_root, timeout)
    except (ContractError, NoticeError, RootError, RoleError) as error:
        reject(str(error))
    if {identifier for identifier, state, _path in fetched} != set(ROOT_IDS) or any(state != "fetched" for _id, state, _path in fetched):
        reject("fresh source refresh did not fetch every immutable root exactly once")
    root = existing_root(cache_root)
    value = admission_catalog()
    stage_builder(root, value)
    paths = {identifier: path for identifier, _state, path in fetched}
    output = root / local_transform(value)["artifact"]
    definition.rebuild(definition.build_record(), paths["vulkan-14-spec"], paths["vulkan-registry"], output)
    return verify_admission(value, root)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--timeout", type=float, default=30.0)
    args = parser.parse_args()
    try:
        print(json.dumps(admit(args.cache_root, args.timeout), sort_keys=True))
    except (AdmissionError, definition.DefinitionError, definition.builder.BuildError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
