"""Small in-memory fixtures for the actual-Docs grammar tests only."""

from __future__ import annotations

import copy
import json
from pathlib import Path

from vulkan_docs_identity_parse import canonical

HERE = Path(__file__).resolve().parent
WITNESS = HERE / "vulkan_docs_build_witness.json"
COMMIT = "f84d432d5b8912362f96f581f29bbc4f3c8c7843"
ROOT_DIGEST = "069b7e6d6326969df7b4a86f189f7ba22359e93ca7aa76c23301504667d3c4b0"
GENERATION_ID = "official-vulkan-docs-core-html-1-4-362"
INCLUDE_LIST = "b35c3b8907a3338e688f31bfffd98c174858d31b95d88607046a07314f6e964b"
EXCLUSIONS = {
    "wsi": ["include::{chapters}/VK_KHR_surface/wsi.adoc[]"],
    "video": ["include::{chapters}/videocoding.adoc[]"],
    "extensions": ["include::{chapters}/extensions.adoc[]"],
}


def witness() -> dict[str, object]:
    return json.loads(WITNESS.read_text(encoding="utf-8"))


def reseal_build(value: dict[str, object]) -> None:
    build = value["build"]
    assert isinstance(build, dict)
    payload = dict(build)
    payload.pop("build_sha256", None)
    build["build_sha256"] = canonical(payload, "webboxvm-graphics-build_sha256-v1")


def reseal_witness(value: dict[str, object]) -> None:
    reseal_build(value)
    payload = dict(value)
    payload.pop("witness_sha256", None)
    value["witness_sha256"] = canonical(payload, "webboxvm-graphics-witness_sha256-v1")


def raw(identifier: str, selector: str, digest: str, count: int, provenance: str = "captured Docs source",
        license: str = "LicenseRef-KhronosSpecCopyright") -> dict[str, object]:
    return {
        "kind": "raw-source-input", "id": identifier, "source_family": "vulkan-docs",
        "immutable_url": f"https://raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/{COMMIT}/{selector}",
        "revision": COMMIT, "sha256": digest, "bytes": count, "license": license,
        "local_cache": f"webboxvm-graphics/successor/vulkan-docs-v1/raw/{identifier}/{digest}.source",
        "generated_code_role": "Docs source", "provenance": provenance, "selector": selector,
    }


def derived(identifier: str, digest: str, generation_id: str = GENERATION_ID) -> dict[str, object]:
    return {
        "kind": "derived-source-input", "id": identifier,
        "producer_input_ids": ["vulkan-14-spec", "vulkan-docs-registry"], "generation_id": generation_id,
        "selector": "generated/specattribs.adoc", "sha256": digest, "bytes": 313,
        "license": "LicenseRef-KhronosSpecCopyright",
        "local_cache": f"webboxvm-graphics/successor/vulkan-docs-v1/derived/{identifier}/{digest}.derived",
        "generated_code_role": "Docs generated input", "provenance": "captured Docs generator output",
    }


def member_identity(rows: list[dict[str, object]]) -> str:
    result = []
    for row in rows:
        if row["kind"] == "raw-source-input":
            result.append({
                "identifier": row["id"], "source_family": row["source_family"],
                "immutable_url": row["immutable_url"], "revision": row["revision"], "digest": row["sha256"],
                "byte_count": row["bytes"], "license": row["license"], "selector": row["selector"],
                "cache_path": row["local_cache"], "generated_code_role": row["generated_code_role"],
                "provenance": row["provenance"],
            })
        else:
            result.append({
                "identifier": row["id"], "digest": row["sha256"], "byte_count": row["bytes"],
                "selector": row["selector"], "cache_path": row["local_cache"],
                "producers": tuple(row["producer_input_ids"]), "generation_id": row["generation_id"],
                "license": row["license"], "generated_code_role": row["generated_code_role"],
                "provenance": row["provenance"],
            })
    return canonical(result, "webboxvm-graphics-vulkan-docs-member-identity-v1")


def closure() -> dict[str, object]:
    inputs = [
        raw("vulkan-14-spec", "vkspec.adoc", ROOT_DIGEST, 8685, "official Vulkan-Docs v1.4.362 source root", "CC-BY-4.0"),
        raw("vulkan-docs-registry", "registry.adoc", "1" * 64, 211),
        derived("vulkan-docs-generated", "2" * 64),
    ]
    scope = {
        "scope_kind": "vulkan-docs-core", "evidence_mode": "captured-exact",
        "ordered_input_ids": [item["id"] for item in inputs], "derived_input_ids": ["vulkan-docs-generated"],
        "excluded_members": copy.deepcopy(EXCLUSIONS), "predecessor_include_list_sha256": INCLUDE_LIST,
        "configuration_sha256": configuration_digest(), "member_identity_sha256": member_identity(inputs),
        "scope_sha256": "",
    }
    scope["scope_sha256"] = canonical(
        {key: value for key, value in scope.items() if key != "scope_sha256"},
        "webboxvm-graphics-vulkan-docs-scope-v1")
    value = {
        "schema": 1, "contract": "vulkan-docs-actual-closure-v1", "status": "unadmitted-actual-docs-closure",
        "profile": "vulkan-1.4-core", "role": "api-limit-format-spec", "required_input_id": "vulkan-14-spec",
        "root_input_id": "vulkan-14-spec", "inputs": inputs, "scope": scope,
        "build_witness_sha256": witness()["witness_sha256"], "closure_sha256": "",
    }
    reseal_closure(value)
    return value


def configuration_digest() -> str:
    build = witness()["build"]
    assert isinstance(build, dict)
    payload = dict(build)
    payload.pop("build_sha256", None)
    return canonical(payload, "webboxvm-graphics-vulkan-docs-build-configuration-v1")


def refresh_member_identity(value: dict[str, object]) -> None:
    inputs = value["inputs"]
    scope = value["scope"]
    assert isinstance(inputs, list) and isinstance(scope, dict)
    scope["member_identity_sha256"] = member_identity(inputs)


def reseal_closure(value: dict[str, object]) -> None:
    scope = value["scope"]
    assert isinstance(scope, dict)
    scope_payload = dict(scope)
    scope_payload.pop("scope_sha256", None)
    scope["scope_sha256"] = canonical(scope_payload, "webboxvm-graphics-vulkan-docs-scope-v1")
    payload = dict(value)
    payload.pop("closure_sha256", None)
    value["closure_sha256"] = canonical(payload, "webboxvm-graphics-closure_sha256-v1")
