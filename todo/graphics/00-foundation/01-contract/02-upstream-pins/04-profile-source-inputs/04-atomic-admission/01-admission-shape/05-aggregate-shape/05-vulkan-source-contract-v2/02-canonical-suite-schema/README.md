# F02.4.4.1.5.5.2 — Define the canonical-suite schema

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.5.2
Depends: F02.2, F02.4.4.1.5.5.1
Evidence: pending

Prerequisite lists: [the V2 decision](../01-policy-decision/README.md) and the
[regular-source policy](../../../../../../02-fetch-verifier/01-fetch-contract/source_model.py).

## Outcome

A separate V2 schema makes the small immutable selector root a regular source input while modelling
its large test-data closure as a canonical upstream suite. It does not alter the V1 manifest schema.

## Starting points

- [VCTS V1 transcript](../../../../../03-vulkan-input-audit/mustpass_references.json)
- [F02 fetch policy](../../../../../../02-fetch-verifier/01-fetch-contract/source_model.py)
- [candidate contract](../../../../../candidate_contract.py)

## Checklist

- [ ] Pin tag object, peeled commit, root URL, root SHA-256, and root byte count.
- [ ] Specify an ordered recursive ledger of safe POSIX paths, blob identities, byte counts, and hashes.
- [ ] Keep source input and suite closure types separate; V1 loaders and receipts stay unchanged.
- [ ] Define explicit member, per-member, and aggregate limits from the audited suite boundary.
- [ ] Add positive and hostile schema tests for substituted roots, paths, duplicates, cycles, and hashes.

## Verification

- The schema rejects a root-only, mutable, reordered, mixed-commit, or locally filtered suite.
- It does not say that `vk-default` is an upstream Vulkan-1.4-core selector.
