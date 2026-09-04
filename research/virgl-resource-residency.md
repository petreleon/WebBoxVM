# Browser GPU resource residency

## Question

How can bounded VirGL batches stop mapping pixels back after every draw without
letting `TRANSFER_FROM_HOST_3D` or a later command observe stale CPU pixels?

## Current boundary

`VGB1` and `VGM1` render to the transient canvas texture, map a full BGRA or
RGBA readback, and replace the Rust resource shadow. That is correct but makes
each batch pay a GPU-to-CPU copy, mapping latency, worker transfer, and copy
into `GpuResource::pixels`.

WebGPU textures are device resources, while a canvas current texture is not a
durable guest resource. A real resident path therefore needs a bounded offscreen
texture plus an explicit control protocol; it cannot merely retain a canvas view.

## Authority state

For each eligible color resource, use one of these states:

```text
Cpu(epoch) -> Gpu(epoch, producer sequence) -> Resolving(sequence) -> Cpu(epoch + 1)
```

The Rust shadow is authoritative only in `Cpu`. `Gpu` names one successful
browser producer and is the authority for its full target. `Resolving` reserves
one exact readback; no second transfer or mutation may consume an older shadow.

## Required invariants

1. A successful GPU completion can promote only the exact pending resource,
   context generation, full target rectangle, and producer sequence.
2. A guest-visible readback must resolve the matching producer before backing
   memory is written. Failure returns an error; it never writes stale bytes.
3. Guest uploads, inline writes, copies, resource destruction, reset, and
   browser device loss invalidate or resolve a GPU owner before CPU mutation.
4. A later GPU command either references the resident source on the GPU or
   resolves it before taking a CPU snapshot. It may not snapshot stale pixels.
5. Deferred acknowledgments preserve a resource's submission order as well as
   the existing VirtIO fence timeline order.

## First safe subset

The first implementation should promote only a non-depth `VirglBatch` whose
rectangle exactly covers its color resource and whose dimensions exceed 64 in
at least one direction. The current bounded sampler path accepts snapshots no
larger than 64×64, so that target cannot re-enter an accepted batch as a sampled
CPU texture. Depth batches remain CPU-synchronized because later depth tests
need their depth state.

This is an eligibility boundary, not a promise of general resource residency.
Copies and other CPU consumers need a deferred-resolve continuation before they
are admitted to the resident path.

## Protocol phases

1. Render eligible batches into a bounded persistent texture, present it with a
   GPU copy, and return a resident completion instead of pixel data.
2. Store the producer sequence in Rust only after that completion validates the
   pending effect and context generation.
3. On `TRANSFER_FROM_HOST_3D`, emit a private deferred `VGR1` request naming
   the producer. The browser maps the persistent texture; Rust validates the
   full image, refreshes its shadow, and only then writes the requested backing
   range and completion response.
4. Add GPU source references and copy continuations before expanding eligibility
   to sampled targets or general VirGL streams.

## Cost model

| Operation | Current | Resident path |
| --- | --- | --- |
| Eligible draw completion | O(W×H) readback and transfer | O(1) host control; GPU presentation copy |
| First guest CPU read | Already paid per draw | O(W×H), once at the synchronization boundary |
| Resident lookup | — | O(1) keyed by resource ID |
| Browser memory | Transient target | Explicit bounded texture budget |

The resident path improves draw-heavy workloads that do not read pixels on the
CPU between draws. It deliberately defers, rather than removes, the cost of a
guest-visible readback.

## Validation plan

- Promote only a matching full-target packet; reject stale, wrong-resource, and
  wrong-generation completions.
- Prove a deferred transfer writes exact scatter-backed bytes only after its
  matching readback.
- Prove an upload, unref, reset, and device loss cannot revive a prior owner.
- Compare a resident batch followed by transfer against the current readback
  fixture byte-for-byte, including BGRA/RGBA normalization.
- Measure mapped readbacks per N eligible draws and report browser/device data
  separately from guest protocol correctness.

## Scope boundary

This is a private browser residency protocol for the existing bounded VirGL
subset. It does not add general OpenGL, Vulkan, standard Venus, external memory,
or native interprocess synchronization. Mesa Venus requires VirtIO resource
blobs, host-visible memory, and host external-memory support that WebGPU does
not expose to this VM.

Sources: [W3C WebGPU](https://www.w3.org/TR/webgpu/), [VirtIO GPU
specification](https://github.com/oasis-tcs/virtio-spec/blob/master/device-types/gpu/description.tex),
and [Mesa Venus requirements](https://docs.mesa3d.org/drivers/venus.html).
