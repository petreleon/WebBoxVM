#!/usr/bin/env python3
"""Public builder and validator for the hermetic VCTS Git-tree plan contract."""

from __future__ import annotations

import sys
from dataclasses import dataclass
from pathlib import Path

from vcts_tree_plan_input import (INPUT_FIELDS, INPUT_KIND, MAX_MEMBER_BYTES, PLAN_FIELDS, SCHEMA,
                                  TREE_NAMES, TreePlanError, checked_identity, digest, document, identity,
                                  normalized, reject)


def build(api_input: object, identity_path: Path) -> dict[str, object]:
    """Return a self-hashed plan from normalized, caller-supplied Git-tree metadata."""
    value = normalized(api_input, checked_identity(identity_path))
    value["plan_sha256"] = digest(value)
    return value


@dataclass(frozen=True)
class TreePlan:
    digest: str
    commit_tree_sha1: str
    root_blob_sha1: str
    member_count: int
    total_bytes: int


def validate(path: Path, identity_path: Path) -> TreePlan:
    value, root = document(path), checked_identity(identity_path)
    if not isinstance(value, dict) or set(value) != PLAN_FIELDS:
        reject("tree plan has an unexpected schema")
    if value.get("root_identity_sha256") != root.digest:
        reject("tree plan is stale against the canonical V2 identity")
    source = {key: value[key] for key in INPUT_FIELDS}
    source["kind"] = INPUT_KIND
    expected = normalized(source, root)
    expected["plan_sha256"] = digest(expected)
    if value != expected:
        reject("tree plan is stale or has substituted metadata")
    return TreePlan(expected["plan_sha256"], expected["commit_tree_sha1"], expected["root"]["blob_sha1"],
                    expected["member_count"], expected["member_total_bytes"])


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit("usage: vcts_tree_plan.py IDENTITY.json TREE_PLAN.json")
    try:
        plan = validate(Path(sys.argv[2]), Path(sys.argv[1]))
    except TreePlanError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"TREE-PLAN: {plan.member_count} members {plan.total_bytes} bytes {plan.digest}")


if __name__ == "__main__":
    main()
