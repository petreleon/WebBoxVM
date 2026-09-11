# F02.4.4.1.5.4.4.5.2.2 — Capture and atomically admit the immutable GLES closure

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.4.4.5.2.2
Depends: F02.2, F02.4.2, F02.4.4.1.5.4.4.5.2.1
Evidence: pending

Prerequisite lists: the [successor integration](../01-successor-integration/README.md), the
[GLES closure probe](../../../../../../01-gles-closure-probe/README.md), and the
[F02 source policy](../../../../../../../../../02-fetch-verifier/README.md).

## Outcome

Admit the GLES successor only after a policy-authorized immutable producer supplies the complete
root-and-member closure, its exact configurations and exclusion boundary, and the aggregate transition
validates fresh cache capture plus all F02/F03 consumers atomically.

## Starting points

- [GLES audit](../../../../../../../../02-gles-input-audit/README.md)
- [successor integration](../01-successor-integration/README.md)
- [F02 source policy](../../../../../../../../../02-fetch-verifier/README.md)

## Checklist

- [ ] [Freeze the immutable GLES closure contract](01-closure-contract/README.md)
- [ ] [Capture and replay the complete GLES closure](02-capture-and-replay/README.md)
- [ ] [Atomically revalidate successor consumers](03-atomic-consumers/README.md)

## Verification

- No candidate becomes admitted, cached as fresh, or visible to F03 until the full atomic transition passes.
- This task does not claim GLES CTS execution, guest API support, browser behavior, conformance, or performance.
