# F05.2 — Register profile-bound graphics checks

[Parent task](../README.md) · [Worker instructions](../../../../workflow.md)

Task: F05.2
Depends: F02, F05.1
Evidence: [receipt](evidence.md)

## Outcome

Frozen source and profile inputs bind each implementation leaf to exact device, browser, Wasm, guest,
and conformance checks through the tested F05.1 substrate.

The current sealed catalog has no admitted semantic implementation leaf, so it records
`profile_implementation_count: 0`. Its Vulkan registry diagnostic and baseline lanes cannot change
that state; a later semantic matrix leaf must supply the exact normative/full-suite pair.

## Starting points

- [F02 upstream pins](../../../01-contract/02-upstream-pins/README.md)
- [F05.1 generic runner](../01-generic-runner/README.md)
- [current verification commands](../../../../baseline.md)

## Checklist

- [x] Register each admitted leaf's exact command, profile, source revision, expected nonzero case count,
  artifact location, and required hardware/browser/guest prerequisites.
- [x] Bind device, browser, serial/threaded Wasm, guest, and conformance checks to frozen F02 inputs;
  reject stale, unpinned, or profile-mismatched registrations.
- [x] Require missing assets, browser, native hardware, guest image, or conformance suite to remain
  blocked and retain their command/output rather than silently skipping them.
- [x] Keep existing `make test` and `make web-pkg` lanes registered separately; label the native guest
  smoke transport-only until a later task proves standard Mesa behavior.
- [x] Add registration and negative tests for profile mismatch, stale pin, empty command, and missing
  mandatory evidence.
- [x] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- No profile-bound check is admitted until F02 is PASS-complete; this child is not a substitute for
  source admission, API behavior, browser execution, conformance, certification, or performance.
- The completed runner rejects a mismatched source/profile pair and records each mandatory unavailable
  prerequisite as blocked, not PASS.
