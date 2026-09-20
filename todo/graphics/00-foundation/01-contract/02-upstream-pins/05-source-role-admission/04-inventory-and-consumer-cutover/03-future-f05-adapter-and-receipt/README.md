# F02.5.4.3 — Expose the future F05 adapter and aggregate receipt

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F02.5.4.3
Depends: F02.5.4.1, F02.5.4.2
Evidence: [receipt](evidence.md)

## Outcome

A small future-F05-facing accessor admits only the same complete sealed source contract used by F03.
It stays profile-neutral, rejects absent or partial provenance, and records the fresh-cache and
downstream checks in an aggregate no-claim receipt.

## Starting points

- [F05 generic runner](../../../../../02-reproducibility/02-check-runner/01-generic-runner/README.md)
- [F03 profile gate](../../../../03-feature-matrix/01-profile-scope/validate_profile_scope.py)
- [role-aware source contract](../01-role-aware-source-contract/README.md)

## Checklist

- [x] Provide a public profile-neutral accessor for a complete admitted source contract.
- [x] Reject an absent, partial, stale, mixed, or auxiliary-only contract at the F05 boundary.
- [x] Keep F05.1 source- and profile-independent; do not fabricate a completed profile registration.
- [x] Run the combined fresh-cache, F03, matrix, and downstream accessor checks.
- [x] Publish an aggregate self-hashed receipt with zero CTS executions and all qualification claims false.
- [x] Update the immediate parent checkboxes only after inspecting every required result.

## Verification

The F05-facing surface proves provenance admission only. It does not run CTS, register a supported
profile, or make guest API, browser, certification, or performance claims.
