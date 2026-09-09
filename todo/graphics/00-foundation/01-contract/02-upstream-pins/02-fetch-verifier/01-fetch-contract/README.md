# F02.2.1 — Define the fail-closed fetch and cache contract

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F02.2.1
Depends: F02.1
Evidence: pending

Prerequisite lists: [F02.1](../../01-input-inventory/README.md).

## Outcome

A small standard-library verifier accepts only declared immutable inputs and writes verified bytes
atomically below an explicit external cache root.

## Starting points

- [input inventory](../../01-input-inventory/README.md)
- [graphics workflow](../../../../../workflow.md)
- [source-file limit test](../../../../../../../emulator/tests/source_file_limits.rs)

## Checklist

- [ ] Define cache-root, URL, redirect, revision, and atomic-write policy with no repository cache.
- [ ] Implement modular manifest loading, byte-count/hash verification, and safe cache naming.
- [ ] Reject malformed inputs before creating or accepting a cache entry.
- [ ] Record focused command, nonzero test count, and observed cache layout in a receipt.

## Verification

- A declared immutable input can reach only a cache path below the caller-supplied external root.
- Invalid URL, revision, path, or metadata is rejected before a cache entry is accepted.
