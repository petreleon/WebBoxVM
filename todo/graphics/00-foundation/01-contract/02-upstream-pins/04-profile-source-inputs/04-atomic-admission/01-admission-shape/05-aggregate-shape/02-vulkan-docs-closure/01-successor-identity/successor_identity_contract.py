#!/usr/bin/env python3
"""Validate an isolated raw/generated successor-closure fixture, never admission."""

from __future__ import annotations

import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))
from successor_identity_generation import generation
from successor_identity_model import (
    FIXTURE_FIELDS,
    GENERATED,
    RAW,
    ClosureCachePlan,
    FixtureClosure,
    GeneratedMember,
    IdentityError,
    reject,
)
from successor_identity_parse import canonical, document, digest, generated, identifier, raw, text
from successor_identity_predecessor import adapter
from successor_identity_scope import scope

FIXTURE = HERE / "successor_identity.fixture.json"
PREDECESSOR_PATHS = {
    "rules": HERE.parents[2] / "04-post-cutover-rules/post_cutover_rules.json",
    "audit": HERE.parents[4] / "03-vulkan-input-audit/candidates.json",
    "boundary": HERE.parents[2] / "03-vulkan-boundaries/boundaries.json",
}

def members(value: object, closure: str):
    if not isinstance(value, list) or len(value) < 2:
        reject("fixture needs at least two ordered members")
    parsed = []
    for row in value:
        kind = row.get("kind") if isinstance(row, dict) else None
        if kind == RAW:
            member = raw(row, closure)
        elif kind == GENERATED:
            member = generated(row, closure)
        else:
            reject("fixture has an unknown member kind")
        parsed.append(member)
    for name in ("identifier", "digest", "selector", "cache_path"):
        rows = tuple(getattr(item, name) for item in parsed)
        if len(set(rows)) != len(rows):
            reject(f"fixture repeats a member {name}")
    seen = set()
    for item in parsed:
        if isinstance(item, GeneratedMember) and any(source not in seen for source in item.producers):
            reject("generated member producers are not earlier closure members")
        seen.add(item.identifier)
    return tuple(parsed)


def generations(value: object, members) -> tuple:
    if not isinstance(value, list) or not value:
        reject("fixture has no generation identities")
    rows = tuple(generation(item) for item in value)
    if len({item.identifier for item in rows}) != len(rows):
        reject("fixture has duplicate generation identities")
    generated_members = tuple(item for item in members if isinstance(item, GeneratedMember))
    outputs = tuple(output for row in rows for output in row.output_ids)
    if outputs != tuple(item.identifier for item in generated_members):
        reject("generation outputs do not exactly cover generated members")
    by_id = {item.identifier: item for item in generated_members}
    generation_ids = {row.identifier for row in rows}
    if any(item.generation_id not in generation_ids for item in generated_members):
        reject("generated member has no generation identity")
    for row in rows:
        if any(by_id[item].generation_id != row.identifier for item in row.output_ids):
            reject("generation output belongs to a different identity")
        tree = [
            {"id": by_id[item].identifier, "selector": by_id[item].selector,
             "sha256": by_id[item].digest,
             "bytes": by_id[item].byte_count}
            for item in row.output_ids
        ]
        if canonical(tree) != row.output_tree_digest:
            reject("generation output tree is stale")
    return rows


def validate(fixture_path: Path = FIXTURE, predecessor_paths: object = PREDECESSOR_PATHS) -> FixtureClosure:
    fixture = document(fixture_path)
    if set(fixture) != FIXTURE_FIELDS or type(fixture.get("schema")) is not int or fixture.get("schema") != 1:
        reject("fixture has an invalid root schema")
    state = (fixture.get("contract"), fixture.get("status"))
    if state != ("successor-raw-generated-fixture-v1", "fixture-only-unadmitted"):
        reject("fixture could be mistaken for active admission")
    logical_id = identifier(fixture.get("required_input_id"), "fixture logical id")
    profile = text(fixture.get("profile"), "fixture profile")
    role = text(fixture.get("role"), "fixture role")
    if (profile, role, logical_id) != (
            "fixture-vulkan-docs", "fixture-only", "fixture-raw-generated-closure"):
        reject("fixture must retain its synthetic profile identity")
    parsed_members = members(fixture.get("members"), logical_id)
    root = fixture.get("root_member_id")
    if root != parsed_members[0].identifier or isinstance(parsed_members[0], GeneratedMember):
        reject("fixture root must be the first raw member")
    parsed_generations = generations(fixture.get("generations"), parsed_members)
    parsed_scope = scope(fixture.get("scope"), parsed_members, parsed_generations)
    predecessor = adapter(fixture.get("predecessor_adapter"), predecessor_paths)
    expected = canonical({key: value for key, value in fixture.items() if key != "closure_sha256"})
    closure_digest = digest(fixture.get("closure_sha256"), "closure sha256")
    if closure_digest != expected:
        reject("fixture closure has a stale digest")
    plan = ClosureCachePlan(logical_id, closure_digest, parsed_members, parsed_generations,
                            parsed_scope, predecessor)
    return FixtureClosure(plan)


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: successor_identity_contract.py FIXTURE.json")
    try:
        result = validate(Path(sys.argv[1]))
    except IdentityError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    summary = (
        f"FIXTURE: {result.state}, {len(result.member_ids)} members, "
        f"{len(result.generated_ids)} generated, 0 cutover-ready"
    )
    print(summary)


if __name__ == "__main__":
    main()
