"""Hermetic normalized-capture fixtures for the core input/scope binder."""

from __future__ import annotations

import copy
import hashlib

from vulkan_docs_scope_bind import _bind_fixture
from vulkan_docs_scope_model import (
    CONFIG_INPUTS, DERIVED, EXTENSION_CONTROLS, IMAGE_COUNT, PHASES, PROMOTIONS, RAW, CaptureExpectation,
)
from vulkan_docs_scope_parse import canonical

from vulkan_docs_identity_build import OUTPUT, ROOT, TREE
from vulkan_docs_observer_events import producer_digest
from vulkan_docs_observer_plan import SOURCE_TREE


def sha(value: str) -> str:
    return hashlib.sha256(value.encode("ascii")).hexdigest()


def row(kind: str, name: str, roles: list[str] | None = None) -> dict[str, object]:
    return {"kind": kind, "selector": name, "sha256": sha(f"{kind}:{name}"), "bytes": len(name) + 10,
            "phase_roles": roles or ["asciidoctor"]}


def refresh(value: dict[str, object]) -> None:
    rows = value["records"]
    assert isinstance(rows, list)
    rows.sort(key=lambda item: (item["kind"], item["selector"]))
    value["raw_count"] = sum(item["kind"] == RAW for item in rows)
    value["derived_count"] = len(rows) - value["raw_count"]
    value["phase_counts"] = {name: sum(name in item["phase_roles"] for item in rows) for name in sorted(PHASES)}
    value["input_manifest_sha256"] = canonical(rows, "webboxvm-graphics-vulkan-docs-observed-input-manifest-v1")
    value["include_identity_sha256"] = canonical(value["includes"], "webboxvm-graphics-vulkan-docs-resolved-includes-v1")


def normalized() -> dict[str, object]:
    rows = [
        {"kind": RAW, "selector": "vkspec.adoc", "sha256": ROOT["sha256"], "bytes": ROOT["bytes"],
         "phase_roles": ["asciidoctor"]},
        row(RAW, "Makefile", ["make-control"]), row(RAW, "scripts/genvk.py", ["generator"]),
        row(RAW, "makeSpec", ["producer"]), row(RAW, "scripts/translate_math.js", ["postprocess"]),
        row(RAW, "styles/core.css", ["asset-copy"]),
    ]
    rows.extend(row(kind, name) for kind, name in EXTENSION_CONTROLS)
    rows.extend(row(kind, name) for kind, name in PROMOTIONS)
    rows.extend(row(DERIVED, name) for name in CONFIG_INPUTS)
    rows.extend(row(RAW, f"images/fixture-{number:02}.svg") for number in range(IMAGE_COUNT))
    selected = (*EXTENSION_CONTROLS, *PROMOTIONS, *((DERIVED, name) for name in CONFIG_INPUTS))
    value: dict[str, object] = {
        "records": rows, "includes": [{"kind": kind, "selector": name, "line": number + 1}
                                            for number, (kind, name) in enumerate(selected)],
        "raw_count": 0, "derived_count": 0, "ignored_runtime_reads": 0, "phase_counts": {},
        "input_manifest_sha256": "", "include_identity_sha256": "",
    }
    refresh(value)
    return value


def expectation(value: dict[str, object], identifier: str = "fixture-a", artifact: str = "runs/fixture-a",
                observation: str = "1" * 64, run: str = "2" * 64, normalized_digest: str = "3" * 64) -> CaptureExpectation:
    return CaptureExpectation(
        observation, identifier, artifact, run, normalized_digest, "4" * 64, "5" * 64, SOURCE_TREE[2], TREE[2], OUTPUT["sha256"],
        producer_digest(), value["raw_count"], value["derived_count"], len(value["includes"]),
        dict(value["phase_counts"]), value["input_manifest_sha256"], value["include_identity_sha256"],
    )


def valid() -> tuple[dict[str, object], CaptureExpectation]:
    value = normalized()
    return value, expectation(value)


def bound() -> tuple[dict[str, object], CaptureExpectation, object]:
    value, capture = valid()
    return value, capture, _bind_fixture(copy.deepcopy(value), capture)
