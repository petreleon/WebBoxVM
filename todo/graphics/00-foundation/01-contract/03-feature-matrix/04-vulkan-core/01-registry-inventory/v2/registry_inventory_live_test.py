#!/usr/bin/env python3
"""Assert the sealed-cache registry produces the known blocked structural facts."""

from __future__ import annotations

import argparse
from collections import Counter
from pathlib import Path

from registry_inventory_cache import payload_from_selector_cache
from registry_inventory_contract import build_inventory
from registry_inventory_validation import validate_inventory, validate_scaffold


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--selector-cache-root", type=Path, required=True)
    args = parser.parse_args()
    identity = validate_scaffold()
    value = build_inventory(payload_from_selector_cache(args.selector_cache_root, identity), identity)
    validate_inventory(value, identity)
    rows = value["rows"]
    kinds, containers = Counter(row["requirement_kind"] for row in rows), Counter(row["container_kind"] for row in rows)
    require(kinds == {"command": 260, "enum": 393, "feature": 117, "type": 668, "version-marker": 20}, "unexpected structural kind counts")
    require(containers == {"require": 1350, "deprecate": 88, "feature": 20}, "unexpected structural lifecycle counts")
    locators = {row["name"]: row["source_locator"] for row in rows}
    require(locators.get("vkCreateInstance") == "xml/vk.xml#feature[@name='VK_BASE_VERSION_1_0']/require[6]/command[1]", "missing base command locator")
    require(locators.get("vkCmdPushDescriptorSet") == "xml/vk.xml#feature[@name='VK_COMPUTE_VERSION_1_4']/require[6]/command[1]", "missing compute command locator")
    require(sum(row["name"] == "pipelineProtectedAccess" for row in rows) == 2, "distinct public feature references were merged")
    print(f"LIVE: {len(rows)} blocked rows; {value['rows_sha256']}")


if __name__ == "__main__":
    main()
