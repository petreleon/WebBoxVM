# F02.5.3 — Record unmodified full conformance-suite roots

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F02.5.3
Depends: F02.5.1
Evidence: pending

## Outcome

Each final profile pins its actual Khronos CTS or must-pass root. Vulkan uses the unfiltered official
`vk-default.txt` root at a fixed VCTS release; a WebBoxVM case map can support development but cannot
replace the full suite or be called a Vulkan-1.4-core Khronos selector.

## Starting points

- [candidate catalog](../../04-profile-source-inputs/candidate_catalog.py)
- [VCTS README](https://github.com/KhronosGroup/VK-GL-CTS/blob/main/external/vulkancts/README.md)
- [Vulkan CTS release](https://github.com/KhronosGroup/VK-GL-CTS/releases/tag/vulkan-cts-1.4.6.2)

## Checklist

- [ ] Re-fetch and verify the released GL, GLES, and Vulkan root selectors and source revisions.
- [ ] Record all included suite categories, including Vulkan WSI, video, and extensions, without relabeling.
- [ ] Keep any WebBoxVM case map, catalog, or byte-preserving shard separate from selector authority.
- [ ] Require each local shard to be at most 8 MiB and prove byte-for-byte reassembly against its upstream blob.
- [ ] Reject filtered, reordered, incomplete, mutable, or falsely core-only suite records with focused tests.
- [ ] Attach a receipt that states no CTS case was executed and no conformance claim follows.

## Verification

- The acceptance record identifies the exact unmodified upstream full suite. A caller-selected subset
  may be useful for regression work, but cannot satisfy a profile's full-suite obligation.
