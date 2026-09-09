"""Offline parser and validator for F02 provenance sidecar records."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path, PurePosixPath

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "01-input-inventory"))

from inventory_layout import InventoryLayoutError, load_inventory  # noqa: E402

DIGEST = re.compile(r"^[0-9a-f]{64}$")
V1_RECORD_FIELDS = frozenset((
    "schema", "manifest_sha256", "inputs", "command", "generator", "artifact_kind",
    "artifact_path", "output_sha256",
))
V2_RECORD_FIELDS = V1_RECORD_FIELDS - {"manifest_sha256"} | {"inventory_sha256"}
RECORD_FIELDS = {1: V1_RECORD_FIELDS, 2: V2_RECORD_FIELDS}
INPUT_FIELDS = frozenset(("id", "sha256", "license"))
GENERATOR_FIELDS = frozenset(("name", "version"))
KINDS = frozenset(("handwritten", "copied-upstream", "generated"))


class ProvenanceError(ValueError):
    """A provenance record does not bind to the reviewed immutable inventory."""


def reject(message: str) -> None:
    raise ProvenanceError(message)


def string(value: object, field: str) -> str:
    if not isinstance(value, str) or not value:
        reject(f"{field} must be a nonempty string")
    return value


def digest(value: object, field: str) -> str:
    value = string(value, field)
    if not DIGEST.fullmatch(value) or value == "0" * 64:
        reject(f"{field} must be a non-placeholder SHA-256")
    return value


def inventory_inputs(path: Path) -> tuple[int, str, dict[str, tuple[str, str]]]:
    try:
        inventory = load_inventory(path, allow_v1=True)
    except InventoryLayoutError as error:
        reject(f"inventory cannot be loaded: {error}")
    identities: dict[str, tuple[str, str]] = {}
    for entry in inventory.inputs:
        identifier = string(entry.get("id"), "manifest input id")
        identity = (digest(entry.get("sha256"), f"manifest input {identifier} sha256"),
                    string(entry.get("license"), f"manifest input {identifier} license"))
        if identifier in identities:
            reject(f"manifest has duplicate input {identifier}")
        identities[identifier] = identity
    return inventory.schema, inventory.revision, identities


def load_record(path: Path) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"record cannot be read: {error}")
    schema = value.get("schema") if isinstance(value, dict) else None
    if type(schema) is not int or schema not in RECORD_FIELDS:
        reject("record does not match provenance schema version 1 or 2")
    if set(value) != RECORD_FIELDS[schema]:
        reject(f"record does not match provenance schema version {schema}")
    return value


def safe_artifact_path(value: object) -> str:
    path = PurePosixPath(string(value, "artifact_path"))
    if not path.parts or path.is_absolute() or ".." in path.parts or str(path) != value:
        reject("artifact_path is unsafe")
    return str(path)


def validate_record(record_path: Path, manifest_path: Path) -> dict[str, object]:
    record = load_record(record_path)
    inventory_schema, revision, identities = inventory_inputs(manifest_path)
    if record["schema"] != inventory_schema:
        reject("record schema version does not match inventory schema")
    revision_field = "manifest_sha256" if inventory_schema == 1 else "inventory_sha256"
    if digest(record[revision_field], revision_field) != revision:
        reject(f"record has a stale {revision_field}")
    kind = string(record["artifact_kind"], "artifact_kind")
    if kind not in KINDS:
        reject("artifact_kind is unknown")
    safe_artifact_path(record["artifact_path"])
    output = digest(record["output_sha256"], "output_sha256")
    string(record["command"], "command")
    generator = record["generator"]
    if not isinstance(generator, dict) or set(generator) != GENERATOR_FIELDS:
        reject("generator does not match schema")
    name, version = string(generator["name"], "generator.name"), string(generator["version"], "generator.version")
    if kind == "generated" and (name == "none" or version == "none"):
        reject("generated artifact needs a generator identity and version")
    if kind != "generated" and (name, version) != ("none", "none"):
        reject("non-generated artifact must name generator none")
    references = record["inputs"]
    if not isinstance(references, list) or not references:
        reject("record must reference at least one manifest input")
    seen: set[str] = set()
    resolved: list[tuple[str, str]] = []
    previous = ""
    for reference in references:
        if not isinstance(reference, dict) or set(reference) != INPUT_FIELDS:
            reject("input reference does not match schema")
        identifier = string(reference["id"], "input id")
        expected = identities.get(identifier)
        if expected is None:
            reject(f"input reference {identifier} is unknown")
        if identifier in seen:
            reject(f"input reference {identifier} is duplicated")
        if previous and identifier <= previous:
            reject("input references are not sorted by id")
        if digest(reference["sha256"], f"input reference {identifier} sha256") != expected[0]:
            reject(f"input reference {identifier} has a stale sha256")
        if string(reference["license"], f"input reference {identifier} license") != expected[1]:
            reject(f"input reference {identifier} has a stale license")
        seen.add(identifier)
        previous = identifier
        resolved.append(expected)
    if kind == "copied-upstream" and (len(resolved) != 1 or output != resolved[0][0]):
        reject("copied-upstream artifact must have one input with matching output_sha256")
    if kind == "handwritten" and output in {identity[0] for identity in resolved}:
        reject("handwritten artifact must not masquerade as copied upstream bytes")
    return record
