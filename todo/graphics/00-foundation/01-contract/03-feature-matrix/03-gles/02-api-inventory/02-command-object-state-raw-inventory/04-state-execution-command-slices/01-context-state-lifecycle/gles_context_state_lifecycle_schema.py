"""Compact lifecycle row schema and explicit open coverage ledger."""

import re

ENTRY_FIELDS = ("raw_id", "family_id", "source_family_order", "triggering_commands", "c_names",
                "rule_text_parts", "physical_page", "section", "source_order")
DEFAULT_FIELDS = frozenset(("derivation_class", "source_locator_format"))
DEFAULTS = {"derivation_class": "explicit-triggered-rule",
            "source_locator_format": "gles32-pdf-v1:page={physical_page};section={section}"}
FAMILIES = (
    ("fundamentals", 6, ("2.1",)),
    ("context-and-lifecycle", 9, ("2.4-2.5", "5")),
    ("object-taxonomy", 10, ("2.6",)),
    ("vertex-remaining-state", 21, ("10.1", "10.6-10.7")),
    ("programmable-vertex-stage", 25, ("11",)),
    ("post-vertex-state", 27, ("12.1", "12.3-12.6")),
    ("programmable-fragment-stage", 29, ("14",)),
    ("state-tables", 34, ("21.1-21.39",)),
)
BINDING = [{"id": identifier, "source_order": order} for identifier, order, _ in FAMILIES]


def raw_defaults(binding):
    if binding != BINDING:
        raise ValueError("lifecycle raw defaults require all eight exact assigned families")
    return dict(DEFAULTS)


def coverage(binding):
    raw_defaults(binding)
    routes = []
    for identifier, _, scope in FAMILIES:
        if identifier == "fundamentals":
            routes.append([identifier, "no-eligible-rule", list(scope), [], "no-literal-trigger"])
        elif identifier in ("context-and-lifecycle", "object-taxonomy"):
            routes.append([identifier, "covered-trigger-class", list(scope), [], "quoted-literal-trigger-only"])
        else:
            routes.append([identifier, "pending", [], list(scope), "state-command-and-table-extraction-pending"])
    return {"complete": False, "contract": "object-model-and-shared-object-literal-triggers-v1",
            "route_fields": ["family_id", "status", "reviewed_scope", "pending_scope", "reason"],
            "routes": routes, "raw_positive_rules": 12, "pending_family_count": 5}


def records(value):
    if value.get("raw_entry_fields") != list(ENTRY_FIELDS) or value.get("raw_entry_defaults") != DEFAULTS:
        raise ValueError("lifecycle row schema/defaults are incomplete or promoted")
    rows, result = value.get("raw_entries"), []
    if not isinstance(rows, list) or len(rows) != 12:
        raise ValueError("lifecycle rows omit mandatory initial-group rules")
    orders = {identifier: order for identifier, order, _ in FAMILIES}
    for number, row in enumerate(rows, 1):
        if not isinstance(row, list) or len(row) != len(ENTRY_FIELDS):
            raise ValueError("lifecycle compact row is malformed")
        item = dict(zip(ENTRY_FIELDS, row))
        names, parts = item["triggering_commands"], item["rule_text_parts"]
        if (item["raw_id"] != f"gles32-context-state-lifecycle-{number:02d}"
                or item["family_id"] not in ("object-taxonomy", "context-and-lifecycle")
                or item["source_family_order"] != orders[item["family_id"]]
                or type(item["source_order"]) is not int or item["source_order"] != number
                or type(item["physical_page"]) is not int or item["physical_page"] not in (43, 44, 63, 65, 66)
                or not isinstance(item["section"], str) or not re.fullmatch(r"[25](?:\.[0-9]+)+", item["section"])
                or not isinstance(names, list) or not names or len(names) != len(set(names))
                or any(not isinstance(name, str) or not re.fullmatch(r"[A-Z][A-Za-z0-9]*", name) for name in names)
                or item["c_names"] != ["gl" + name for name in names]
                or not isinstance(parts, list) or not parts
                or any(not isinstance(part, str) or not part or part != part.strip() or len(part) > 96 for part in parts)):
            raise ValueError("lifecycle anchor, trigger, quote, domain identity, or order is malformed")
        text = " ".join(parts)
        if any(not re.search(r"\b" + name + r"\b", text.replace("- ", "")) for name in names):
            raise ValueError("lifecycle command is not explicitly witnessed by the source quotation")
        item.update(DEFAULTS)
        item["rule_text"] = text
        item["source_locator"] = DEFAULTS["source_locator_format"].format(**item)
        result.append(item)
    return result
