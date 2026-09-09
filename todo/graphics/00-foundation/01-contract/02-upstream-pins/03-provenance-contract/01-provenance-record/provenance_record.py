"""Offline parser and validator for F02 provenance sidecar records."""

from __future__ import annotations

import hashlib
import json
import re
import tomllib
from pathlib import Path, PurePosixPath

DIGEST = re.compile(r"^[0-9a-f]{64}$")
RECORD_FIELDS = frozenset((
    "schema", "manifest_sha256", "inputs", "command", "generator", "artifact_kind",
    "artifact_path", "output_sha256",
))
MANIFEST_FIELDS = frozenset(("schema", "cache_root", "cache_note", "required_families", "inputs"))
MANIFEST_INPUT_FIELDS = frozenset((
    "id", "source_family", "immutable_url", "revision", "sha256", "bytes", "license",
    "local_cache", "generated_code_role", "provenance",
))
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


def manifest_inputs(path: Path) -> tuple[str, dict[str, tuple[str, str]]]:
    try:
        raw = path.read_bytes()
        document = tomllib.loads(raw.decode("utf-8"))
    except (OSError, UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
        reject(f"manifest cannot be read: {error}")
    if (not isinstance(document, dict) or set(document) != MANIFEST_FIELDS
            or type(document.get("schema")) is not int or document["schema"] != 1):
        reject("manifest does not match F02.1 schema version 1")
    families = document["required_families"]
    if (document["cache_root"] != "$XDG_CACHE_HOME" or not isinstance(document["cache_note"], str)
            or not document["cache_note"] or not isinstance(families, list) or not families
            or any(not isinstance(family, str) or not family for family in families)
            or len(set(families)) != len(families)):
        reject("manifest does not declare the F02.1 external-cache contract")
    entries = document["inputs"]
    if not isinstance(entries, list) or not entries:
        reject("manifest has no declared inputs")
    identities: dict[str, tuple[str, str]] = {}
    for entry in entries:
        if not isinstance(entry, dict) or set(entry) != MANIFEST_INPUT_FIELDS:
            reject("manifest input does not match F02.1 schema")
        identifier = string(entry.get("id"), "manifest input id")
        identity = (digest(entry.get("sha256"), f"manifest input {identifier} sha256"),
                    string(entry.get("license"), f"manifest input {identifier} license"))
        if identifier in identities:
            reject(f"manifest has duplicate input {identifier}")
        identities[identifier] = identity
    return hashlib.sha256(raw).hexdigest(), identities


def load_record(path: Path) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"record cannot be read: {error}")
    if (not isinstance(value, dict) or set(value) != RECORD_FIELDS
            or type(value.get("schema")) is not int or value["schema"] != 1):
        reject("record does not match provenance schema version 1")
    return value


def safe_artifact_path(value: object) -> str:
    path = PurePosixPath(string(value, "artifact_path"))
    if not path.parts or path.is_absolute() or ".." in path.parts or str(path) != value:
        reject("artifact_path is unsafe")
    return str(path)


def validate_record(record_path: Path, manifest_path: Path) -> dict[str, object]:
    record = load_record(record_path)
    revision, identities = manifest_inputs(manifest_path)
    if digest(record["manifest_sha256"], "manifest_sha256") != revision:
        reject("record has a stale manifest_sha256")
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
