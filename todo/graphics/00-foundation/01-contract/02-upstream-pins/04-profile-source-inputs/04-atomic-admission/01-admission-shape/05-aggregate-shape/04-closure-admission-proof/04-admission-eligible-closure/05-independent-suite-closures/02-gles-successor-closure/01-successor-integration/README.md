# F02.4.4.1.5.4.4.5.2.1 — Define a coexistence-safe GLES successor integration

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.4.4.5.2.1
Depends: F02.4.2, F02.4.4.1.5.4.4.5.1, F02.4.4.1.5.4.4.6
Evidence: [receipt](evidence.md)

Prerequisite lists: the [independent-suite boundary](../../01-successor-boundary/README.md), the
[Docs successor transition](../../../06-successor-inventory-transition/README.md), and the
[GLES closure probe](../../../../../../01-gles-closure-probe/README.md).

## Outcome

A future-only aggregate integration can represent separate `vulkan-docs` and `gles-cts` wrappers
without mutating active schema-v2, treating either existing design as admitted, or permitting one
closure to stand in for the other. The GLES wrapper remains an unadmitted shape until an authorized
fresh capture and atomic F02/F03 transition pass.

## Starting points

- [active inventory layout](../../../../../../../../../01-input-inventory/README.md)
- [F02 fetch policy](../../../../../../../../../02-fetch-verifier/README.md)
- [GLES closure record](../../../../../../../../02-gles-input-audit/cts_closure.json)
- [Docs schema-v3 design](../../../06-successor-inventory-transition/README.md)

## Checklist

- [x] Freeze active schema-v2 and the independent Docs schema-v3 design as non-admitted predecessors.
- [x] Define a future aggregate wrapper shape with exactly one `vulkan-docs` and one `gles-cts` family.
- [x] Bind the GLES root, ordered core members, configurations, extension exclusion, and 8 MiB member rule.
- [x] Require separate fresh closure, cache, authority, and atomic F02/F03 revalidation conditions for each wrapper.
- [x] Reject aliasing, omitted wrappers, root-only import, cross-wrapper substitution, and active-state promotion.
- [x] Add focused positive and hostile integration tests with every effect false.

## Verification

- The integration is a design-only record; it cannot amend the active inventory or authorize a capture.
- It must preserve the current rejected GLES candidate and all Vulkan V2 false states.
