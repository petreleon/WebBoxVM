"""Generation-recipe parsing for the isolated successor fixture."""

from __future__ import annotations

from pathlib import PurePosixPath

from successor_identity_model import (
    CONFIG_FIELDS,
    GENERATION_FIELDS,
    RECIPE_FIELDS,
    TOOLCHAIN_FIELDS,
    Generation,
    reject,
)
from successor_identity_parse import canonical_digest, digest, identifier, relative_path, string_list, strings, text


def recipe(value: object) -> str:
    if not isinstance(value, dict) or set(value) != RECIPE_FIELDS:
        reject("generation recipe has an invalid schema")
    argv = string_list(value.get("argv"), "recipe argv")
    if any("\x00" in item or PurePosixPath(item).is_absolute() or ".." in PurePosixPath(item).parts for item in argv):
        reject("generation recipe has an unsafe argv")
    relative_path(value.get("workdir"), "recipe workdir")
    return canonical_digest(value, "sha256", "generation recipe")


def configuration(value: object) -> str:
    if not isinstance(value, dict) or set(value) != CONFIG_FIELDS:
        reject("generation configuration has an invalid schema")
    attributes = value.get("attributes")
    if not isinstance(attributes, list) or not attributes:
        reject("generation configuration has no attributes")
    pairs = []
    for item in attributes:
        if not isinstance(item, dict) or set(item) != {"name", "value"}:
            reject("generation configuration has an invalid attribute")
        name = text(item.get("name"), "attribute name")
        pairs.append((name, text(item.get("value"), "attribute value")))
    names = tuple(name for name, _ in pairs)
    if names != tuple(sorted(names)) or len(set(names)) != len(names):
        reject("generation configuration attributes are not canonical")
    return canonical_digest(value, "sha256", "generation configuration")


def generation(value: object) -> Generation:
    if not isinstance(value, dict) or set(value) != GENERATION_FIELDS:
        reject("generation has an invalid schema")
    generation_id = identifier(value.get("id"), "generation id")
    recipe_digest = recipe(value.get("recipe"))
    config = configuration(value.get("configuration"))
    toolchain = value.get("toolchain")
    if not isinstance(toolchain, dict) or set(toolchain) != TOOLCHAIN_FIELDS:
        reject("generation toolchain has an invalid schema")
    toolchain_name = text(toolchain.get("name"), "toolchain name")
    toolchain_version = text(toolchain.get("version"), "toolchain version")
    toolchain_digest = digest(toolchain.get("sha256"), "toolchain sha256")
    outputs = strings(value.get("output_member_ids"), "generation outputs")
    tree = digest(value.get("output_tree_sha256"), "generation output tree")
    runs = value.get("clean_run_tree_sha256s")
    if not isinstance(runs, list) or len(runs) != 2:
        reject("generation needs exactly two clean-run trees")
    clean = tuple(digest(item, "clean-run tree") for item in runs)
    if clean[0] != clean[1] or clean[0] != tree:
        reject("generation clean runs are not deterministic")
    return Generation(generation_id, recipe_digest, config, toolchain_name, toolchain_version,
                      toolchain_digest, outputs, tree, clean)
