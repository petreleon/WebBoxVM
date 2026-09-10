"""Bind GitHub Git metadata to the reviewed VCTS identity and tree-plan builder."""
from __future__ import annotations

import sys
import re
from pathlib import Path, PurePosixPath
from typing import NamedTuple

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import vcts_tree_plan as plan
from vcts_github_tree_http import GitHubTreeError, endpoint, json_document, ref_endpoint, reject, sha1

MAX_ENTRIES = 512
PART = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]*$")


def tree_document(value: object, requested: str) -> list[dict[str, object]]:
    if not isinstance(value, dict) or value.get("sha") != requested or value.get("truncated") is not False:
        reject("GitHub tree response is substituted or truncated")
    rows = value.get("tree")
    if not isinstance(rows, list) or len(rows) > MAX_ENTRIES or any(not isinstance(row, dict) for row in rows):
        reject("GitHub tree response has an invalid entry list")
    return rows


def one(rows: list[dict[str, object]], name: str, mode: str, kind: str) -> dict[str, object]:
    hits = [row for row in rows if row.get("path") == name]
    if len(hits) != 1 or hits[0].get("mode") != mode or hits[0].get("type") != kind:
        reject(f"GitHub tree entry {name} is missing or substituted")
    sha1(hits[0].get("sha"), f"GitHub tree entry {name}")
    return hits[0]


def relative(value: object) -> str:
    if not isinstance(value, str) or not value or value.startswith("/"):
        reject("GitHub recursive tree has an unsafe path")
    path = PurePosixPath(value)
    if str(path) != value or any(not PART.fullmatch(item) for item in path.parts):
        reject("GitHub recursive tree has an unsafe path")
    return value


def members(rows: list[dict[str, object]], root) -> list[dict[str, object]]:
    prefix = root.root_path.removesuffix(".txt") + "/"
    expected = {path.removeprefix(prefix): path for path in root.direct_members}
    parents = {"/".join(name.split("/")[:depth]) for name in expected for depth in range(1, name.count("/") + 1)}
    found: dict[str, dict[str, object]] = {}
    seen: set[str] = set()
    for row in rows:
        name = relative(row.get("path"))
        if name in seen:
            reject("GitHub recursive tree repeats an entry")
        seen.add(name)
        if name in expected:
            size = row.get("size")
            if (row.get("mode") != "100644" or row.get("type") != "blob" or type(size) is not int
                    or not 1 <= size <= plan.MAX_MEMBER_BYTES):
                reject("GitHub member entry has an invalid mode, type, or size")
            sha1(row.get("sha"), "GitHub member blob")
            found[name] = row
        elif name in parents:
            if row.get("mode") != "040000" or row.get("type") != "tree":
                reject("GitHub recursive tree parent is substituted")
            sha1(row.get("sha"), "GitHub recursive tree parent")
        else:
            reject("GitHub recursive tree has an unselected entry")
    if set(found) != set(expected):
        reject("GitHub recursive tree is missing a pinned direct member")
    return [{"path": path, "mode": "100644", "blob_sha1": found[name]["sha"], "bytes": found[name]["size"]}
            for name, path in ((item.removeprefix(prefix), item) for item in root.direct_members)]


def fetch_api_input(identity_path: Path, timeout: float = 30.0, opener=None) -> dict[str, object]:
    try:
        root = plan.checked_identity(identity_path)
    except plan.TreePlanError as error:
        reject(f"canonical V2 identity failed: {error}")
    expected = plan.identity.EXPECTED
    tag_sha, commit_sha = expected["tag_object_sha1"], root.peeled_commit
    ref = json_document(ref_endpoint(expected["tag_name"]), timeout, opener)
    ref_object = ref.get("object")
    if (ref.get("ref") != expected["tag_ref"] or not isinstance(ref_object, dict)
            or ref_object.get("type") != "tag" or ref_object.get("sha") != tag_sha):
        reject("GitHub named tag ref does not bind the pinned tag object")
    tag = json_document(endpoint("tags", tag_sha), timeout, opener)
    target = tag.get("object")
    if (tag.get("sha") != tag_sha or tag.get("tag") != expected["tag_name"] or not isinstance(target, dict)
            or target.get("sha") != commit_sha or target.get("type") != "commit"):
        reject("GitHub annotated tag does not bind the pinned commit")
    commit = json_document(endpoint("commits", commit_sha), timeout, opener)
    commit_tree = commit.get("tree")
    if commit.get("sha") != commit_sha or not isinstance(commit_tree, dict):
        reject("GitHub commit response is substituted")
    parent, chain, final = sha1(commit_tree.get("sha"), "GitHub commit tree"), [], []
    for name in plan.TREE_NAMES:
        rows = tree_document(json_document(endpoint("trees", parent), timeout, opener), parent)
        entry = one(rows, name, "040000", "tree")
        if name == plan.TREE_NAMES[-1]:
            final = rows
        child = sha1(entry["sha"], f"GitHub {name} tree")
        chain.append({"name": name, "mode": "040000", "parent_tree_sha1": parent, "tree_sha1": child})
        parent = child
    root_row = one(final, root.root_path.rsplit("/", 1)[1], "100644", "blob")
    if type(root_row.get("size")) is not int or root_row["size"] != plan.identity.ROOT["bytes"]:
        reject("GitHub root blob has an invalid size")
    leaf_rows = tree_document(json_document(endpoint("trees", parent, True), timeout, opener), parent)
    return {"schema": 1, "kind": plan.INPUT_KIND, "tag_ref": expected["tag_ref"], "tag_object_sha1": tag_sha,
            "tag_target_sha1": commit_sha, "tag_target_type": "commit", "peeled_commit_sha1": commit_sha,
            "commit_tree_sha1": chain[0]["parent_tree_sha1"], "tree_chain": chain,
            "root": {"path": root.root_path, "mode": "100644", "blob_sha1": root_row["sha"], "bytes": root_row["size"]},
            "members": members(leaf_rows, root)}


class GitHubTreeCapture(NamedTuple):
    api_input: dict[str, object]
    tree_plan: dict[str, object]


def capture(identity_path: Path, timeout: float = 30.0, opener=None) -> GitHubTreeCapture:
    """Fetch metadata and return normalized builder input plus its immutable plan."""
    source = fetch_api_input(identity_path, timeout, opener)
    try:
        return GitHubTreeCapture(source, plan.build(source, identity_path))
    except plan.TreePlanError as error:
        reject(f"GitHub metadata cannot form an immutable tree plan: {error}")
