# F05.2 — Register profile-bound graphics checks

[Parent task](../README.md) · [Worker instructions](../../../../workflow.md)

Task: F05.2
Depends: F02, F05.1
Evidence: pending

## Outcome

Frozen source and profile inputs bind each implementation leaf to exact device, browser, Wasm, guest,
and conformance checks through the tested F05.1 substrate.

## Starting points

- [F02 upstream pins](../../../01-contract/02-upstream-pins/README.md)
- [F05.1 generic runner](../01-generic-runner/README.md)
- [current verification commands](../../../../baseline.md)

## Checklist

- [ ] Register each admitted leaf's exact command, profile, source revision, expected nonzero case count,
  artifact location, and required hardware/browser/guest prerequisites.
- [ ] Bind device, browser, serial/threaded Wasm, guest, and conformance checks to frozen F02 inputs;
  reject stale, unpinned, or profile-mismatched registrations.
- [ ] Require missing assets, browser, native hardware, guest image, or conformance suite to remain
  blocked and retain their command/output rather than silently skipping them.
- [ ] Keep existing `make test` and `make web-pkg` lanes registered separately; label the native guest
  smoke transport-only until a later task proves standard Mesa behavior.
- [ ] Add registration and negative tests for profile mismatch, stale pin, empty command, and missing
  mandatory evidence.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- No profile-bound check is admitted until F02 is PASS-complete; this child is not a substitute for
  source admission, API behavior, browser execution, conformance, certification, or performance.
- The completed runner rejects a mismatched source/profile pair and records each mandatory unavailable
  prerequisite as blocked, not PASS.
