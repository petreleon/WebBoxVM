"""Parse a future Docs closure grammar; later capture binds all conditionals."""

from __future__ import annotations

from vulkan_docs_identity_members import derived, raw
from vulkan_docs_identity_model import BuildRecipe, DERIVED, SCOPE_FIELDS, DerivedInput, RawInput, reject
from vulkan_docs_identity_parse import canonical, digest, identifier, strings

INCLUDE_LIST = "b35c3b8907a3338e688f31bfffd98c174858d31b95d88607046a07314f6e964b"
EXCLUSIONS = {
    "wsi": ["include::{chapters}/VK_KHR_surface/wsi.adoc[]"],
    "video": ["include::{chapters}/videocoding.adoc[]"],
    "extensions": ["include::{chapters}/extensions.adoc[]"],
}
EXCLUDED_SELECTORS = frozenset((
    "chapters/VK_KHR_surface/wsi.adoc", "chapters/videocoding.adoc", "chapters/extensions.adoc",
))


def inputs(value: object):
    if not isinstance(value, list) or len(value) < 3:
        reject("Docs closure needs root, raw, and generated source inputs")
    parsed = tuple(raw(item) if isinstance(item, dict) and item.get("kind") != DERIVED else derived(item) for item in value)
    first = parsed[0]
    if not isinstance(first, RawInput) or first.identifier != "vulkan-14-spec":
        reject("Docs closure must begin with its raw pinned root")
    for attribute in ("identifier", "selector", "cache_path"):
        values = tuple(getattr(item, attribute) for item in parsed)
        if len(set(values)) != len(values):
            reject(f"Docs closure repeats a source-input {attribute}")
    seen: set[str] = set()
    for item in parsed:
        if isinstance(item, DerivedInput) and any(source not in seen for source in item.producers):
            reject("Docs derived input producers are not earlier closure inputs")
        seen.add(item.identifier)
    if sum(isinstance(item, RawInput) for item in parsed) < 2 or not any(
            isinstance(item, DerivedInput) for item in parsed):
        reject("Docs closure is structurally partial")
    return parsed


def member_identity(members) -> str:
    rows = [item.__dict__ for item in members]
    return canonical(rows, "webboxvm-graphics-vulkan-docs-member-identity-v1")


def scope(value: object, members, recipe: BuildRecipe) -> None:
    if not isinstance(value, dict) or set(value) != SCOPE_FIELDS:
        reject("Docs scope has an invalid schema")
    if value.get("scope_kind") != "vulkan-docs-core" or value.get("evidence_mode") != "captured-exact":
        reject("Docs scope is not an exact captured Vulkan core scope")
    ordered = tuple(identifier(item, "Docs scope input") for item in strings(value.get("ordered_input_ids"), "Docs scope inputs"))
    if ordered != tuple(item.identifier for item in members):
        reject("Docs scope input order does not match the closure")
    derived_ids = tuple(identifier(item, "Docs scope derived input") for item in strings(
        value.get("derived_input_ids"), "Docs scope derived inputs"))
    if derived_ids != tuple(item.identifier for item in members if isinstance(item, DerivedInput)):
        reject("Docs scope derived inputs do not match the closure")
    excluded = value.get("excluded_members")
    if excluded != EXCLUSIONS:
        reject("Docs scope does not retain the reviewed WSI/video/extension boundary")
    if any(item.selector in EXCLUDED_SELECTORS for item in members):
        reject("Docs scope resolves a reviewed WSI/video/extension exclusion")
    if digest(value.get("predecessor_include_list_sha256"), "Docs scope include transcript") != INCLUDE_LIST:
        reject("Docs scope does not retain the reviewed direct-include transcript")
    if digest(value.get("configuration_sha256"), "Docs scope configuration") != recipe.configuration_digest:
        reject("Docs scope does not bind the pinned build configuration")
    if digest(value.get("member_identity_sha256"), "Docs scope member identity") != member_identity(members):
        reject("Docs scope does not bind its full ordered member identity")
    if any(isinstance(item, DerivedInput) and item.generation_id != recipe.generation_id for item in members):
        reject("Docs generated inputs do not bind the pinned build recipe")
    actual = digest(value.get("scope_sha256"), "Docs scope sha256")
    expected = dict(value)
    expected.pop("scope_sha256", None)
    if actual != canonical(expected, "webboxvm-graphics-vulkan-docs-scope-v1"):
        reject("Docs scope has a stale scope_sha256")
