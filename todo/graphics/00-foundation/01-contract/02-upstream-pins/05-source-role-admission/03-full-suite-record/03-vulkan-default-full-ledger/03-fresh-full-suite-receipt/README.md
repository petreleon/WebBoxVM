# F02.5.3.3.3 — Refresh the full Vulkan suite and receipt

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F02.5.3.3.3
Depends: F02.5.3.3.1, F02.5.3.3.2
Evidence: pending

## Outcome

An externally cached fresh replay verifies every immutable VCTS member and publishes a
no-claim receipt. The raw payload is not committed, filtered, or represented as a CTS run.

## Starting points

- [Vulkan ledger task](../01-vulkan-ledger-taxonomy/README.md)
- [cache replay task](../02-streaming-cache-replay/README.md)
- [live capture entrypoint](../../../../04-profile-source-inputs/04-atomic-admission/01-admission-shape/05-aggregate-shape/05-vulkan-source-contract-v2/03-external-closure-cache/02-live-closure/vcts_live_capture.py)

## Checklist

- [ ] Preflight external cache space and freshness before network access.
- [ ] Fetch and verify the root plus all 98 ordered members exactly once.
- [ ] Replay all cache members offline with zero network reuse and one aggregate digest.
- [ ] State aggregate bytes and oversize members without excluding them.
- [ ] Reject a forged or positive-claim receipt and report zero CTS executions.

## Verification

`make graphics-vulkan-full-suite-receipt-test` must cover receipt and replay mutations;
the live receipt must record actual network, cache, disk, and failure/skip observations.
