# F02.4.4.1.5.4.4.6 — Define the successor inventory transition

[Parent task](../README.md)

Task: F02.4.4.1.5.4.4.6
Depends: F02.4.4.1.5.4.4.1
Evidence: pending

Prerequisite lists: the [derived-Docs policy](../01-derived-docs-policy/README.md), the active
[F02 inventory schema](../../../../../../../01-input-inventory/README.md), and the
[F02 fetch contract](../../../../../../../02-fetch-verifier/README.md).

## Outcome

One future atomic inventory-schema successor may represent a complete `vulkan-docs` source family
and its bounded member closure without mutating the active 17-family inventory or relaxing F02.2.
It remains a design boundary until the exact schema, cache, and closure proof pass together.

## Starting points

- [active inventory layout](../../../../../../../01-input-inventory/inventory_layout.py)
- [F02 source model](../../../../../../../02-fetch-verifier/01-fetch-contract/source_model.py)
- [Docs-derived policy](../01-derived-docs-policy/README.md)

## Checklist

- [ ] Define a versioned inventory schema/family/cardinality transition for cap-valid Vulkan-Docs members.
- [ ] Bind immutable URLs, revisions, digests, byte counts, license expressions, roles, and safe external-cache names.
- [ ] Preserve all active inventory identities and reject mixed old/new schema, aliases, duplicate families, or root-only imports.
- [ ] Require the complete successor Docs closure before a proposed inventory record can become admitted.
- [ ] Keep active F02.1/F02.2, cache freshness, F03, and all implementation/release states unchanged.
- [ ] Add focused positive and hostile transition tests with an unadmitted policy receipt.

## Verification

- A new source-family design is not an active inventory mutation and cannot bypass F02.2 validation.
- No later aggregate may claim admission until this transition and every independent closure are valid.
