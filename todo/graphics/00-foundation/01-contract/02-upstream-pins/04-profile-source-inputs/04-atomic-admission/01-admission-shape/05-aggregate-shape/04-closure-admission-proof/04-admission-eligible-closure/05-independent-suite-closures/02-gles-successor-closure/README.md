# F02.4.4.1.5.4.4.5.2 — Resolve the GLES immutable successor closure

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.4.4.5.2
Depends: F02.2, F02.4.2, F02.4.4.1.5.4.4.5.1
Evidence: [receipt](evidence.md); child evidence: [successor integration](01-successor-integration/evidence.md), [authorized capture](02-authorized-capture/evidence.md)

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

- [x] [Define a coexistence-safe GLES successor integration](01-successor-integration/README.md)
- [x] [Capture and atomically admit the immutable GLES closure](02-authorized-capture/README.md)

## Verification

- No candidate becomes admitted, cached as fresh, or visible to F03 until the full atomic transition passes.
- This task does not claim GLES CTS execution, guest API support, browser behavior, conformance, or performance.
