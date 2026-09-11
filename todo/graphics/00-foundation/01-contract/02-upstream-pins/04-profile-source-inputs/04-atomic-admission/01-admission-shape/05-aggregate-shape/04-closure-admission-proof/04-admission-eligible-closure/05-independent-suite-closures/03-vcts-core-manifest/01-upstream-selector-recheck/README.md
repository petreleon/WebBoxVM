# F02.4.4.1.5.4.4.5.3.1 — Recheck the official VCTS selector

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.4.4.5.3.1
Depends: F02.4.3, F02.4.4.1.5.5, F02.4.4.1.5.4.4.5.1
Evidence: [PASS probe receipt](evidence.md)

## Outcome

Freeze a reproducible, read-only observation of Khronos's current immutable Vulkan CTS 1.4 release
roots. It must distinguish an absent explicit Vulkan 1.4 core manifest from the existing broader
`vk-default` diagnostic and retain a blocker rather than derive a local selector.

## Starting points

- [VCTS handoff](../../../../../05-vulkan-source-contract-v2/05-aggregate-handoff/vcts_aggregate_handoff.json)
- [successor boundary](../../01-successor-boundary/README.md)
- [historical VCTS blocker](../../../../../03-vcts-core-closure/evidence.md)

## Checklist

- [x] Pin the current official tag/release observation and the immutable root directory names.
- [x] Bind the existing `vk-default` identity and WSI/video diagnostic without treating it as core.
- [x] Reject local derivation, taxonomy promotion, root-only evidence, opaque splitting, and VCTS-as-Docs.
- [x] Preserve the V2 handoff, active F02/F03, admission, and support effects as exact false values.
- [x] Add focused positive/hostile checks and an external-blocker receipt with live source links.

## Verification

- This research probe passes when it reproducibly records the upstream selector state. Its current
  external condition is `BLOCKED` with
  `missing-khronos-published-immutable-explicit-vulkan-1.4-core-vcts-manifest`.
- A future tag alone is not sufficient: Khronos must publish an immutable artifact whose scope explicitly
  asserts complete Vulkan 1.4 core selection.
