"""Canonical, self-hashed completion markers for successor closures."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

from successor_cache_fs import atomic_file, read_file
from successor_cache_model import CacheMiss, reject
from successor_cache_store import target

FIELDS = frozenset((
    "schema", "kind", "status", "logical_id", "closure_sha256", "members",
    "generations", "scope", "predecessor", "marker_sha256",
))
MARKER_LIMIT = 4 * 1024 * 1024


def canonical(value: object) -> str:
    raw = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(raw).hexdigest()


def member(value) -> dict[str, object]:
    common = {
        "id": value.identifier, "selector": value.selector, "sha256": value.digest,
        "bytes": value.byte_count, "license": value.license, "local_cache": value.cache_path,
        "generated_code_role": value.generated_code_role, "provenance": value.provenance,
    }
    if hasattr(value, "immutable_url"):
        return {
            "kind": "raw-member", "source_family": value.source_family,
            "immutable_url": value.immutable_url, "revision": value.revision, **common,
        }
    return {
        "kind": "generated-member", "producer_member_ids": list(value.producers),
        "generation_id": value.generation_id, **common,
    }


def generation(value) -> dict[str, object]:
    return {
        "id": value.identifier, "recipe_sha256": value.recipe_digest,
        "configuration_sha256": value.configuration_digest,
        "toolchain_name": value.toolchain_name, "toolchain_version": value.toolchain_version,
        "toolchain_sha256": value.toolchain_digest, "output_member_ids": list(value.output_ids),
        "output_tree_sha256": value.output_tree_digest,
        "clean_run_tree_sha256s": list(value.clean_run_tree_digests),
    }


def value(plan) -> dict[str, object]:
    predecessor = plan.predecessor
    result = {
        "schema": 1, "kind": "successor-closure-cache-marker-v1",
        "status": "fixture-cache-only-unadmitted", "logical_id": plan.logical_id,
        "closure_sha256": plan.closure_digest,
        "members": [member(item) for item in plan.members],
        "generations": [generation(item) for item in plan.generations],
        "scope": {
            "ordered_member_ids": list(plan.member_ids), "generated_member_ids": list(plan.generated_ids),
            "configuration_sha256": plan.scope.configuration_digest,
            "scope_sha256": plan.scope.scope_digest,
        },
        "predecessor": {
            "kind": "v1-predecessor-only", "candidate_root_id": predecessor.candidate_root_id,
            "candidate_root_sha256": predecessor.candidate_root_digest,
            "rules_sha256": predecessor.rules_digest, "audit_sha256": predecessor.audit_digest,
            "boundary_sha256": predecessor.boundary_digest, "decision": predecessor.decision,
        },
    }
    result["marker_sha256"] = canonical(result)
    return result


def bytes_for(plan) -> bytes:
    return (json.dumps(value(plan), sort_keys=True, separators=(",", ":")) + "\n").encode()


def marker_relative(plan) -> str:
    return f"webboxvm-graphics/successor/{plan.logical_id}/markers/{plan.closure_digest}.complete.json"


def lock_relative(plan) -> str:
    return f"webboxvm-graphics/successor/{plan.logical_id}/locks/{plan.closure_digest}.lock"


def location(root: Path, plan) -> Path:
    return target(root, marker_relative(plan))


def unique_object(pairs):
    result = {}
    for key, item in pairs:
        if key in result:
            reject("successor marker has duplicate JSON object fields")
        result[key] = item
    return result


def read(root: Path, plan) -> dict[str, object]:
    encoded = read_file(
        root, marker_relative(plan), "successor closure marker", maximum_bytes=MARKER_LIMIT, optional=True,
    )
    if encoded is None:
        raise CacheMiss("successor closure marker is missing")
    try:
        parsed = json.loads(encoded.decode("utf-8"), object_pairs_hook=unique_object)
    except (UnicodeDecodeError, json.JSONDecodeError, RecursionError, ValueError, OverflowError) as error:
        reject(f"successor marker cannot be read: {error}")
    if not isinstance(parsed, dict) or set(parsed) != FIELDS:
        reject("successor marker has an invalid schema")
    if (type(parsed.get("schema")) is not int or parsed.get("schema") != 1
            or parsed.get("kind") != "successor-closure-cache-marker-v1"
            or parsed.get("status") != "fixture-cache-only-unadmitted"):
        reject("successor marker claims an invalid state")
    try:
        digest = parsed.get("marker_sha256")
        actual = canonical({key: item for key, item in parsed.items() if key != "marker_sha256"})
    except (RecursionError, ValueError, OverflowError) as error:
        reject(f"successor marker cannot be read: {error}")
    if not isinstance(digest, str) or digest != actual:
        reject("successor marker has a stale self digest")
    return parsed


def exact(root: Path, plan) -> None:
    if read(root, plan) != value(plan):
        reject("successor marker does not exactly match the validated closure plan")


def publish(root: Path, plan) -> Path:
    return atomic_file(root, marker_relative(plan), bytes_for(plan), "successor closure marker")
