"""Validate the exact pinned-container build witness and rendered-output bounds."""

from __future__ import annotations

from vulkan_docs_identity_members import DOCS_COMMIT, NAMESPACE, raw, rendered_outputs
from vulkan_docs_identity_model import (
    BUILD_FIELDS,
    ENV_FIELDS,
    BuildRecipe,
    MAX_RENDERED_FILES,
    MAX_RENDERED_TREE_BYTES,
    TOOLCHAIN_FIELDS,
    TREE_FIELDS,
    reject,
)
from vulkan_docs_identity_parse import bounded_int, canonical, digest, ordered, self_digest

COMMIT = DOCS_COMMIT
IMAGE = "khronosgroup/docker-images@sha256:f1ca671f3bdb10ad49e238b9bf28853088a21af49504498fc9084c9b4fea4762"
GENERATION_ID = "official-vulkan-docs-core-html-1-4-362"
ROOT = {
    "kind": "raw-source-input", "id": "vulkan-14-spec", "source_family": "vulkan-docs",
    "immutable_url": f"https://raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/{COMMIT}/vkspec.adoc",
    "revision": COMMIT, "sha256": "069b7e6d6326969df7b4a86f189f7ba22359e93ca7aa76c23301504667d3c4b0",
    "bytes": 8685, "license": "CC-BY-4.0",
    "local_cache": f"{NAMESPACE}/raw/vulkan-14-spec/069b7e6d6326969df7b4a86f189f7ba22359e93ca7aa76c23301504667d3c4b0.source",
    "generated_code_role": "Docs source", "provenance": "official Vulkan-Docs v1.4.362 source root",
    "selector": "vkspec.adoc",
}
ENVIRONMENT = (("LC_ALL", "C"), ("MAKEFLAGS", ""), ("PYTHONDONTWRITEBYTECODE", "1"), ("TZ", "UTC"))
ARGV = (
    "./makeSpec", "-clean", "-spec", "core", "-version", "1.4", "-genpath", "/work/generated",
    "SPECREVISION=1.4.362", "SPECDATE=2026-09-10 00:00:00Z",
    f"SPECREMARK=from pinned commit {COMMIT}", "VULKAN_API=vulkan", "EXTENSIONS=", "DIFFEXTENSIONS=",
    "EXTRAATTRIBS=", "IMAGEOPTS=inline", "NOTEOPTS=-a editing-notes -a implementation-guide", "html",
)
OUTPUT = {
    "kind": "rendered-output", "id": "vkspec-html", "producer_generation_id": GENERATION_ID,
    "selector": "out/html/vkspec.html",
    "sha256": "896452d3a3e4887ba1bad81a6b9b91e9b3b324e1fffbb9e1dec889d0b06ac041",
    "bytes": 10377052, "license": "LicenseRef-KhronosSpecCopyright",
    "local_cache": f"{NAMESPACE}/rendered/vkspec-html/896452d3a3e4887ba1bad81a6b9b91e9b3b324e1fffbb9e1dec889d0b06ac041.artifact",
    "artifact_kind": "rendered-html", "provenance": "official-pinned-container-build",
}
TREE = (2530, 17019466, "26e8e484d34222d9ba8a72c883ff8bb65a4e5194f399c24aa9bd5d7e47c49f32")


def configuration(value: dict[str, object]) -> str:
    payload = dict(value)
    payload.pop("build_sha256", None)
    return canonical(payload, "webboxvm-graphics-vulkan-docs-build-configuration-v1")


def build(value: object) -> BuildRecipe:
    if not isinstance(value, dict) or set(value) != BUILD_FIELDS:
        reject("Docs build has an invalid schema")
    if tuple(value.get(name) for name in (
            "image", "platform", "network", "user", "source_mount", "work_mount", "generated_mount", "workdir", "home", "path", "safe_directory")) != (
            IMAGE, "linux/amd64", "none", "501:20", "/vulkan:ro", "/work:rw", "/work/generated", "/vulkan", "/tmp",
            "/opt/venv/bin:/usr/local/bundle/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
            "not-required-source-owned-by-user"):
        reject("Docs build does not retain the pinned isolated container context")
    environment = value.get("environment")
    if not isinstance(environment, list) or any(not isinstance(item, dict) or set(item) != ENV_FIELDS for item in environment):
        reject("Docs build environment has an invalid schema")
    if tuple((item.get("name"), item.get("value")) for item in environment) != ENVIRONMENT:
        reject("Docs build environment is not the reviewed deterministic environment")
    if tuple(value.get("argv", ())) != ARGV:
        reject("Docs build argv is not the pinned core HTML recipe")
    toolchain = value.get("toolchain")
    if not isinstance(toolchain, dict) or set(toolchain) != TOOLCHAIN_FIELDS:
        reject("Docs build toolchain has an invalid schema")
    if tuple(toolchain.get(name) for name in ("python", "python_version", "pyparsing_version")) != (
            "/opt/venv/bin/python3", "3.13.5", "3.3.1"):
        reject("Docs build toolchain is not the pinned image venv")
    return BuildRecipe(
        self_digest(value, "build_sha256", "Docs build"), configuration(value), GENERATION_ID)


def root(value: object):
    result = raw(value)
    if result != raw(ROOT):
        reject("Docs root does not retain the reviewed pinned identity")
    return result


def outputs(value: object, recipe: BuildRecipe):
    if not isinstance(value, list) or len(value) != 1:
        reject("Docs witness must name exactly one observed primary output")
    rows = rendered_outputs(value)
    if rows != rendered_outputs([OUTPUT]) or rows[0].producer != recipe.generation_id:
        reject("Docs witness output does not retain the observed HTML identity")
    return rows


def output_tree(value: object, outputs) -> str:
    if not isinstance(value, dict) or set(value) != TREE_FIELDS:
        reject("Docs output tree has an invalid schema")
    if value.get("algorithm") != "canonical-json-ordered-selector-sha256-bytes-v1":
        reject("Docs output tree has an unknown identity algorithm")
    if (bounded_int(value.get("file_count"), "Docs output tree file count", MAX_RENDERED_FILES),
            bounded_int(value.get("bytes"), "Docs output tree bytes", MAX_RENDERED_TREE_BYTES),
            digest(value.get("sha256"), "Docs output tree sha256")) != TREE:
        reject("Docs output tree does not retain the observed complete-tree identity")
    if value.get("primary_output_id") != outputs[0].identifier:
        reject("Docs output tree does not bind its primary output")
    runs = tuple(digest(item, "Docs output clean run") for item in ordered(
        value.get("clean_run_tree_sha256s"), "Docs output clean runs"))
    if runs != (TREE[2], TREE[2]):
        reject("Docs output tree does not bind two equal clean runs")
    return TREE[2]
