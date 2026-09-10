"""Bounded output-witness values and strict receipt JSON primitives."""

from __future__ import annotations

from dataclasses import dataclass
import hashlib
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
STAGE = HERE.parent / "01-stage-contract"
if str(STAGE) not in sys.path:
    sys.path.insert(0, str(STAGE))

from vulkan_docs_stage_model import MAX_OUTPUT_BYTES, MAX_OUTPUT_FILES, reject
from vulkan_docs_stage_parse import digest
from vulkan_docs_stage_paths import relative

MAX_OUTPUT_MEMBER_BYTES = 16 * 1024 * 1024
MAX_OUTPUT_DEPTH = 32
MAX_OUTPUT_ENTRIES = MAX_OUTPUT_FILES * 2
OUTPUT_STAGE_CONTRACT = "vulkan-docs-output-witness-stage-receipt-v1"
OUTPUT_STAGE_FIELDS = frozenset((
    "schema", "contract", "status", "admitted", "cutover_ready", "profile", "role", "required_input_id",
    "plan_sha256", "build_witness_sha256", "scope_identity_sha256", "comparison_sha256", "captures", "inputs",
    "input_stage_sha256", "output_witnesses", "output_stage_sha256",
))


def bound(value: object, label: str, maximum: int, *, zero: bool = False) -> int:
    if type(value) is not int or value < (0 if zero else 1) or value > maximum:
        reject(f"{label} is outside its explicit bound")
    return value


@dataclass(frozen=True)
class OutputMember:
    selector: str
    bytes: int
    sha256: str

    def row(self) -> dict[str, object]:
        return {"selector": self.selector, "sha256": self.sha256, "bytes": self.bytes}


@dataclass(frozen=True)
class OutputTree:
    run_id: str
    members: tuple[OutputMember, ...]
    bytes: int
    sha256: str
    primary: OutputMember

    def manifest(self) -> dict[str, object]:
        return {
            "run_id": self.run_id, "manifest_sha256": self.sha256, "file_count": len(self.members),
            "bytes": self.bytes, "tree_sha256": self.sha256, "primary_html_sha256": self.primary.sha256,
        }


def member(selector: object, size: object, value: object) -> OutputMember:
    if not isinstance(selector, str):
        reject("Docs output selector is invalid")
    relative(selector)
    return OutputMember(
        selector, bound(size, "Docs output member bytes", MAX_OUTPUT_MEMBER_BYTES, zero=True),
        digest(value, "Docs output member sha256"),
    )


def tree_digest(members: tuple[OutputMember, ...]) -> str:
    rows = [item.row() for item in members]
    try:
        payload = json.dumps(rows, sort_keys=True, separators=(",", ":"), ensure_ascii=True, allow_nan=False).encode("utf-8")
    except (TypeError, ValueError) as error:
        reject(f"Docs output manifest cannot be encoded: {error}")
    return hashlib.sha256(payload).hexdigest()


def encode(value: object) -> bytes:
    try:
        return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True, allow_nan=False).encode("utf-8")
    except (TypeError, ValueError) as error:
        reject(f"Docs output-stage receipt cannot be encoded: {error}")


def _pairs(rows: list[tuple[str, object]]) -> dict[str, object]:
    result = dict(rows)
    if len(result) != len(rows):
        reject("Docs output-stage receipt has duplicate JSON keys")
    return result


def _constant(value: str) -> None:
    reject(f"Docs output-stage receipt has a non-finite JSON value: {value}")


def decode(payload: bytes) -> dict[str, object]:
    try:
        value = json.loads(payload.decode("utf-8"), object_pairs_hook=_pairs, parse_constant=_constant)
    except (UnicodeDecodeError, json.JSONDecodeError, TypeError, ValueError) as error:
        reject(f"Docs output-stage receipt is invalid JSON: {error}")
    if not isinstance(value, dict):
        reject("Docs output-stage receipt is not an object")
    return value
