#!/usr/bin/env python3
"""Build a bounded WebBoxVM Vulkan 1.4 XML-facts artifact without network access."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "01-normative-root-pins"))
from normative_roots import EXPECTED, NO_CLAIMS  # noqa: E402

MAX_LOCAL_BYTES = 8 * 1024 * 1024
REQUIRED = ("vulkan-14-spec", "vulkan-registry")


class BuildError(ValueError):
    """Pinned source data cannot make the bounded local definition."""


def payload(identifier: str, path: Path) -> tuple[bytes, dict[str, object]]:
    record = EXPECTED.get(identifier)
    if record is None or path.is_symlink() or not path.is_file():
        raise BuildError(f"{identifier} source is not a regular pinned file")
    data = path.read_bytes()
    if (len(data), hashlib.sha256(data).hexdigest()) != (record["bytes"], record["sha256"]):
        raise BuildError(f"{identifier} source bytes do not match its pinned record")
    return data, {key: record[key] for key in ("id", "revision", "sha256", "bytes", "license", "attribution",
                                                "scope", "immutable_url")}


def attributes(node: ET.Element) -> dict[str, str]:
    return {name: node.attrib[name] for name in sorted(node.attrib)}


def core_feature(vkxml: bytes) -> tuple[dict[str, object], list[str]]:
    try:
        root = ET.fromstring(vkxml)
    except ET.ParseError as error:
        raise BuildError(f"vk.xml cannot be parsed: {error}") from error
    matches = [item for item in root.findall("feature") if item.get("name") == "VK_VERSION_1_4"]
    if len(matches) != 1:
        raise BuildError("vk.xml must contain exactly one VK_VERSION_1_4 feature")
    feature = matches[0]
    if feature.get("number") != "1.4" or "vulkan" not in feature.get("api", "").split(","):
        raise BuildError("VK_VERSION_1_4 is not a Vulkan 1.4 feature")
    requirements = []
    for require in feature.findall("require"):
        items = []
        for item in require:
            if item.tag not in {"command", "enum", "feature", "type"}:
                raise BuildError("VK_VERSION_1_4 has an unknown XML requirement kind")
            items.append({"kind": item.tag, "attributes": attributes(item)})
        requirements.append({"attributes": attributes(require), "items": items})
    extensions = root.findall("./extensions/extension")
    promoted = sorted(item.attrib["name"] for item in extensions
                      if "VK_VERSION_1_4" in item.get("promotedto", "").split(",") and "name" in item.attrib)
    return {"attributes": attributes(feature), "requirements": requirements}, promoted


def definition(vkspec: Path, vkxml: Path) -> dict[str, object]:
    prose, prose_record = payload("vulkan-14-spec", vkspec)
    registry, registry_record = payload("vulkan-registry", vkxml)
    del prose
    feature, promoted = core_feature(registry)
    return {
        "schema": 1,
        "kind": "webboxvm-vulkan-14-core-definition",
        "scope": "webboxvm-core-definition",
        "authority": "WebBoxVM",
        "producer": "WebBoxVM",
        "claims": dict(NO_CLAIMS),
        "license": "Apache-2.0 OR MIT for extracted vk.xml facts; no Vulkan prose reproduced",
        "attribution": "Khronos Group Vulkan-Docs; WebBoxVM-derived bounded XML facts",
        "sources": {"normative_prose_root": prose_record, "registry_metadata": registry_record},
        "source_locators": {
            "normative_prose_root": {"id": "vulkan-14-spec", "path": "vkspec.adoc", "usage": "identity-verified root locator only"},
            "registry_metadata": {"id": "vulkan-registry", "path": "xml/vk.xml", "usage": "VK_VERSION_1_4 XML facts only"},
        },
        "xml_feature": feature,
        "boundaries": {
            "prose": "vkspec.adoc is an identity-verified root locator only; includes and prose are not reproduced",
            "extensions_promoted_to_vulkan_1_4": promoted,
            "extensions_wsi_video": "not extracted as core facts or implementation support",
            "qualification": "not a Khronos selector, complete CTS root, compatibility, or performance result",
        },
    }


def encode(vkspec: Path, vkxml: Path) -> bytes:
    return json.dumps(definition(vkspec, vkxml), ensure_ascii=False, sort_keys=True,
                      separators=(",", ":"), allow_nan=False).encode("utf-8") + b"\n"


def write_output(path: Path, data: bytes) -> None:
    if len(data) > MAX_LOCAL_BYTES:
        raise BuildError("WebBoxVM core definition exceeds 8 MiB")
    if path.is_symlink():
        raise BuildError("output must not be a symlink")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mode", required=True)
    parser.add_argument("--vkspec", type=Path, required=True)
    parser.add_argument("--vkxml", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        if args.mode != "core-definition":
            raise BuildError("builder mode must be core-definition")
        data = encode(args.vkspec, args.vkxml)
        write_output(args.output, data)
        print(f"PASS: {args.output} bytes={len(data)} sha256={hashlib.sha256(data).hexdigest()}")
    except BuildError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
