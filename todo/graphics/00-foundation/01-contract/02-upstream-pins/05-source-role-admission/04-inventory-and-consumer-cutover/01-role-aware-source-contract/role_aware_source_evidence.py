"""Exact no-claim closure and auxiliary evidence for the F02.5.4 source contract."""

from __future__ import annotations

import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
SOURCE_ROLE = HERE.parents[1]
VULKAN_RECEIPT = SOURCE_ROLE / "03-full-suite-record/03-vulkan-default-full-ledger/03-fresh-full-suite-receipt/vulkan_full_suite_receipt.json"
SHARD_RECEIPT = SOURCE_ROLE / "03-full-suite-record/04-local-shards-and-no-claim-receipt/vulkan_local_shard_receipt.json"
NO_CLAIMS = {"khronos_selector": False, "api_support": False, "conformance": False,
             "certification": False, "profile_support": False, "performance": False}
GL_GLES_RECEIPT = "778c4eeb479fa4a6169385ef0a8e75ed54bf128b2ee0d23e29ee1afdff069e95"
VULKAN_FULL_RECEIPT = "406130b30ca18b1843962456a5309ab24d41b222213b9a9d0e1cac06b6c98bb0"
VULKAN_SHARD_RECEIPT = "0f4452659c0e2b1a23d58e11af12786a97d6f91c87dd8da387a697766a7f90ed"


class EvidenceError(ValueError):
    """A local receipt has been promoted or disconnected from its source root."""


def reject(message: str) -> None:
    raise EvidenceError(message)


def closure(record: dict[str, object], profile: str, ledger: str, receipt: str,
            task: str) -> dict[str, object]:
    return {"profile": profile, "root_id": record["id"], "root_kind": record["kind"],
            "revision": record["revision"], "sha256": record["sha256"],
            "unfiltered": record["unfiltered"], "closure_kind": "full-suite-ledger",
            "ledger_sha256": ledger, "receipt_sha256": receipt, "evidence_task": task}


def closures(records: dict[str, dict[str, object]]) -> list[dict[str, object]]:
    result = [
        closure(records["opengl-cts-gl46-main"], "opengl-4.6-core",
                "a88cd6b9fad2268f9ef355806cdfea1cac54994ff512b14f8292be89bd5a7517",
                GL_GLES_RECEIPT, "F02.5.3.2.3"),
        closure(records["gles-cts-main"], "gles-3.2",
                "3d4d8b10d3a46a51f0c30606430131dce8207253423e0a33c996573ffb4b3655",
                GL_GLES_RECEIPT, "F02.5.3.2.3"),
        closure(records["vulkan-cts-default"], "vulkan-1.4-core",
                "608d520463bfd4724e79b52ee639a12d446e14c5e8d7b3687872eaf3c4e917f0",
                VULKAN_FULL_RECEIPT, "F02.5.3.3.3"),
    ]
    result[2].update(identity_sha256="30b272f8c563e0dabf307795c01496eb70f744514b4790439ebbfc69c7bd5218",
                     member_count=98, member_total_bytes=434669348,
                     taxonomy_record_sha256="cea45295dab76b0adeed650f884cb14a12ff9e169bade31200c10649e50ed6f3",
                     replay_receipt_sha256="bc844fe3b3c32234b3ed3ec61da8267dbc3d67ab93e73bf4455fd8e320ccfdd4",
                     selector_scope="Khronos vk-default root; broader than core-only")
    return result


def auxiliary(records: dict[str, dict[str, object]]) -> list[dict[str, object]]:
    def local(record: dict[str, object]) -> dict[str, object]:
        value = {"id": record["id"], "record_kind": record["kind"], "scope": record["scope"],
                 "sha256": record["sha256"], "claims": dict(NO_CLAIMS), "cts_executions": 0,
                 "discharges_required_role": False}
        if "builder" in record:
            value["builder"] = dict(record["builder"])
        return value
    return [
        local(records["vulkan-registry"]), local(records["vulkan-14-core-definition"]),
        {"id": "gl-gles-full-suite-engineering-map", "evidence_kind": "webboxvm-engineering-map",
         "receipt_sha256": GL_GLES_RECEIPT, "claims": dict(NO_CLAIMS), "cts_executions": 0,
         "discharges_required_role": False},
        {"id": "vulkan-api-byte-preserving-shards", "evidence_kind": "webboxvm-byte-preserving-shard-receipt",
         "receipt_sha256": VULKAN_SHARD_RECEIPT, "claims": dict(NO_CLAIMS), "cts_executions": 0,
         "discharges_required_role": False},
    ]


def read(path: Path, label: str) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        reject(f"{label} is unavailable: {error}")
    if not isinstance(value, dict):
        reject(f"{label} is not an object")
    return value


def root_matches(value: object, closure: dict[str, object]) -> bool:
    if not isinstance(value, dict):
        return False
    return (value.get("id"), value.get("revision"), value.get("sha256")) == (
        closure["root_id"], closure["revision"], closure["sha256"])


def false_state(value: object) -> bool:
    return value == {"admitted": False, "cutover_ready": False,
                     "satisfies_vulkan_14_core_manifest": False}


def validate_receipt_files(closure_rows: list[dict[str, object]], auxiliary_rows: list[dict[str, object]]) -> None:
    if len(closure_rows) != 3 or len(auxiliary_rows) != 4:
        reject("source contract has incomplete closure or auxiliary evidence")
    vulkan = closure_rows[2]
    full = read(VULKAN_RECEIPT, "Vulkan full-suite receipt")
    if (full.get("receipt_sha256") != vulkan["receipt_sha256"] or full.get("claims") != NO_CLAIMS
            or full.get("cts_executions") != 0 or not root_matches(full.get("source_root"), vulkan)
            or full.get("source_root", {}).get("unfiltered") is not True
            or any(full.get("ledger", {}).get(key) != vulkan[key] for key in
                   ("identity_sha256", "ledger_sha256", "member_count", "member_total_bytes"))
            or full.get("capture", {}).get("cache", {}).get("replay_receipt_sha256") != vulkan["replay_receipt_sha256"]
            or not false_state(full.get("states"))):
        reject("Vulkan full-suite receipt does not remain exact no-claim closure evidence")
    shards = read(SHARD_RECEIPT, "Vulkan local shard receipt")
    expected = auxiliary_rows[3]["receipt_sha256"]
    if (shards.get("receipt_sha256") != expected or shards.get("claims") != NO_CLAIMS
            or shards.get("cts_executions") != 0 or not root_matches(shards.get("source_root"), vulkan)
            or shards.get("max_local_bytes") != 8 * 1024 * 1024
            or shards.get("mode") != "byte-preserving-shard"
            or not false_state(shards.get("states"))):
        reject("local Vulkan shards are not exact non-admitting auxiliary evidence")
