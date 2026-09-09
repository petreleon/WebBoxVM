# F02.3.3.4.1.1.2 — Adopt the versioned inventory in F02.1 and F02.2

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F02.3.3.4.1.1.2
Depends: F02.1, F02.2, F02.3.3.4.1.1.1
Evidence: pending

Prerequisite lists: [the composite-lock contract](../01-shared-layout-loader/README.md),
[F02.1](../../../../../../01-input-inventory/README.md), and
[F02.2](../../../../../../02-fetch-verifier/README.md).

## Outcome

F02.1 and F02.2 load both the current schema-v1 inventory and hermetic schema-v2 fixtures only
through the shared layout contract. The active 15 entries, external cache behavior, and accepted
sidecars remain unchanged until the final atomic cutover.

## Starting points

- [composite-lock contract](../01-shared-layout-loader/README.md)
- [F02.1 structural validator](../../../../../../01-input-inventory/validate_manifest.py)
- [F02.2 source model](../../../../../../02-fetch-verifier/01-fetch-contract/source_model.py)

## Checklist

- [ ] Make the F02.1 validator and F02.2 source model obtain entries and an inventory revision only
  through the shared layout loader.
- [ ] Preserve current schema-v1 behavior while adding hermetic schema-v2 layout fixtures.
- [ ] Reject a stale lock or component-closure mismatch before the fetch model returns an input or
  creates an external-cache action.
- [ ] Keep network fetches, cache payloads, and accepted source identities unchanged in this
  preparation child; run its focused nonzero suites.

## Verification

- A stale lock or component closure is rejected before F02.2 can accept an input.
- The currently checked schema-v1 inventory retains its historical behavior until the cutover.
