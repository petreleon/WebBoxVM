"""Parse distinct F02 source inputs and locally rendered Docs artifacts."""

from __future__ import annotations

from urllib.parse import urlsplit

from vulkan_docs_identity_model import (
    DERIVED,
    DERIVED_FIELDS,
    MAX_RENDERED_BYTES,
    RAW,
    RAW_FIELDS,
    RENDERED,
    RENDERED_FIELDS,
    DerivedInput,
    RawInput,
    RenderedOutput,
    reject,
)
from vulkan_docs_identity_parse import F02, bounded_int, digest, identifier, selector, strings, text

NAMESPACE = "webboxvm-graphics/successor/vulkan-docs-v1"
DOCS_COMMIT = "f84d432d5b8912362f96f581f29bbc4f3c8c7843"


def cache(value: object, kind: str, member_id: str, member_digest: str, suffix: str) -> str:
    value = selector(value, f"{kind} cache")
    expected = f"{NAMESPACE}/{kind}/{member_id}/{member_digest}.{suffix}"
    if value != expected:
        reject(f"{kind} cache escapes the actual-Docs namespace")
    return value


def raw(value: object) -> RawInput:
    if not isinstance(value, dict) or set(value) != RAW_FIELDS or value.get("kind") != RAW:
        reject("raw Docs input has an invalid schema")
    member_id = identifier(value.get("id"), "raw Docs input")
    source_family = text(value.get("source_family"), "raw Docs source_family")
    if source_family != "vulkan-docs":
        reject("raw Docs input has the wrong source family")
    revision = text(value.get("revision"), "raw Docs revision")
    if revision != DOCS_COMMIT:
        reject("raw Docs input revision is not the pinned Docs commit")
    member_digest = digest(value.get("sha256"), "raw Docs sha256")
    if not F02.COMMIT.fullmatch(revision):
        reject("raw Docs input revision is not a commit")
    url = text(value.get("immutable_url"), "raw Docs URL")
    try:
        F02.immutable_url(url, revision, member_id)
    except F02.ContractError as error:
        reject(f"raw Docs input violates F02.2 URL policy: {error}")
    item_selector = selector(value.get("selector"), "raw Docs selector")
    parsed = urlsplit(url)
    url_parts = tuple(parsed.path.split("/")[1:])
    if (parsed.hostname != "raw.githubusercontent.com" or url_parts[:3] != (
            "KhronosGroup", "Vulkan-Docs", revision) or tuple(item_selector.split("/")) != url_parts[3:]):
        reject("raw Docs selector does not name its immutable URL")
    size = bounded_int(value.get("bytes"), "raw Docs input bytes", F02.MAX_INPUT_BYTES)
    license = text(value.get("license"), "raw Docs license")
    generated_code_role = text(value.get("generated_code_role"), "raw Docs generated_code_role")
    provenance = text(value.get("provenance"), "raw Docs provenance")
    return RawInput(
        member_id, source_family, url, revision, member_digest, size, license, item_selector,
        cache(value.get("local_cache"), "raw", member_id, member_digest, "source"),
        generated_code_role, provenance)


def derived(value: object) -> DerivedInput:
    if not isinstance(value, dict) or set(value) != DERIVED_FIELDS or value.get("kind") != DERIVED:
        reject("derived Docs input has an invalid schema")
    member_id = identifier(value.get("id"), "derived Docs input")
    member_digest = digest(value.get("sha256"), "derived Docs sha256")
    size = bounded_int(value.get("bytes"), "derived Docs input bytes", F02.MAX_INPUT_BYTES)
    license = text(value.get("license"), "derived Docs license")
    generated_code_role = text(value.get("generated_code_role"), "derived Docs generated_code_role")
    provenance = text(value.get("provenance"), "derived Docs provenance")
    item_selector = selector(value.get("selector"), "derived Docs selector")
    producers = tuple(identifier(item, "derived Docs producer") for item in strings(
        value.get("producer_input_ids"), "derived Docs producers"))
    generation_id = identifier(value.get("generation_id"), "derived Docs generation")
    return DerivedInput(
        member_id, member_digest, size, item_selector,
        cache(value.get("local_cache"), "derived", member_id, member_digest, "derived"),
        producers, generation_id, license, generated_code_role, provenance)


def rendered(value: object) -> RenderedOutput:
    if not isinstance(value, dict) or set(value) != RENDERED_FIELDS or value.get("kind") != RENDERED:
        reject("rendered Docs output has an invalid schema")
    output_id = identifier(value.get("id"), "rendered Docs output")
    output_digest = digest(value.get("sha256"), "rendered Docs sha256")
    output_bytes = bounded_int(value.get("bytes"), "rendered Docs output bytes", MAX_RENDERED_BYTES)
    item_selector = selector(value.get("selector"), "rendered Docs selector")
    artifact_kind = value.get("artifact_kind")
    if artifact_kind != "rendered-html":
        reject("rendered Docs artifact kind is not rendered-html")
    license = text(value.get("license"), "rendered Docs license")
    provenance = text(value.get("provenance"), "rendered Docs provenance")
    producer = identifier(value.get("producer_generation_id"), "rendered Docs producer")
    return RenderedOutput(
        output_id, output_digest, output_bytes, item_selector,
        cache(value.get("local_cache"), "rendered", output_id, output_digest, "artifact"),
        producer, license, artifact_kind, provenance)


def rendered_outputs(value: object) -> tuple[RenderedOutput, ...]:
    if not isinstance(value, list) or not value:
        reject("rendered Docs outputs must be a nonempty ordered list")
    rows = tuple(rendered(item) for item in value)
    for attribute in ("identifier", "selector", "cache_path"):
        values = tuple(getattr(item, attribute) for item in rows)
        if len(set(values)) != len(values):
            reject(f"rendered Docs outputs repeat an output {attribute}")
    return rows
