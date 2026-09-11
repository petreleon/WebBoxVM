# F05.1 — Implement the profile-independent runner substrate

[Parent task](../README.md) · [Worker instructions](../../../../workflow.md)

Task: F05.1
Depends: F01
Evidence: pending

## Outcome

A small local runner executes named checks, records reproducible observations, and makes failures or
missing prerequisites explicit without asserting any graphics profile or API support.

## Starting points

- [Makefile](../../../../../../Makefile)
- [receipt template](../../../../evidence-template.md)
- [native transport smoke](../../../../../../scripts/virgl_guest_transport_smoke.sh)

## Checklist

- [ ] Create a small `scripts/graphics/` CLI and helper modules for named local command execution.
- [ ] Capture command, revision, dirty-diff hash, tool versions, duration, exit status, expected count,
  observed count, and artifact hashes in a machine-readable local result.
- [ ] Reject unknown or empty selections and propagate a deliberately failing child command unchanged.
- [ ] Report missing executable, asset, browser, hardware, or permission prerequisites as explicit local
  blocked results without treating them as a passing run.
- [ ] Test success, failure, zero-match, malformed input, and missing-prerequisite behavior.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- The runner has no profile inventory, source-admission, guest-API, browser-GPU, conformance, or
  performance claim; it only preserves observations for later profile-bound checks.
- Its focused tests prove one successful command, one child failure, empty/unknown selection rejection,
  and one missing prerequisite report with nonzero case accounting.
