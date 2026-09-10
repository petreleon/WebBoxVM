#!/usr/bin/env python3
"""Freeze V2's diagnostic-only Vulkan CTS coverage taxonomy."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
from dataclasses import dataclass
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCHEMA = HERE.parent / "02-canonical-suite-schema"
CATEGORIES = ("core", "wsi", "video", "extension", "unknown")
FIELDS = frozenset(("schema", "kind", "suite_identity_sha256", "suite_revision",
                    "direct_member_paths_sha256", "categories", "rules", "fallback", "taxonomy_sha256"))
RULE_FIELDS = frozenset(("id", "category", "path", "source_locator"))
FALLBACK_FIELDS = frozenset(("id", "category", "reason", "source_locator"))
EXPECTED_TAXONOMY_SHA256 = "752a3ae5ff10ea0d9f9a2638c0d1612fb7af3d376dabad1601f8b9c1e76cb2f7"
REPOSITORY_URL = "https://github.com/KhronosGroup/VK-GL-CTS"
RULES = (
    ("wsi-selector", "wsi", "vk-default/wsi.txt"),
    ("video-selector", "video", "vk-default/video.txt"),
    ("cooperative-vector-selector", "extension", "vk-default/cooperative-vector.txt"),
    ("data-graph-selector", "extension", "vk-default/data-graph.txt"),
    ("ray-query-selector", "extension", "vk-default/ray-query.txt"),
    ("tensor-selector", "extension", "vk-default/tensor.txt"),
)


def load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load V2 schema module: {path}")
    value = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = value
    spec.loader.exec_module(value)
    return value


identity = load("f025_v2_taxonomy_identity", SCHEMA / "canonical_suite_identity.py")
ledger = load("f025_v2_taxonomy_ledger", SCHEMA / "canonical_suite_ledger.py")


class TaxonomyError(ValueError):
    """The taxonomy or an ordered coverage view is stale or incomplete."""


def reject(message: str) -> None:
    raise TaxonomyError(message)


def no_duplicates(pairs: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for key, item in pairs:
        if key in value:
            reject("taxonomy has a duplicate JSON key")
        value[key] = item
    return value


def document(path: Path) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=no_duplicates)
    except TaxonomyError:
        raise
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"taxonomy cannot be read: {error}")
    if not isinstance(value, dict):
        reject("taxonomy is not an object")
    return value


def digest(value: dict[str, object]) -> str:
    body = {key: item for key, item in value.items() if key != "taxonomy_sha256"}
    return hashlib.sha256(json.dumps(body, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def source_locator(root, path: str) -> str:
    return f"{REPOSITORY_URL}/blob/{root.peeled_commit}/{path}"


def expected_rules(root) -> list[dict[str, str]]:
    prefix = "external/vulkancts/mustpass/main/"
    return [{"id": identifier, "category": category, "path": prefix + suffix,
             "source_locator": source_locator(root, prefix + suffix)} for identifier, category, suffix in RULES]


@dataclass(frozen=True)
class Taxonomy:
    digest: str
    rules: dict[str, dict[str, str]]
    fallback_id: str


@dataclass(frozen=True)
class CoverageView:
    identity_digest: str
    revision: str
    ledger_digest: str
    taxonomy_digest: str
    members: tuple[dict[str, str], ...]


def validate(path: Path, identity_path: Path) -> Taxonomy:
    try:
        root = identity.validate(identity_path)
    except identity.IdentityError as error:
        reject(f"taxonomy root identity failed: {error}")
    value = document(path)
    if (set(value) != FIELDS or type(value.get("schema")) is not int or value.get("schema") != 1
            or value.get("kind") != "vulkan-cts-coverage-taxonomy"):
        reject("taxonomy has an unexpected schema")
    if (value.get("suite_identity_sha256") != root.digest or value.get("suite_revision") != root.peeled_commit
            or value.get("direct_member_paths_sha256") != identity.paths_digest(root.direct_members)):
        reject("taxonomy does not bind the reviewed suite root")
    categories = value.get("categories")
    if not isinstance(categories, dict) or set(categories) != set(CATEGORIES):
        reject("taxonomy categories are not exhaustive")
    for name, category in categories.items():
        if (not isinstance(category, dict) or set(category) != {"core_eligible", "meaning"}
                or type(category["core_eligible"]) is not bool or not isinstance(category["meaning"], str)
                or not category["meaning"] or category["core_eligible"] != (name == "core")):
            reject("taxonomy category has an unsafe core boundary")
    rules = value.get("rules")
    if (not isinstance(rules, list) or not all(isinstance(rule, dict) and set(rule) == RULE_FIELDS for rule in rules)
            or rules != expected_rules(root)):
        reject("taxonomy rules are stale, incomplete, or reclassified")
    fallback = value.get("fallback")
    if (not isinstance(fallback, dict) or set(fallback) != FALLBACK_FIELDS or fallback.get("id") != "unknown-by-default"
            or fallback.get("category") != "unknown" or not isinstance(fallback.get("reason"), str)
            or fallback.get("source_locator") != source_locator(root, root.root_path)):
        reject("taxonomy fallback is not the reviewed unknown boundary")
    claimed = value.get("taxonomy_sha256")
    if not isinstance(claimed, str) or claimed != digest(value) or claimed != EXPECTED_TAXONOMY_SHA256:
        reject("taxonomy has a stale canonical digest")
    return Taxonomy(claimed, {rule["path"]: rule for rule in rules}, str(fallback["id"]))


def classify(taxonomy_path: Path, identity_path: Path, ledger_path: Path) -> CoverageView:
    taxonomy = validate(taxonomy_path, identity_path)
    try:
        suite = ledger.validate(ledger_path, identity_path)
        value = ledger.document(ledger_path)
    except ledger.LedgerError as error:
        reject(f"coverage ledger failed: {error}")
    if ledger.digest(value) != suite.digest:
        reject("coverage ledger changed during classification")
    rows: list[dict[str, str]] = []
    root = identity.validate(identity_path)
    for member in value["members"]:
        assert isinstance(member, dict)
        rule = taxonomy.rules.get(str(member["path"]))
        rows.append({"path": str(member["path"]), "blob_sha1": str(member["blob_sha1"]),
                     "member_sha256": str(member["sha256"]), "category": rule["category"] if rule else "unknown",
                     "rule_id": rule["id"] if rule else taxonomy.fallback_id,
                     "source_locator": source_locator(root, str(member["path"]))})
    return CoverageView(root.digest, root.peeled_commit, suite.digest, taxonomy.digest, tuple(rows))


def main() -> None:
    if len(sys.argv) != 4:
        raise SystemExit("usage: coverage_taxonomy.py TAXONOMY.json IDENTITY.json LEDGER.json")
    try:
        view = classify(Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3]))
    except TaxonomyError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"TAXONOMY: {len(view.members)} ordered members {view.ledger_digest} {view.taxonomy_digest}")


if __name__ == "__main__":
    main()
