#!/usr/bin/env python3
"""Fail closed around an actual Docs build witness and future unadmitted closure."""

from __future__ import annotations

import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))
PREDECESSOR_DIR = HERE.parent.parent / "01-successor-identity"
if str(PREDECESSOR_DIR) not in sys.path:
    sys.path.insert(0, str(PREDECESSOR_DIR))

from successor_identity_predecessor import adapter
from vulkan_docs_identity_build import output_tree, outputs, root, build
from vulkan_docs_identity_model import (
    CLOSURE_FIELDS,
    WITNESS_FIELDS,
    ActualClosure,
    BuildWitness,
    DocsIdentityError,
    reject,
)
from vulkan_docs_identity_parse import digest, document, self_digest
from vulkan_docs_identity_scope import inputs, scope

WITNESS = HERE / "vulkan_docs_build_witness.json"
PREDECESSOR_PATHS = {
    "rules": HERE.parents[3] / "04-post-cutover-rules/post_cutover_rules.json",
    "audit": HERE.parents[5] / "03-vulkan-input-audit/candidates.json",
    "boundary": HERE.parents[3] / "03-vulkan-boundaries/boundaries.json",
}


def predecessor(value: object):
    try:
        return adapter(value, PREDECESSOR_PATHS)
    except Exception as error:
        reject(f"Docs witness cannot preserve the reviewed predecessor: {error}")


def witness_value(value: object) -> BuildWitness:
    if not isinstance(value, dict) or set(value) != WITNESS_FIELDS:
        reject("Docs build witness has an invalid schema")
    if type(value.get("schema")) is not int or tuple(value.get(name) for name in (
            "schema", "contract", "status", "profile", "role", "required_input_id")) != (
            1, "vulkan-docs-build-witness-v1", "build-witness-only-unadmitted", "vulkan-1.4-core",
            "api-limit-format-spec", "vulkan-14-spec"):
        reject("Docs build witness can be mistaken for a closure or active consumer")
    root_input = root(value.get("root"))
    predecessor_value = predecessor(value.get("predecessor_adapter"))
    if (predecessor_value.candidate_root_id != root_input.identifier
            or predecessor_value.candidate_root_digest != root_input.digest):
        reject("Docs build witness root does not preserve the rejected predecessor root")
    recipe = build(value.get("build"))
    rendered = outputs(value.get("outputs"), recipe)
    tree_digest = output_tree(value.get("output_tree"), rendered)
    return BuildWitness(
        self_digest(value, "witness_sha256", "Docs build witness"), root_input, recipe, rendered, tree_digest)


def witness(path: Path = WITNESS) -> BuildWitness:
    return witness_value(document(path))


def closure_value(value: object, build_witness: BuildWitness) -> ActualClosure:
    if not isinstance(value, dict) or set(value) != CLOSURE_FIELDS:
        reject("Docs closure has an invalid schema")
    if type(value.get("schema")) is not int or tuple(value.get(name) for name in (
            "schema", "contract", "status", "profile", "role", "required_input_id")) != (
            1, "vulkan-docs-actual-closure-v1", "unadmitted-actual-docs-closure", "vulkan-1.4-core",
            "api-limit-format-spec", "vulkan-14-spec"):
        reject("Docs closure attempts an active or unrelated state")
    members = inputs(value.get("inputs"))
    if value.get("root_input_id") != members[0].identifier or members[0] != build_witness.root:
        reject("Docs closure does not retain the witness root as its first source input")
    scope(value.get("scope"), members, build_witness.recipe)
    if digest(value.get("build_witness_sha256"), "Docs closure witness sha256") != build_witness.digest:
        reject("Docs closure does not bind the exact build witness")
    closure_digest = self_digest(value, "closure_sha256", "Docs closure")
    return ActualClosure(closure_digest, members, build_witness.digest)


def closure(path: Path, witness_path: Path = WITNESS) -> ActualClosure:
    return closure_value(document(path), witness(witness_path))


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: vulkan_docs_identity_contract.py WITNESS.json")
    try:
        result = witness(Path(sys.argv[1]))
    except DocsIdentityError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"WITNESS: {result.state}, {len(result.outputs)} rendered output, 0 cutover-ready")


if __name__ == "__main__":
    main()
