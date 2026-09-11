# F02.4.4.1.5.4.4.5.2.2.1 — Freeze the immutable GLES closure contract

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.4.4.5.2.2.1
Depends: F02.2, F02.4.2, F02.4.4.1.5.4.4.5.2.1
Evidence: pending

Prerequisite lists: the [successor integration](../../01-successor-integration/README.md), the
[GLES audit](../../../../../../../../../02-gles-input-audit/README.md), and the
[F02 fetch policy](../../../../../../../../../../02-fetch-verifier/README.md).

## Outcome

A successor-only, hostile-tested contract describes the pinned Khronos GLES root, its four core
members, twelve configurations, optional-extension exclusion, and reproducible producer entrypoints.
It remains unadmitted and cannot modify active schema-v2 or cache state.

## Starting points

- [pinned root audit](../../../../../../../../../02-gles-input-audit/cts_closure.json)
- [pinned configurations](../../../../../../../../../02-gles-input-audit/cts_configurations.json)
- [multi-suite successor integration](../../01-successor-integration/README.md)

## Checklist

- [ ] Bind the immutable root, four ordered core members, and optional-extension boundary to one revision.
- [ ] Require each physical member's URL, digest, bytes, license, provenance, successor cache identity, and 8 MiB cap.
- [ ] Bind the twelve core configurations and explicit upstream producer entrypoints without inventing output attestation.
- [ ] Keep the contract successor-only; reject active-v2 mutation, root-only import, member omission, aliasing, and promotion.
- [ ] Add focused positive and hostile tests that retain every effect false.

## Verification

- This design can describe a future capture but cannot fetch, write a fresh cache, admit a source, or change F03.
- A later capture must re-fetch only this frozen identity and reproduce the closure offline.
