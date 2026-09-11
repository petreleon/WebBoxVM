# F02.5.3.3 — Ledger the unfiltered Vulkan default suite

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F02.5.3.3
Depends: F02.5.3.1
Evidence: pending

## Outcome

An offline-replayable ledger preserves every ordered member of the official Vulkan
vk-default.txt root, including its actual WSI, video, extension, and currently
uncategorized paths. The ledger is not a claim that all entries are Vulkan core.

## Starting points

- [VCTS closure ledger](../../../04-profile-source-inputs/04-atomic-admission/01-admission-shape/05-aggregate-shape/05-vulkan-source-contract-v2/03-external-closure-cache/02-live-closure/vcts_closure_ledger.json)
- [Vulkan CTS release](https://github.com/KhronosGroup/VK-GL-CTS/releases/tag/vulkan-cts-1.4.6.2)

## Checklist

- [x] [F02.5.3.3.1 — Bind the Vulkan ledger and observed taxonomy](01-vulkan-ledger-taxonomy/README.md)
- [x] [F02.5.3.3.2 — Prove streaming cache replay](02-streaming-cache-replay/README.md)
- [ ] [F02.5.3.3.3 — Refresh the full Vulkan suite and receipt](03-fresh-full-suite-receipt/README.md)

## Verification

The full root replays byte-for-byte from the immutable VCTS release. Any local storage
optimization is deferred to F02.5.3.4 and cannot redefine selector authority.

## Split rationale

The exact member ledger/taxonomy, safe large-member cache, and expensive live refresh
have different failure boundaries. The split ensures a recorded shape is not confused
with a downloaded payload or a CTS execution result.
