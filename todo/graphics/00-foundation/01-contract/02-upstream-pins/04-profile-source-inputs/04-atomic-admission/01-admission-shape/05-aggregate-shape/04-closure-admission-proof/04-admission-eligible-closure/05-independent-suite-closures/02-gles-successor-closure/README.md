# F02.4.4.1.5.4.4.5.2 — Resolve the GLES immutable successor closure

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.4.4.5.2
Depends: F02.2, F02.4.2, F02.4.4.1.5.4.4.5.1
Evidence: pending

Prerequisite lists: the [successor boundary](../01-successor-boundary/README.md), the
[GLES closure probe](../../../../../01-gles-closure-probe/README.md), and the
[GLES input audit](../../../../../../../02-gles-input-audit/README.md).

## Outcome

Resolve the currently rejected GLES multi-file candidate only through a fresh, scope-complete,
immutable successor closure. The four known selectors remain pre-admission diagnostic data until that
proof exists; a partial or locally reconstructed closure is not a substitute.

## Starting points

- [GLES closure record](../../../../../../../02-gles-input-audit/cts_closure.json)
- [GLES configurations](../../../../../../../02-gles-input-audit/cts_configurations.json)
- [F02 source policy](../../../../../../../../02-fetch-verifier/README.md)
- [successor boundary](../01-successor-boundary/README.md)

## Checklist

- [ ] Obtain a policy-authorized immutable closure whose selected members, scope, and producer are explicit.
- [ ] Validate every physical member's URL, revision, digest, bytes, license, provenance, cache, and 8 MiB cap.
- [ ] Bind all required configurations and retain the explicit optional-extension exclusion.
- [ ] Reproduce a fresh complete capture and offline replay without root-only or partial substitution.
- [ ] Reject stale, mutable, duplicate, over-cap, configuration-drift, and scope-expanded evidence.
- [ ] Record focused evidence or retain the first external closure-authority blocker.

## Verification

- No candidate becomes admitted, cached as fresh, or visible to F03 until the full atomic transition passes.
- This task does not claim GLES CTS execution, guest API support, browser behavior, conformance, or performance.
