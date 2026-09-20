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

- [x] [F02.5.3.1 — Pin canonical full-suite roots](01-canonical-full-suite-roots/README.md)
- [x] [F02.5.3.2 — Ledger the complete GL and GLES suites](02-gl-gles-full-ledgers/README.md)
- [x] [F02.5.3.3 — Ledger the unfiltered Vulkan default suite](03-vulkan-default-full-ledger/README.md)
- [ ] [F02.5.3.4 — Build local shards and publish a no-claim receipt](04-local-shards-and-no-claim-receipt/README.md)

## Verification

- The acceptance record identifies the exact unmodified upstream full suite. A caller-selected subset
  may be useful for regression work, but cannot satisfy a profile's full-suite obligation.
- The 8 MiB cap applies only to WebBoxVM transforms. It does not erase, split, or relabel an upstream
  root or member, and no Vulkan default-suite record is described as a Khronos core-only selector.
