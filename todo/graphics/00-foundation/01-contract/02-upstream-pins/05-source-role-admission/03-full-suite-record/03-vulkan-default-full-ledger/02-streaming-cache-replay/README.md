# F02.5.3.3.2 — Prove streaming cache replay

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F02.5.3.3.2
Depends: F02.5.3.3.1
Evidence: [receipt](evidence.md)

## Outcome

A safe external cache can stream and verify the complete immutable member ledger,
including members above 8 MiB, then replay it offline without changing source authority.

## Starting points

- [Vulkan ledger task](../01-vulkan-ledger-taxonomy/README.md)
- [existing cache contract](../../../../04-profile-source-inputs/04-atomic-admission/01-admission-shape/05-aggregate-shape/05-vulkan-source-contract-v2/03-external-closure-cache/01-cache-contract/vcts_cache_contract.py)

## Checklist

- [x] Use bounded streaming verification rather than the small-input source fetcher.
- [x] Bind cache paths to exact member identity and reject symlink or path escape.
- [x] Verify Git blob SHA-1, SHA-256, bytes, order, and aggregate replay.
- [x] Reject tampered, missing, mixed-revision, or oversize-in-policy members.
- [x] Keep every raw member outside the local 8 MiB transform cap.

## Verification

`make graphics-vulkan-cache-replay-test` must exercise a fixture above 8 MiB, streaming
digest checks, offline replay, and hostile filesystem/content mutations.
