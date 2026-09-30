"""Self-hashed source-family row fragments with an exact aggregate index."""

import hashlib
from pathlib import Path

from .json_output import pretty_json
from .source import reject


def sealed(engine, body, field):
    return {**body, field: hashlib.sha256(engine.canonical(body)).hexdigest()}


def documents(engine, value):
    if not getattr(engine.CATALOG, "FRAGMENTED", False):
        return [(engine.INVENTORY.name, value, "inventory_sha256")]
    fields = value["raw_entry_fields"]
    if "family_id" not in fields or "source_family_order" not in fields:
        reject("source-family fragments require per-row domain identity")
    position, order_position = fields.index("family_id"), fields.index("source_family_order")
    references, fragments, reconstructed = [], [], []
    for family in value["domain_families"]:
        identifier = family["id"]
        if not isinstance(identifier, str) or any(char not in "abcdefghijklmnopqrstuvwxyz0123456789-" for char in identifier):
            reject("fragment family name is unsafe")
        rows = [row for row in value["raw_entries"] if row[position] == identifier]
        if any(row[order_position] != family["source_order"] for row in rows):
            reject("fragment rows disagree with the domain family order")
        filename = engine.INVENTORY.stem + "_" + identifier + ".json"
        body = {"schema": 1, "kind": value["kind"] + "-source-family", "domain_family": family,
                "source_sha256": value["source"]["sha256"], "raw_entry_fields": fields,
                "raw_entries": rows, "raw_entry_count": len(rows),
                "raw_entries_sha256": hashlib.sha256(engine.canonical(rows)).hexdigest(),
                "raw_only": True, "promotion_allowed": False}
        fragment = sealed(engine, body, "fragment_sha256")
        references.append({"filename": filename, "family_id": identifier, "raw_entry_count": len(rows),
                           "fragment_sha256": fragment["fragment_sha256"]})
        fragments.append((filename, fragment, "fragment_sha256"))
        reconstructed.extend(rows)
    if not engine.exact(sorted(reconstructed, key=lambda row: row[fields.index("source_order")]), value["raw_entries"]):
        reject("source-family fragments omit, duplicate, or reorder aggregate rows")
    index = {key: item for key, item in value.items() if key not in ("raw_entries", "inventory_sha256")}
    index.update(raw_entry_fragments=references, aggregate_inventory_sha256=value["inventory_sha256"])
    return [(engine.INVENTORY.name, sealed(engine, index, "inventory_sha256"), "inventory_sha256"), *fragments]


def generated(engine, value):
    return [(name, pretty_json(document) + "\n") for name, document, _ in documents(engine, value)]


def validate(engine, expected, inventory_file: Path):
    for position, (name, document, hash_field) in enumerate(documents(engine, expected)):
        path = inventory_file if position == 0 else inventory_file.parent / name
        actual = engine.artifact(engine.ARTIFACT.document, path)
        engine.artifact(engine.ARTIFACT.self_hashed, actual, hash_field)
        engine.artifact(engine.ARTIFACT.forbidden, actual)
        if not engine.exact(actual, document):
            reject("inventory index or source-family fragment is stale, incomplete, rerouted, or promoted")
