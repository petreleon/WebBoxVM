"""Reusable provenance and raw-row validation; catalogs retain section semantics."""

from __future__ import annotations

import argparse
import copy
import hashlib
import sys
from pathlib import Path

from .json_output import pretty_json
from .source import InventoryError, SourceContext, reject

ENTRY_FIELDS = ("raw_id", "unprefixed_name", "c_name", "declaration", "physical_page", "section", "source_order")
DEFAULT_FIELDS = frozenset(("family_id", "source_family_order", "derivation_class", "source_locator_format"))


class InventoryEngine(SourceContext):
    def __init__(self, here: Path, catalog_filename: str, task_key: str, kind: str,
                 family_id: str, scope_fence: str, derivation_class: str = "literal",
                 inventory_filename: str | None = None):
        super().__init__(here, catalog_filename, task_key)
        self.kind, self.family_id, self.scope_fence = kind, family_id, scope_fence
        self.derivation_class = derivation_class
        self.INVENTORY = here / (inventory_filename or catalog_filename.replace("_catalog.py", "_raw_inventory.json"))

    def records(self, value: dict[str, object]) -> list[dict[str, object]]:
        if value.get("raw_entry_fields") != list(ENTRY_FIELDS) or set(value.get("raw_entry_defaults", {})) != DEFAULT_FIELDS:
            reject("raw inventory compact-row schema is incomplete or promoted")
        defaults, rows, names, result = value["raw_entry_defaults"], value.get("raw_entries"), set(), []
        if not isinstance(rows, list):
            reject("raw inventory entries are not a list")
        for row in rows:
            if not isinstance(row, list) or len(row) != len(ENTRY_FIELDS):
                reject("raw inventory compact-row shape is malformed")
            item = dict(zip(ENTRY_FIELDS, row))
            name = item["unprefixed_name"]
            if not isinstance(name, str) or name in names:
                reject("raw inventory name is duplicate or malformed")
            names.add(name)
            item.update(defaults)
            item["source_locator"] = defaults["source_locator_format"].format(**item)
            result.append(item)
        if [item["source_order"] for item in result] != list(range(1, len(result) + 1)):
            reject("raw inventory compact-row order is incomplete or unstable")
        return result

    def rendered(self, cache_root: Path) -> dict[str, object]:
        source, authority, decision, manifest, domain, grammar, ledger, family_order, raw = self.source_input(cache_root)
        try:
            if self.derivation_class == "template-expansion":
                facts = self.CATALOG.facts(raw, self.CATALOG.PAGES, family_order, grammar, self.GRAMMAR.normalize)
            else:
                facts = self.CATALOG.facts(raw, self.CATALOG.PAGES, family_order, self.GRAMMAR.normalize)
        except self.CATALOG.CatalogError as error:
            reject(str(error))
        defaults = {"family_id": self.family_id, "source_family_order": family_order,
                    "derivation_class": self.derivation_class,
                    "source_locator_format": "gles32-pdf-v1:page={physical_page};section={section}"}
        body = {
            "schema": 1, "kind": self.kind, "profile": self.CATALOG.PROFILE,
            "source": source, "source_class": "command-state", "source_decision": decision,
            "source_authority_sha256": authority["boundary_sha256"],
            "source_contract_sha256": authority["source_contract_sha256"],
            "inventory_lock_sha256": authority["inventory_lock_sha256"],
            "cache_boundary_sha256": manifest["cache_boundary_sha256"],
            "physical_pdf_pages": self.CATALOG.PAGES,
            "domain_classification_sha256": domain["classification_sha256"],
            "grammar_sha256": grammar["grammar_sha256"],
            "unavailable_ledger_sha256": ledger["ledger_sha256"],
            "domain_families": [{"id": self.family_id, "source_order": family_order}],
            "raw_entry_fields": list(ENTRY_FIELDS), "raw_entry_defaults": defaults,
            "raw_entries": facts, "raw_entry_count": len(facts),
            "raw_entries_sha256": hashlib.sha256(self.canonical(facts)).hexdigest(),
            "raw_only": True, "promotion_allowed": False, "scope_fence": self.scope_fence,
        }
        return {**body, "inventory_sha256": hashlib.sha256(self.canonical(body)).hexdigest()}

    def validate(self, cache_root: Path, inventory_file: Path | None = None) -> dict[str, object]:
        expected = self.rendered(cache_root)
        actual = self.artifact(self.ARTIFACT.document, inventory_file or self.INVENTORY)
        self.artifact(self.ARTIFACT.self_hashed, actual, "inventory_sha256")
        self.artifact(self.ARTIFACT.forbidden, actual)
        if not self.exact(actual, expected):
            reject("inventory is stale, incomplete, rerouted, or promoted")
        self.records(actual)
        return copy.deepcopy(actual)

    def main(self) -> None:
        parser = argparse.ArgumentParser(description=__doc__)
        parser.add_argument("--cache-root", type=Path, required=True)
        parser.add_argument("--inventory", type=Path, default=self.INVENTORY)
        parser.add_argument("--emit-json", action="store_true")
        args = parser.parse_args()
        try:
            value = self.rendered(args.cache_root) if args.emit_json else self.validate(args.cache_root, args.inventory)
            print(pretty_json(value) if args.emit_json else f"PASS: {value['raw_entry_count']} bounded GLES forms; source-only")
        except InventoryError as error:
            print(f"FAIL: {error}", file=sys.stderr)
            raise SystemExit(2)
