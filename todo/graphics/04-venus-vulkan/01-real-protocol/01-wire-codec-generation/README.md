# V01 — Generate a bounded decoder for the pinned Venus protocol

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: V01
Depends: F02, F04
Evidence: pending

Prerequisite lists: [F02](../../../00-foundation/01-contract/02-upstream-pins/README.md), [F04](../../../00-foundation/02-reproducibility/01-feasibility/README.md).

## Outcome

Real upstream Venus wire records decode through reproducible generated modules with audited framing
and extension handling.

## Starting points

- [research/venus-foundations.md](../../../../../research/venus-foundations.md)
- [research/renderer-blob-ordering.md](../../../../../research/renderer-blob-ordering.md)
- [emulator/src/devices/virtio_gpu/three_d/virgl/stream/decode.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/stream/decode.rs)

## Checklist

- [ ] Pin the Venus protocol generator inputs and establish the exact wire/version relationship to
  the selected stock Mesa guest.
- [ ] Generate decoders in files of at most 180 lines; begin with initialization records and create
  separate descendants per remaining opcode family.
- [ ] Validate lengths, arrays, alignment, enum/flag values, optional structures, and pNext chains;
  reject malformed/unsupported required records with the protocol-defined error behavior.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Decoder fixtures round-trip real Mesa initialization records and reject truncation, integer
  overflow, invalid pNext chains, and malformed nested arrays without allocation spikes or state
  mutation.
- A deterministic regeneration reproduces the same checked-in shard hashes; record the generator
  command as a new deliverable rather than pretending an existing Venus generator command is
  present.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
