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

- [ ] Build a streaming upstream cache that accepts full-suite members above 8 MiB.
- [ ] Record all 98 ordered vk-default paths with revision, blob identity, bytes, and SHA-256.
- [ ] Preserve WSI, video, and extension paths as observed taxonomy, leaving unknown paths unknown.
- [ ] Reject missing, duplicated, reordered, mixed-revision, tampered, or symlinked members.
- [ ] State the aggregate size and oversize members without filtering them from the upstream root.

## Verification

The full root replays byte-for-byte from the immutable VCTS release. Any local storage
optimization is deferred to F02.5.3.4 and cannot redefine selector authority.
