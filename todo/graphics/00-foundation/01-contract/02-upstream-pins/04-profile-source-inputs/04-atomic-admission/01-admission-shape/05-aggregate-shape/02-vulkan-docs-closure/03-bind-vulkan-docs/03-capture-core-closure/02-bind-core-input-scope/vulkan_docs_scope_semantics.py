"""Turn validated observed records into core-only semantic identities."""

from __future__ import annotations

import re
import sys
from pathlib import Path

from vulkan_docs_scope_model import (
    CONFIG_INPUTS, DERIVED, EXTENSION_CONTROLS, IMAGE_COUNT, IMAGE_PREFIX, INACTIVE_EXTENSION_SELECTORS,
    INDIVIDUAL_PREFIXES, PROMOTIONS, RAW, ROOT_SELECTOR, VIDEO, WSI, reject,
)
from vulkan_docs_scope_parse import canonical

HERE = Path(__file__).resolve().parent
IDENTITY = HERE.parent.parent / "02-actual-closure-identity"
if str(IDENTITY) not in sys.path:
    sys.path.insert(0, str(IDENTITY))

from vulkan_docs_identity_build import GENERATION_ID, ROOT  # noqa: E402
from vulkan_docs_identity_members import DOCS_COMMIT  # noqa: E402

GENERATED_EXTENSION = re.compile(r"/(?:vk|Vk).*?[A-Z]{2,}\.adoc$")


def raw_rows(records: tuple[dict[str, object], ...]) -> list[dict[str, object]]:
    return [{"kind": RAW, "selector": row["selector"], "revision": DOCS_COMMIT,
             "immutable_url": f"https://raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/{DOCS_COMMIT}/{row['selector']}",
             "sha256": row["sha256"], "bytes": row["bytes"], "phase_roles": row["phase_roles"]}
            for row in records if row["kind"] == RAW]


def derived_rows(records: tuple[dict[str, object], ...]) -> list[dict[str, object]]:
    return [{"kind": DERIVED, "selector": row["selector"], "generation_id": GENERATION_ID,
             "sha256": row["sha256"], "bytes": row["bytes"], "phase_roles": row["phase_roles"]}
            for row in records if row["kind"] == DERIVED]


def selected(index, includes, pairs, label: str) -> dict[str, object]:
    rows = []
    for pair in pairs:
        row = index.get(pair)
        if row is None or row["phase_roles"] != ["asciidoctor"]:
            reject(f"core scope lacks the required {label} input")
        rows.append(row)
    names = set(pairs)
    hits = [row for row in includes if (row["kind"], row["selector"]) in names]
    if { (row["kind"], row["selector"]) for row in hits } != names:
        reject(f"core scope lacks resolved evidence for its {label} inputs")
    return {"selectors": [selector for _, selector in pairs], "identity_sha256": canonical(
        {"records": rows, "includes": hits}, f"webboxvm-graphics-vulkan-docs-{label}-v1")}


def absent(records, includes, selector: str) -> dict[str, int]:
    count = sum(row["selector"] == selector for row in records)
    include_count = sum(row["selector"] == selector for row in includes)
    if count or include_count:
        reject("core scope resolves a forbidden WSI or video branch")
    return {"records": count, "includes": include_count}


def individual_extension(row, inactive: set[str]) -> bool:
    name = row["selector"]
    return (name.startswith(INDIVIDUAL_PREFIXES) or name in inactive or (row["kind"] == DERIVED and
            (name.startswith("generated/meta/VK_") or
             (name.startswith("generated/interfaces/VK_") and not name.startswith("generated/interfaces/VK_VERSION_")) or
             GENERATED_EXTENSION.search(name))))


def conditions(records: tuple[dict[str, object], ...], includes: tuple[dict[str, object], ...]) -> dict[str, object]:
    index = {(row["kind"], row["selector"]): row for row in records}
    root = index.get((RAW, ROOT_SELECTOR))
    if root is None or root["phase_roles"] != ["asciidoctor"] or (root["sha256"], root["bytes"]) != (ROOT["sha256"], ROOT["bytes"]):
        reject("core scope does not retain the reviewed pinned root")
    controls = selected(index, includes, EXTENSION_CONTROLS, "extension-controls")
    promotions = selected(index, includes, PROMOTIONS, "promotions")
    config = selected(index, includes, tuple((DERIVED, item) for item in CONFIG_INPUTS), "configuration-inputs")
    inactive = set(INACTIVE_EXTENSION_SELECTORS)
    individual_records = [row for row in records if individual_extension(row, inactive)]
    individual_includes = [row for row in includes if individual_extension(row, inactive)]
    if individual_records or individual_includes:
        reject("core scope resolves individual extension semantics")
    image_rows = [row for row in records if row["kind"] == RAW and row["selector"].startswith(IMAGE_PREFIX)]
    if len(image_rows) != IMAGE_COUNT or any(not row["selector"].endswith(".svg") or row["phase_roles"] != ["asciidoctor"] for row in image_rows):
        reject("core scope does not retain the observed SVG image inputs")
    value = {
        "wsi": absent(records, includes, WSI), "video": absent(records, includes, VIDEO),
        "extension_controls": controls, "configuration_inputs": config,
        "individual_extension_branches": {"prefixes": list(INDIVIDUAL_PREFIXES),
                                            "selectors": list(INACTIVE_EXTENSION_SELECTORS), "records": 0, "includes": 0},
        "images": {"count": len(image_rows), "bytes": sum(row["bytes"] for row in image_rows),
                   "identity_sha256": canonical(image_rows, "webboxvm-graphics-vulkan-docs-core-images-v1")},
        "promotions": promotions,
    }
    value["identity_sha256"] = canonical(value, "webboxvm-graphics-vulkan-docs-core-conditions-v1")
    return value
