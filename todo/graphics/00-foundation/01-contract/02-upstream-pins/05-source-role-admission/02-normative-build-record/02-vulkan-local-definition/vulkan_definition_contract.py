#!/usr/bin/env python3
"""Bind the bounded Vulkan XML-facts build to exact local identities."""

from __future__ import annotations

import copy
import argparse
import hashlib
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROLE = HERE.parent.parent / "01-authority-and-transform-boundary"
ROOTS = HERE.parent / "01-normative-root-pins"
sys.path[:0] = [str(ROLE), str(ROOTS)]
from normative_roots import EXPECTED  # noqa: E402
from source_role_contract import validate_catalog  # noqa: E402
from source_role_records import RoleError  # noqa: E402
import webboxvm_source_builder as builder  # noqa: E402

INPUT_IDS = ("vulkan-14-spec", "vulkan-registry")
OUTPUT_ID = "vulkan-14-core-definition"
OUTPUT_SHA256 = "29b9b64f67eca6c8ddf3edd40bddcb0843c059403dd757ec3479c2939dfd7556"
OUTPUT_BYTES = 7506
BUILDER_SHA256 = "5783e76ee1293887f3678153837d4887368ede3376970ce048a9c109aca5e5b6"
BUILDER_BYTES = 5615
BUILDER_PATH = HERE / "webboxvm_source_builder.py"
NO_CLAIMS = {"khronos_selector": False, "api_support": False, "conformance": False,
             "certification": False, "profile_support": False, "performance": False}
TOOLCHAIN = {
    "builder": {"id": "webboxvm-source-builder", "sha256": BUILDER_SHA256,
                "bytes": BUILDER_BYTES, "artifact": f"webboxvm-graphics/f02/webboxvm-source-builder/{BUILDER_SHA256}.py",
                "source_revision": "de5d7d05"},
    "runtime": {"implementation": "cpython", "version": "3.14.6"},
    "execution": {"network": "forbidden", "argv": [
        "python3", "@builder:webboxvm-source-builder", "--mode", "core-definition",
        "--vkspec", "@input:vulkan-14-spec", "--vkxml", "@input:vulkan-registry",
        "--output", "@output:vulkan-14-core-definition"],
        "serialization": "UTF-8 JSON; sorted keys; compact separators; one trailing LF"},
}


class DefinitionError(ValueError):
    """The local Vulkan definition is not its reviewed reproducible build."""


def reject(message: str) -> None:
    raise DefinitionError(message)


def source_input(identifier: str) -> dict[str, object]:
    item = EXPECTED[identifier]
    return {"id": identifier, "kind": "upstream-source", "sha256": item["sha256"], "revision": item["revision"]}


def transform() -> dict[str, object]:
    builder_record = {key: TOOLCHAIN["builder"][key] for key in ("id", "sha256", "artifact")}
    return {
        "kind": "webboxvm-transform", "id": OUTPUT_ID, "sha256": OUTPUT_SHA256, "bytes": OUTPUT_BYTES,
        "license": "Apache-2.0 OR MIT for extracted vk.xml facts; no Vulkan prose reproduced",
        "attribution": "Khronos Group Vulkan-Docs; WebBoxVM-derived bounded XML facts",
        "scope": "webboxvm-core-definition", "authority": "WebBoxVM", "producer": "WebBoxVM",
        "claims": dict(NO_CLAIMS), "inputs": [source_input(item) for item in INPUT_IDS],
        "command": ["webboxvm-source-builder", "--mode=core-definition",
                    "@input:vulkan-14-spec", "@input:vulkan-registry", "@output:vulkan-14-core-definition"],
        "artifact": f"webboxvm-graphics/f02/{OUTPUT_ID}/{OUTPUT_SHA256}.json", "builder": builder_record,
    }


def catalog() -> dict[str, object]:
    return {"schema": 1, "records": [*(copy.deepcopy(EXPECTED[item]) for item in INPUT_IDS), transform()]}


def build_record() -> dict[str, object]:
    return {"schema": 1, "catalog": catalog(), "toolchain": copy.deepcopy(TOOLCHAIN)}


def fingerprint(path: Path, label: str) -> tuple[int, str]:
    if path.is_symlink() or not path.is_file():
        reject(f"{label} must be a regular nonsymlink file")
    data = path.read_bytes()
    return len(data), hashlib.sha256(data).hexdigest()


def live_runtime() -> dict[str, str]:
    return {"implementation": sys.implementation.name,
            "version": ".".join(str(part) for part in sys.version_info[:3])}


def checked_record(value: object) -> dict[str, object]:
    if not isinstance(value, dict) or set(value) != {"schema", "catalog", "toolchain"} or value["schema"] != 1:
        reject("build record has an invalid schema")
    try:
        validate_catalog(value["catalog"])
    except RoleError as error:
        reject(str(error))
    if value["catalog"] != catalog():
        reject("build catalog differs from the exact Vulkan source and output record")
    if value["toolchain"] != TOOLCHAIN:
        reject("build toolchain differs from its exact offline identity")
    return value


def validate_build(value: object, builder_path: Path = BUILDER_PATH,
                   runtime: dict[str, str] | None = None) -> None:
    checked_record(value)
    if fingerprint(builder_path, "builder") != (BUILDER_BYTES, BUILDER_SHA256):
        reject("builder bytes or SHA-256 differ from the recorded builder")
    if (live_runtime() if runtime is None else runtime) != TOOLCHAIN["runtime"]:
        reject("runtime differs from the recorded CPython identity")


def expected_output(value: object) -> tuple[int, str]:
    checked_record(value)
    output = next(item for item in value["catalog"]["records"] if item["id"] == OUTPUT_ID)
    return output["bytes"], output["sha256"]


def verify_output(value: object, output: Path) -> None:
    validate_build(value)
    if fingerprint(output, "output") != expected_output(value):
        reject("output bytes or SHA-256 differ from the recorded local definition")


def rebuild(value: object, vkspec: Path, vkxml: Path, output: Path) -> Path:
    validate_build(value)
    if output.exists() or output.is_symlink():
        reject("independent rebuild output must be absent and nonsymlink")
    data = builder.encode(vkspec, vkxml)
    if (len(data), hashlib.sha256(data).hexdigest()) != expected_output(value):
        reject("verified inputs did not reproduce the recorded local definition")
    builder.write_output(output, data)
    verify_output(value, output)
    return output


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--vkspec", type=Path, required=True)
    parser.add_argument("--vkxml", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        output = rebuild(build_record(), args.vkspec, args.vkxml, args.output)
        size, digest = fingerprint(output, "output")
        print(f"PASS: {output} bytes={size} sha256={digest}")
    except (DefinitionError, builder.BuildError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
