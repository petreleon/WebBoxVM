"""Readable, sealed artifact bundle for F03.2.2.5.1."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT_NAME = "opengl_command_domain_classification.json"
MAX_SERIALIZED = 8 * 1024 * 1024
CHUNKS = (
    ("core-sync-families", "opengl_command_domain_core_sync_families.json", 0, 7),
    ("resource-vertex-families", "opengl_command_domain_resource_vertex_families.json", 7, 14),
    ("program-submission-families", "opengl_command_domain_program_submission_families.json", 14, 21),
    ("framebuffer-special-families", "opengl_command_domain_framebuffer_special_families.json", 21, 28),
)
FRAGMENTS = (("routes", "opengl_command_domain_routes.json"), ("baseline", "opengl_command_domain_baseline.json")) + \
    tuple((identifier, filename) for identifier, filename, _, _ in CHUNKS) + \
    (("non-command-families", "opengl_command_domain_non_command_families.json"),)
ALL_NAMES = (ROOT_NAME,) + tuple(name for _, name in FRAGMENTS)


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def serialized(value: object) -> str:
    return json.dumps(value, sort_keys=True, indent=2, allow_nan=False) + "\n"


def digest(value: object) -> str:
    return hashlib.sha256(canonical(value)).hexdigest()


def exact(left: object, right: object) -> bool:
    if type(left) is not type(right):
        return False
    if isinstance(left, dict):
        return left.keys() == right.keys() and all(exact(left[key], right[key]) for key in left)
    if isinstance(left, list):
        return len(left) == len(right) and all(exact(a, b) for a, b in zip(left, right))
    return left == right


def pairs(items):
    value = {}
    for key, item in items:
        if key in value:
            raise ValueError("duplicate JSON field")
        value[key] = item
    return value


def sealed(identifier: str, payload: dict[str, object]) -> dict[str, object]:
    body = {"schema": 1, "kind": "webboxvm-opengl46-command-domain-artifact",
            "artifact_id": identifier, **payload}
    return {**body, "artifact_sha256": digest(body)}


def receipt(identifier: str, filename: str, value: dict[str, object]) -> dict[str, str]:
    return {"artifact_id": identifier, "file": filename, "artifact_sha256": value["artifact_sha256"],
            "serialized_sha256": hashlib.sha256(serialized(value).encode("utf-8")).hexdigest()}


def bundle(core: dict[str, object], routes: list[dict[str, object]], baseline: dict[str, object],
           commands: list[dict[str, object]], non_commands: list[dict[str, object]]) -> dict[str, dict[str, object]]:
    if len(commands) != 28 or len(non_commands) != 6:
        raise ValueError("closed source-family counts changed")
    files = {
        FRAGMENTS[0][1]: sealed("routes", {"routes": routes}),
        FRAGMENTS[1][1]: sealed("baseline", {"baseline": baseline}),
        FRAGMENTS[6][1]: sealed("non-command-families", {"non_command_families": non_commands}),
    }
    for identifier, filename, start, stop in CHUNKS:
        files[filename] = sealed(identifier, {"command_families": commands[start:stop]})
    receipts = [receipt(identifier, filename, files[filename]) for identifier, filename in FRAGMENTS]
    root = {**core, "artifact_receipts": receipts}
    files[ROOT_NAME] = {**root, "classification_sha256": digest(root)}
    return files


def load(path: Path, reject):
    if path.is_symlink() or not path.is_file():
        reject("classification artifact must be a regular file")
    try:
        raw = path.read_bytes()
        if len(raw) > MAX_SERIALIZED:
            reject("classification artifact exceeds the 8 MiB serialized cap")
        value = json.loads(raw.decode("utf-8"), object_pairs_hook=pairs)
    except (OSError, UnicodeDecodeError, ValueError, json.JSONDecodeError) as error:
        reject(f"classification artifact cannot be read: {error}")
    if not isinstance(value, dict):
        reject("classification artifact is not a JSON object")
    return value, hashlib.sha256(raw).hexdigest()


def self_hash(value: dict[str, object], key: str, reject) -> None:
    body = {name: item for name, item in value.items() if name != key}
    if value.get(key) != digest(body):
        reject("classification artifact has a stale self hash")


def materialize(files: dict[str, dict[str, object]]) -> dict[str, object]:
    by_id = {value["artifact_id"]: value for name, value in files.items() if name != ROOT_NAME}
    commands = [row for identifier, _, _, _ in CHUNKS for row in by_id[identifier]["command_families"]]
    return {**files[ROOT_NAME], "routes": by_id["routes"]["routes"], "baseline": by_id["baseline"]["baseline"],
            "command_families": commands, "non_command_families": by_id["non-command-families"]["non_command_families"]}


def validate(root_path: Path, expected: dict[str, dict[str, object]], reject) -> dict[str, object]:
    wanted = set(expected)
    visible = {path.name for path in root_path.parent.glob("opengl_command_domain_*.json")}
    expected_visible = (wanted - {ROOT_NAME}) | {root_path.name}
    if visible != expected_visible:
        reject("classification artifact set is missing a fragment or contains an extra fragment")
    root, _ = load(root_path, reject)
    self_hash(root, "classification_sha256", reject)
    if not exact(root, expected[ROOT_NAME]):
        reject("classification receipt is stale, reordered, partial, or mixed")
    receipts = {item.get("file"): item for item in root["artifact_receipts"] if isinstance(item, dict)}
    if set(receipts) != wanted - {ROOT_NAME}:
        reject("classification receipt does not bind the exact fragment identities")
    actual = {ROOT_NAME: root}
    for name in wanted - {ROOT_NAME}:
        value, raw_sha256 = load(root_path.parent / name, reject)
        self_hash(value, "artifact_sha256", reject)
        if receipts[name].get("serialized_sha256") != raw_sha256:
            reject("classification receipt does not bind the exact fragment bytes")
        if not exact(value, expected[name]):
            reject("classification fragment is stale, reordered, partial, or mixed")
        actual[name] = value
    return materialize(actual)
