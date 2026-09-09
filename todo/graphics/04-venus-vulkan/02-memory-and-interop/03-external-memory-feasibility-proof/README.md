# V06 — Prove the stock Venus external-memory and synchronization boundary

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: V06
Depends: F04, V03, V04, V05
Evidence: pending

Prerequisite lists: [F04](../../../00-foundation/02-reproducibility/01-feasibility/README.md), [V03](../../01-real-protocol/03-rings-replies-negotiation/README.md), [V04](../01-memory-allocation-binding/README.md), [V05](../02-mapped-memory-coherence/README.md).

## Outcome

Every external-memory/synchronization requirement of stock Mesa Venus has a demonstrated browser
implementation or an explicit goal-blocking issue.

## Starting points

- [research/venus-foundations.md](../../../../../research/venus-foundations.md)
- [research/virgl2-capset.md](../../../../../research/virgl2-capset.md)
- [emulator/src/devices/virtio_gpu/blob/map.rs](../../../../../emulator/src/devices/virtio_gpu/blob/map.rs)
- [emulator/src/devices/virtio_gpu/fence.rs](../../../../../emulator/src/devices/virtio_gpu/fence.rs)

## Checklist

- [ ] Trace the pinned Mesa Venus driver's actual external-memory, handle, blob mapping, fence, and
  host Vulkan assumptions instead of assuming WebGPU exposes native handles.
- [ ] Implement or prove a semantics-preserving browser-local realization for each mandatory
  interaction and run the stock driver through it.
- [ ] If any mandatory interaction cannot be realized, record the precise unsupported operation and
  an architecture decision task; do not silently switch to a patched guest protocol, remote GPU, or
  native helper.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Evidence links every required interaction to trace bytes, an implementation path, and a passing
  stock-Mesa guest/browser test; a byte-buffer allocation alone is insufficient.
- Any unresolved mandatory requirement remains a visible blocker for Vulkan initialization/capset
  exposure and for the overall compatibility goal, with the goal scope unchanged.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
