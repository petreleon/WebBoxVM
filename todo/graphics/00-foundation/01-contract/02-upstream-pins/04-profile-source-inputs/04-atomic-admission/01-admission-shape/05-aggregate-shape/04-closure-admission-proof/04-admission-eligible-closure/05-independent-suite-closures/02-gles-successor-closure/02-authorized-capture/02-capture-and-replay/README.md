# F02.4.4.1.5.4.4.5.2.2.2 — Capture and replay the complete GLES closure

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.4.4.5.2.2.2
Depends: F02.4.4.1.5.4.4.5.2.2.1
Evidence: [receipt](evidence.md)

Prerequisite lists: the [closure contract](../01-closure-contract/README.md), the
[GLES audit](../../../../../../../../../02-gles-input-audit/README.md), and the
[F02 fetch policy](../../../../../../../../../../02-fetch-verifier/README.md).

## Outcome

One fresh, bounded capture obtains the frozen root, all four core members, and the explicit excluded
member, then proves an offline replay reconstructs the same complete closure without substitution.
The capture remains a successor candidate and does not itself change F03.

## Starting points

- [closure contract](../01-closure-contract/README.md)
- [GLES closure record](../../../../../../../../../02-gles-input-audit/cts_closure.json)
- [GLES configurations](../../../../../../../../../02-gles-input-audit/cts_configurations.json)

## Checklist

- [x] Fetch the root, every required core member, and the exclusion witness under the frozen contract.
- [x] Verify regular-file safety, immutable URL/revision, digest, byte count, license, provenance, cache path, and 8 MiB cap.
- [x] Parse the root and prove the captured members, twelve configurations, and exclusion exactly match the contract.
- [x] Rebuild the closure from the sealed cache with network disabled; reject root-only, partial, stale, and substituted replay.
- [x] Record bounded capture artifacts and focused positive and hostile results without claiming CTS execution.

## Verification

- A capture is not source admission, a guest run, a conformance result, or a performance measurement.
- Any network, producer, cache, or replay failure leaves the successor candidate unadmitted.
