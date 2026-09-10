"""Strictly normalize caller-supplied VCTS Git-tree metadata without transport."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCHEMA = HERE.parent.parent / "02-canonical-suite-schema"
HEX40 = re.compile(r"^[0-9a-f]{40}$")
INPUT_KIND = "vcts-git-api-tree-input"
PLAN_KIND = "vcts-immutable-git-tree-plan"
TREE_NAMES = ("external", "vulkancts", "mustpass", "main", "vk-default")
MAX_PLAN_BYTES = 256 * 1024
MAX_MEMBER_BYTES = 64 * 1024 * 1024
MAX_TOTAL_BYTES = 512 * 1024 * 1024
INPUT_FIELDS = frozenset(("schema", "kind", "tag_ref", "tag_object_sha1", "tag_target_sha1",
                          "tag_target_type", "peeled_commit_sha1", "commit_tree_sha1", "tree_chain",
                          "root", "members"))
PLAN_FIELDS = frozenset(("schema", "kind", "root_identity_sha256", "tag_ref", "tag_object_sha1",
                         "tag_target_sha1", "tag_target_type", "peeled_commit_sha1", "commit_tree_sha1",
                         "tree_chain", "root_parent_tree_sha1", "members_tree_sha1", "root", "member_count",
                         "member_total_bytes", "members", "plan_sha256"))
LINK_FIELDS = frozenset(("name", "mode", "parent_tree_sha1", "tree_sha1"))
ROOT_FIELDS = frozenset(("path", "mode", "blob_sha1", "bytes"))


class TreePlanError(ValueError):
    """A supplied VCTS Git-tree metadata plan is unsafe, partial, or stale."""


def reject(message: str) -> None:
    raise TreePlanError(message)


def pairs(items: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for key, item in items:
        if key in value:
            reject("tree plan has a duplicate JSON key")
        value[key] = item
    return value


def load_identity():
    spec = importlib.util.spec_from_file_location("f025_v2_tree_plan_identity", SCHEMA / "canonical_suite_identity.py")
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load the reviewed V2 suite identity")
    value = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = value
    spec.loader.exec_module(value)
    return value


identity = load_identity()


def digest(value: dict[str, object]) -> str:
    body = {key: item for key, item in value.items() if key != "plan_sha256"}
    return hashlib.sha256(json.dumps(body, sort_keys=True, separators=(",", ":")).encode("utf-8")).hexdigest()


def document(path: Path) -> dict[str, object]:
    try:
        raw = path.read_bytes()
        if len(raw) > MAX_PLAN_BYTES:
            reject("tree plan exceeds its JSON byte limit")
        value = json.loads(raw.decode("utf-8"), object_pairs_hook=pairs)
    except TreePlanError:
        raise
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"tree plan cannot be read: {error}")
    if not isinstance(value, dict):
        reject("tree plan is not an object")
    return value


def exact(value: object, fields: frozenset[str], label: str) -> dict[str, object]:
    if not isinstance(value, dict) or set(value) != fields:
        reject(f"{label} has an unexpected schema")
    return value


def sha1(value: object, label: str) -> str:
    if not isinstance(value, str) or not HEX40.fullmatch(value) or value == "0" * 40:
        reject(f"{label} is not a nonzero Git SHA-1")
    return value


def positive(value: object, limit: int, label: str) -> int:
    if type(value) is not int or not 1 <= value <= limit:
        reject(f"{label} is outside its byte limit")
    return value


def checked_identity(path: Path):
    try:
        return identity.validate(path)
    except identity.IdentityError as error:
        reject(f"canonical V2 identity failed: {error}")


def tree_chain(value: object, commit_tree: str) -> list[dict[str, object]]:
    if not isinstance(value, list) or len(value) != len(TREE_NAMES):
        reject("tree plan has an incomplete named tree chain")
    chain, parent = [], commit_tree
    for name, item in zip(TREE_NAMES, value):
        row = exact(item, LINK_FIELDS, "named tree entry")
        if row["name"] != name or row["mode"] != "040000" or row["parent_tree_sha1"] != parent:
            reject("named tree chain has a substituted path or mode")
        child = sha1(row["tree_sha1"], "tree SHA-1")
        chain.append({"name": name, "mode": "040000", "parent_tree_sha1": parent, "tree_sha1": child})
        parent = child
    return chain


def root_record(value: object, root) -> dict[str, object]:
    row = exact(value, ROOT_FIELDS, "root blob")
    if row["path"] != root.root_path or row["mode"] != "100644" or row["bytes"] != identity.ROOT["bytes"]:
        reject("root blob does not bind the reviewed selector path, mode, and bytes")
    return {"path": root.root_path, "mode": "100644", "blob_sha1": sha1(row["blob_sha1"], "root blob SHA-1"),
            "bytes": identity.ROOT["bytes"]}


def member_records(value: object, root) -> tuple[list[dict[str, object]], int]:
    if not isinstance(value, list) or len(value) != root.direct_member_count:
        reject("tree plan does not contain exactly the pinned direct members")
    rows, total = [], 0
    for expected_path, item in zip(root.direct_members, value):
        row = exact(item, ROOT_FIELDS, "direct member")
        try:
            safe_path = identity.safe_path(row["path"])
        except identity.IdentityError as error:
            reject(f"direct member path is unsafe: {error}")
        if safe_path != expected_path or row["mode"] != "100644":
            reject("direct members are reordered, filtered, or use a non-regular mode")
        size = positive(row["bytes"], MAX_MEMBER_BYTES, "direct member bytes")
        total += size
        rows.append({"path": expected_path, "mode": "100644", "blob_sha1": sha1(row["blob_sha1"], "member blob SHA-1"),
                     "bytes": size})
    if total > MAX_TOTAL_BYTES:
        reject("tree plan exceeds the aggregate byte limit")
    return rows, total


def normalized(api_input: object, root) -> dict[str, object]:
    value = exact(api_input, INPUT_FIELDS, "Git-tree API input")
    if type(value["schema"]) is not int or value["schema"] != 1 or value["kind"] != INPUT_KIND:
        reject("Git-tree API input has an unsupported schema")
    expected = {"tag_ref": identity.EXPECTED["tag_ref"], "tag_object_sha1": identity.EXPECTED["tag_object_sha1"],
                "tag_target_sha1": root.peeled_commit, "tag_target_type": "commit", "peeled_commit_sha1": root.peeled_commit}
    if any(value[key] != item for key, item in expected.items()):
        reject("Git-tree API input does not bind the reviewed annotated tag and peeled commit")
    commit_tree = sha1(value["commit_tree_sha1"], "commit tree SHA-1")
    chain, (rows, total) = tree_chain(value["tree_chain"], commit_tree), member_records(value["members"], root)
    return {"schema": 1, "kind": PLAN_KIND, "root_identity_sha256": root.digest, **expected,
            "commit_tree_sha1": commit_tree, "tree_chain": chain, "root_parent_tree_sha1": chain[3]["tree_sha1"],
            "members_tree_sha1": chain[-1]["tree_sha1"], "root": root_record(value["root"], root),
            "member_count": len(rows), "member_total_bytes": total, "members": rows}
