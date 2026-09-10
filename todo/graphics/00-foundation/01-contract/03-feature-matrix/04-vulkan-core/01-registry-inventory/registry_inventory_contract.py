#!/usr/bin/env python3
"""Extract raw Vulkan registry structure without evaluating its semantics."""

from __future__ import annotations

import hashlib
import json
import re
from xml.etree import ElementTree as XML

from registry_inventory_source import reject

MAX_SERIALIZED, MAX_ROW_BYTES = 8 * 1024 * 1024, 8 * 1024 * 1024 - 64 * 1024
NAME = re.compile(r"^[A-Za-z][A-Za-z0-9_]*$")
LOCATOR = re.compile(
    r"^xml/vk\.xml#feature\[@name='VK_(?:(?:BASE|COMPUTE|GRAPHICS)_)?VERSION_1_[0-4]'\]"
    r"(?:/(?:require|deprecate)\[[1-9][0-9]*\]/(?:command|enum|feature|type)\[[1-9][0-9]*\])?$")
KINDS = frozenset(("command", "enum", "feature", "type", "version-marker"))
CONTAINERS = frozenset(("require", "deprecate"))
BOUNDARIES = ("extensions-excluded", "wsi-excluded", "external-memory-excluded",
              "spirv-grammar-separate", "bring-up-unimplemented")
EFFECTS = ("project_complete_docs_role", "project_cts_selector", "project_release_claim",
           "supported", "emulated", "conformant", "certified", "near_native")
ROW_FIELDS = frozenset(("requirement_kind", "container_kind", "name", "source_locator",
                        "source_order", "condition", "status", "implementation_owner",
                        "independent_test_plan", "blocker"))
BLOCKER = "source-contract-owner-and-independent-test-plan-unassigned"
FEATURES = tuple(
    (f"VK_{prefix}VERSION_1_{minor}", "internal" if prefix else None)
    for minor in range(5) for prefix in ("BASE_", "COMPUTE_", "GRAPHICS_", ""))


def packed(value: object) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"))


def attrs(element) -> dict[str, str]:
    return {str(key): str(value) for key, value in element.attrib.items()}


def condition(feature, section=None, item=None) -> str:
    value: dict[str, object] = {"feature": attrs(feature)}
    if section is not None:
        value["section"], value["item"] = attrs(section), attrs(item)
    return packed(value)


def core_features(root) -> list[object]:
    expected = [name for name, _ in FEATURES]
    found = [feature for feature in root.findall("feature") if feature.get("name") in expected]
    if [feature.get("name") for feature in found] != expected:
        reject("registry lacks the source-ordered Vulkan 1.0 through 1.4 structural blocks")
    for feature, (name, internal) in zip(found, FEATURES):
        if (feature.get("api", "").split(",").count("vulkan") != 1 or feature.get("number") != f"1.{name[-1]}"
                or feature.get("apitype") != internal):
            reject("registry has an unsafe Vulkan structural-block identity")
    return found


def row(kind: str, container: str, name: str, locator: str, order: int, source_condition: str) -> dict[str, object]:
    if kind not in KINDS or container not in CONTAINERS | {"feature"} or not NAME.fullmatch(name) or not LOCATOR.fullmatch(locator):
        reject("registry has an unsafe technical reference")
    return {"requirement_kind": kind, "container_kind": container, "name": name,
            "source_locator": locator, "source_order": order, "condition": source_condition,
            "status": "blocked", "implementation_owner": None, "independent_test_plan": None,
            "blocker": BLOCKER}


def append(rows: list[dict[str, object]], record: dict[str, object], used: int) -> int:
    used += len(packed(record).encode()) + 1
    if used > MAX_ROW_BYTES:
        reject("technical inventory exceeds the 8 MiB serialized member cap")
    rows.append(record)
    return used


def parsed_root(raw: bytes):
    if len(raw) > MAX_SERIALIZED:
        reject("registry XML exceeds the 8 MiB member cap")
    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError as error:
        reject(f"registry XML is not UTF-8: {error}")
    if "<!DOCTYPE" in text or "<!ENTITY" in text:
        reject("registry XML must not use document-type declarations or entities")
    try:
        root = XML.fromstring(text)
    except XML.ParseError as error:
        reject(f"registry XML is malformed: {error}")
    if root.tag != "registry":
        reject("registry XML has an unexpected root")
    return root


def rows_from_raw(raw: bytes) -> list[dict[str, object]]:
    rows, order, used = [], 0, 0
    root = parsed_root(raw)
    for feature in core_features(root):
        marker, base, sections = str(feature.get("name")), "", {kind: 0 for kind in CONTAINERS}
        base = f"xml/vk.xml#feature[@name='{marker}']"
        order += 1
        used = append(rows, row("version-marker", "feature", marker, base, order, condition(feature)), used)
        for section in feature:
            container = section.tag
            if container == "comment":
                continue
            if container not in CONTAINERS:
                reject("registry has an unsupported structural section")
            sections[container] += 1
            positions: dict[str, int] = {}
            for item in section:
                kind, name = item.tag, item.get("name")
                if kind == "comment":
                    continue
                if kind not in KINDS - {"version-marker"} or name is None:
                    reject("registry has an unsupported structural member")
                positions[kind] = positions.get(kind, 0) + 1
                order += 1
                locator = f"{base}/{container}[{sections[container]}]/{kind}[{positions[kind]}]"
                used = append(rows, row(kind, container, name, locator, order, condition(feature, section, item)), used)
    return rows


def rows_sha256(rows: list[dict[str, object]]) -> str:
    return hashlib.sha256(packed(rows).encode()).hexdigest()


def build_inventory(raw: bytes, identity: dict[str, object]) -> dict[str, object]:
    rows = rows_from_raw(raw)
    value = {"schema": 1, "contract": "vulkan-registry-technical-inventory-v1", "status": "blocked",
             "registry": identity, "scope": "raw-direct-cumulative-vulkan-1.0-through-1.4-structural-blocks",
             "boundaries": list(BOUNDARIES), "rows": rows, "rows_sha256": rows_sha256(rows),
             "effects": {effect: False for effect in EFFECTS}}
    if len(packed(value).encode()) > MAX_SERIALIZED:
        reject("technical inventory exceeds the 8 MiB serialized member cap")
    return value
