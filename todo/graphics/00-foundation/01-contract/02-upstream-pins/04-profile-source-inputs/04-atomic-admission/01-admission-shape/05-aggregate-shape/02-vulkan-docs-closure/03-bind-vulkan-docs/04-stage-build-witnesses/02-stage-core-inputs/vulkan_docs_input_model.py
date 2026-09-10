"""Bounded member and receipt vocabulary for unadmitted Docs input staging."""

from __future__ import annotations

from dataclasses import dataclass
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
STAGE = HERE.parent / "01-stage-contract"
if str(STAGE) not in sys.path:
    sys.path.insert(0, str(STAGE))

from vulkan_docs_stage_model import MAX_INPUT_BYTES, reject
from vulkan_docs_stage_parse import bounded, digest, exact_object
from vulkan_docs_stage_paths import relative

RAW = "raw-observed-input"
DERIVED = "derived-observed-input"
MAX_STAGED_BYTES = 32 * 1024 * 1024
MAX_PROVIDER_BYTES = 2 * MAX_STAGED_BYTES
INPUT_STAGE_CONTRACT = "vulkan-docs-core-input-stage-receipt-v1"
RAW_FIELDS = frozenset(("kind", "selector", "sha256", "bytes", "phase_roles", "revision", "immutable_url"))
DERIVED_FIELDS = frozenset(("kind", "selector", "sha256", "bytes", "phase_roles", "generation_id"))
INPUT_STAGE_FIELDS = frozenset((
    "schema", "contract", "status", "admitted", "cutover_ready", "profile", "role", "required_input_id",
    "plan_sha256", "build_witness_sha256", "scope_identity_sha256", "comparison_sha256", "captures", "inputs",
    "input_stage_sha256",
))


@dataclass(frozen=True)
class InputMember:
    kind: str
    selector: str
    staged_selector: str
    bytes: int
    sha256: str
    phase_roles: tuple[str, ...]

    def normalized(self) -> dict[str, object]:
        return {
            "kind": self.kind, "selector": self.selector, "sha256": self.sha256,
            "bytes": self.bytes, "phase_roles": list(self.phase_roles),
        }


def member(value: object, kind: str) -> InputMember:
    fields = RAW_FIELDS if kind == RAW else DERIVED_FIELDS
    data = exact_object(value, fields, "Docs staged input")
    if data.get("kind") != kind:
        reject("Docs staged input has the wrong kind")
    selector = data.get("selector")
    if not isinstance(selector, str):
        reject("Docs staged input has an invalid selector")
    relative(selector)
    staged = selector
    if kind == DERIVED:
        if not selector.startswith("generated/") or selector == "generated/out" or selector.startswith("generated/out/"):
            reject("Docs derived input lacks its generated selector prefix")
    elif selector.startswith("generated/"):
        reject("Docs raw input has a generated selector prefix")
    roles = data.get("phase_roles")
    if not isinstance(roles, list) or not roles or any(not isinstance(role, str) for role in roles):
        reject("Docs staged input has invalid phase roles")
    return InputMember(
        kind, selector, staged, bounded(data.get("bytes"), "Docs staged input bytes", MAX_INPUT_BYTES),
        digest(data.get("sha256"), "Docs staged input sha256"), tuple(roles),
    )


def encode(value: object) -> bytes:
    try:
        return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True, allow_nan=False).encode("utf-8")
    except (TypeError, ValueError) as error:
        reject(f"Docs input-stage receipt cannot be encoded: {error}")


def _pairs(rows: list[tuple[str, object]]) -> dict[str, object]:
    result = dict(rows)
    if len(result) != len(rows):
        reject("Docs input-stage receipt has duplicate JSON keys")
    return result


def _constant(value: str) -> None:
    reject(f"Docs input-stage receipt has a non-finite JSON value: {value}")


def decode(payload: bytes) -> dict[str, object]:
    try:
        value = json.loads(payload.decode("utf-8"), object_pairs_hook=_pairs, parse_constant=_constant)
    except (UnicodeDecodeError, json.JSONDecodeError, TypeError, ValueError) as error:
        reject(f"Docs input-stage receipt is invalid JSON: {error}")
    if not isinstance(value, dict):
        reject("Docs input-stage receipt is not an object")
    return value
