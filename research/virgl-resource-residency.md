# Browser GPU resource residency

## Question

How can bounded VirGL batches stop mapping pixels back after every draw without
letting `TRANSFER_FROM_HOST_3D` or a later command observe stale CPU pixels?

## Current boundary

Normal non-resident `VGB1` and `VGM1` packets render to the transient canvas
texture, map a full BGRA or RGBA readback, and replace the Rust resource shadow.
Eligible non-depth source-over batches use resident `VGB1` v6/v7 or `VGM1`
v2/v3. Eligible non-depth direct batches use `VGB1` v14/v15 or `VGM1` v10/v11;
their flags carry the exact nonzero RGBA write mask. Fresh versions render to a
bounded offscreen texture, copy it to the canvas, and acknowledge the producer
without pixel mapping. Replacement versions name an existing producer and
repaint that same texture for a later full redraw. A full `VGC1` clear uses
version 2 and the same optional predecessor contract. These paths avoid the
GPU-to-CPU copy, mapping latency, worker transfer, and copy into
`GpuResource::pixels`; sampled inputs remain bounded immutable snapshots rather
than GPU owners, except the narrow `VGM1` v12 path described below.

An already resident source can also copy to a fresh, equal-size offscreen target
through private `VRC1` v1. The browser issues `copyTextureToTexture` between two
durable targets; the source stays resident and the destination becomes resident
only after its matching completion. This intentionally excludes scanout,
partial/overlapping copies, cross-context sources, and a destination that is
already resident.

A fresh full-target non-depth one-texture or texture-color draw can name one
same-context, non-scanout resident source through `VGM1` v12. Its 16-byte source
record contains only canonical sampler state, dimensions, and producer sequence;
it carries no stale CPU pixels. The browser resolves that producer to an existing
durable texture and binds it with `TEXTURE_BINDING` while creating a fresh durable
output. A pending sample locks its source against transfer, copy, detach, and
unref until completion; missing or stale producers fail closed and release only
the new target.

WebGPU textures are device resources, while a canvas current texture is not a
durable guest resource. A real resident path therefore needs a bounded offscreen
texture plus an explicit control protocol; it cannot merely retain a canvas view.

## Authority state

For each eligible color resource, use one of these states:

```text
Cpu(epoch) -> Gpu(epoch, producer sequence) -> FullResolve(sequence) -> Cpu(epoch + 1)
                                           -> PartialReadback(sequence) -> Gpu(epoch, same producer)
                                           -> GpuCopy(sequence) -> Gpu(source, same) + Gpu(destination, new)
                                           -> GpuSample(sequence) -> Gpu(source, same) + Gpu(destination, new)
```

The Rust shadow is authoritative only in `Cpu`. `Gpu` names one successful
browser producer and is the authority for its full target. `FullResolve` reserves
one exact full-image readback. `PartialReadback` reserves one exact guest-visible
rectangle while deliberately leaving the CPU shadow stale; no second transfer or
mutation may consume an older shadow.

## Required invariants

1. A successful GPU completion can promote only the exact pending resource,
   context generation, full target rectangle, and expected predecessor sequence.
2. A guest-visible readback must resolve the matching producer before backing
   memory is written. Failure returns an error; it never writes stale bytes.
3. Partial guest uploads and copies cannot mutate a resident shadow. A full CPU
   replacement, matching readback, resource destruction, or VM reset ends the
   GPU owner; a browser-loss owner stays fail-closed until one of those paths
   re-establishes CPU authority.
4. A later GPU command either references the resident source on the GPU or
   resolves it before taking a CPU snapshot. It may not snapshot stale pixels.
5. Deferred acknowledgments preserve a resource's submission order as well as
   the existing VirtIO fence timeline order.
6. Each candidate captures a conservative CPU-authority epoch. Any later CPU
   replacement invalidates an unresolved candidate before it can become GPU
   authority.
7. A resident copy names exactly one stable source producer and one fresh full
   destination. A lost source, changed context, source/destination scanout, or
   failed browser copy releases only the new target and never changes either CPU
   shadow. Both resources stay locked against another copy or transfer until
   that completion settles.
8. A resident sample names one exact producer and no CPU snapshot. Its source
   must remain attached to the same context and unavailable to guest mutation
   until completion; the browser must bind that durable texture or reject it.

## First safe subset

The safe subset promotes a non-depth source-over or direct solid/mixed-material
`VirglBatch`, including a singleton rewritten to its batch envelope, or a
standalone `VirglClear`, whose rectangle exactly covers its color resource and
whose dimensions exceed 64 in at least one direction. The bounded sampler path
accepts snapshots no larger than 64×64, so that target cannot re-enter an
accepted batch as a sampled CPU texture. Depth batches remain CPU-synchronized
because later depth tests need their CPU depth shadow.

One additional fresh-target singleton is accepted: a one-texture or
texture-color source-over draw may reference one resident sampled color resource
instead of the normal `<=64x64` CPU snapshot. It cannot rekey its destination,
combine two textures, use depth, sample scanout, or share a batch.

This is an eligibility boundary, not a promise of general resource residency.
Only a same-context, non-scanout, full copy between equal-size color targets is
also admitted; partial, overlapping, CPU-observable, and general VirGL copies
remain on the CPU path.

## Protocol phases

1. Eligible source-over v6 `VGB1`/v2 `VGM1`, direct v14 `VGB1`/v10 `VGM1`,
   and v2 clear packets render into one of at most 16 persistent textures
   totaling 16 MiB, with an individual texture capped at 4 MiB, present through
   a GPU copy, and return a resident completion.
2. Eligible source-over v7 `VGB1`/v3 `VGM1`, direct v15 `VGB1`/v11 `VGM1`,
   or v2 clear packets with a nonzero predecessor repaint and rekey that exact
   texture only after GPU completion; Rust accepts it only while the resource
   still names that predecessor.
3. Rust stores the producer sequence only after the matching completion
   validates the pending effect and context generation.
4. A full `TRANSFER_FROM_HOST_3D` emits private `VGR1` v1 naming that producer.
   The browser maps the persistent texture; Rust validates the full image,
   refreshes its shadow, and only then writes the requested backing range and
   completion response. A strictly partial transfer emits `VGR1` v2 with a
   bounded source origin and size. Rust validates and converts only those pixels
   directly into the requested backing rows, without shadow mutation, damage, or
   producer release; the same GPU target remains authoritative.
5. When a full CPU replacement or a new CPU-synchronized render ends residency,
   Rust emits a no-ack `VGL1` release for the old producer. The browser destroys
   that cached texture; duplicate or delayed releases are harmless.
   Context and resource teardown use the same release path.
6. A strict full `RESOURCE_COPY_REGION` from a resident source emits `VRC1` v1.
   The browser copies the source texture into a fresh bounded target without a
   canvas transfer or pixel map. Rust validates the source owner and promotes
   only the destination; a stale completion emits `VGL1` for that new sequence.
7. `VGM1` v12 accepts one resident sampled source for a fresh non-depth
   singleton. Both Rust and the browser revalidate its producer, dimensions,
   context attachment, and bounded durable texture; only the new target is
   released after a stale completion.

## Cost model

| Operation | Current | Resident path |
| --- | --- | --- |
| Eligible draw completion | O(W×H) readback and transfer | O(1) host control; GPU presentation copy |
| Repeated full eligible draw | O(W×H) readback and transfer | Repaint and rekey one persistent GPU texture |
| First guest CPU read | Already paid per draw | O(W×H), once at the synchronization boundary |
| Partial guest CPU read (w×h) | O(W×H) map before scatter | O(w×h) map; retain GPU authority |
| Full resident copy | O(W×H) readback plus CPU pixel copy | One GPU texture copy; retain source and promote destination |
| Resident texture sample | O(W×H) readback plus upload | One sampled durable texture; no pixel payload or map |
| Resident lookup | — | O(1) keyed by resource ID |
| Identical vertex input | Upload every frame | Exact cached bytes skip `queue.writeBuffer` |
| Browser memory | Transient target | Explicit bounded texture budget |

The resident path improves draw-heavy workloads that do not read pixels on the
CPU between draws. It deliberately defers, rather than removes, the cost of a
guest-visible readback.

## Validation plan

- Promote only a matching full-target packet; reject stale, wrong-resource, and
  wrong-generation completions.
- Prove a deferred `VGR1` transfer writes exact scatter-backed bytes only after
  its matching readback.
- Prove `VGR1` v2 maps only its validated source rectangle, converts BGRA/RGBA
  correctly, writes no partial backing range on failure, and retains its producer.
- Prove an upload, unref, reset, and device loss cannot revive a prior owner.
- Compare a resident batch followed by transfer against the current readback
  fixture byte-for-byte, including BGRA/RGBA normalization.
- Prove direct full/RGB/partial masks keep their target resident, preserve the
  exact WebGPU write mask, and do not issue a mapped GPU readback.
- Prove a `VRC1` completion retains its source, promotes only its fresh target,
  uses `COPY_SRC`/`COPY_DST` texture usage, and releases a stale target alone.
- Prove `VGM1` v12 binds only an existing producer with `TEXTURE_BINDING`, maps
  no pixels, locks its source, and releases a stale new target alone.
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
