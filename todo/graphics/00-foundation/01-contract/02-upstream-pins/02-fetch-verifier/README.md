# F02.2 — Verify isolated source fetches and hashes

[Parent task](../README.md) · [Worker instructions](../../../../workflow.md)

Task: F02.2
Depends: F02.1
Evidence: pending

Prerequisite lists: [F02.1](../01-input-inventory/README.md).

## Outcome

A clean cache can fetch each pinned input, verify its bytes, and fail closed on corruption.

## Starting points

- [input inventory](../01-input-inventory/README.md)
- [baseline commands](../../../../baseline.md)
- [source-file limit test](../../../../../../emulator/tests/source_file_limits.rs)

## Checklist

- [ ] Implement a small standard-library fetch/hash verifier with an explicit external cache root;
  it must never vendor downloads into maintained source.
- [ ] Add isolated fixtures for successful fetches, wrong SHA-256, wrong revision metadata, and
  unavailable input, with stable non-network test inputs.
- [ ] Run a fresh temporary-cache fetch of every manifest entry and record exact command output,
  cache paths, checksums, and licenses in its receipt.
- [ ] Keep verifier, fixtures, and generated reports modular and at most 180 physical lines each.

## Verification

- A clean temporary build fetches every pinned entry and a deliberately wrong hash fails.
- Unit tests prove that a network failure, redirect policy violation, or malformed manifest cannot
  produce an accepted cache entry.
