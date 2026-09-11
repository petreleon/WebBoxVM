# F02.4.4.1.5.4.4.5.3.2 — Validate an authoritative core manifest

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.4.4.5.3.2
Depends: F02.4.3, F02.4.4.1.5.5, F02.4.4.1.5.4.4.5.1, F02.4.4.1.5.4.4.5.3.1
Status: blocked
Blocked-by: external/khronos-vulkan-14-core-vcts-manifest
Evidence: [blocker record](evidence.md)

## Outcome

Validate a future Khronos-published immutable manifest only when it explicitly asserts complete Vulkan
1.4 core scope and provenance. Do not use a local selector, an inferred taxonomy, a root-only assertion,
or a Docs artifact as a substitute.

## Starting points

- [upstream selector recheck](../01-upstream-selector-recheck/README.md)
- [F02 source policy](../../../../../../../../../02-fetch-verifier/README.md)

## Checklist

- [ ] Bind the immutable upstream locator, revision, provenance, and explicit Vulkan 1.4 core claim.
- [ ] Validate complete ordered recursive membership without local selection or opaque splitting.
- [ ] Reject an absent, partial, stale, aliased, broader, or non-Khronos manifest.
- [ ] Preserve `vk-default` as a diagnostic and every active/admission/support effect false.

## Verification

- This child remains open until the authoritative artifact exists and validates.
- Its success does not capture content, change F02/F03, or execute a conformance suite.
