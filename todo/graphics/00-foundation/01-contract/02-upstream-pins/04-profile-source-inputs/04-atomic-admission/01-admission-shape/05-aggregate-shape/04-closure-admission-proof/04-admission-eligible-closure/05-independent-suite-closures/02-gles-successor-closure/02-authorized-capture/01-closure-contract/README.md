# F02.4.4.1.5.4.4.5.2.2.1 — Freeze the immutable GLES closure contract

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.4.4.5.2.2.1
Depends: F02.2, F02.4.2, F02.4.4.1.5.4.4.5.2.1
Evidence: [receipt](evidence.md)

Prerequisite lists: the [successor integration](../../01-successor-integration/README.md), the
[GLES audit](../../../../../../../../../02-gles-input-audit/README.md), and the
[F02 fetch policy](../../../../../../../../../../02-fetch-verifier/README.md).

## Outcome

A successor-only, hostile-tested contract describes the pinned Khronos GLES root, its four core
members, twelve configurations, optional-extension exclusion, and reproducible producer entrypoints.
Its identities use the existing F02 cache grammar but it remains unadmitted and cannot write cache state.

## Starting points

- [pinned root audit](../../../../../../../../../02-gles-input-audit/cts_closure.json)
- [pinned configurations](../../../../../../../../../02-gles-input-audit/cts_configurations.json)
- [multi-suite successor integration](../../01-successor-integration/README.md)

## Checklist

- [x] Bind the immutable root, four ordered core members, and optional-extension boundary to one revision.
- [x] Require each physical member's URL, digest, bytes, license, provenance, F02-compatible cache identity, and 8 MiB cap.
- [x] Bind the twelve core configurations and upstream producer entrypoints; this raw capture neither runs nor attests their output.
- [x] Keep the contract successor-only; reject active-v2 mutation, root-only import, member omission, aliasing, and promotion.
- [x] Add focused positive and hostile tests that retain every effect false.

## Verification

- This design can describe a future capture but cannot fetch, write a fresh cache, admit a source, or change F03.
- A later capture must re-fetch only this frozen identity and reproduce the closure offline.
