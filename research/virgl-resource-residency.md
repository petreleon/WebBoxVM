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
same-context, non-scanout resident source through `VGM1` v12 source-over or v13
opaque replacement. Its 16-byte source record contains only canonical sampler
state, dimensions, and producer sequence; it carries no stale CPU pixels. The
browser resolves that producer to an existing durable texture and binds it with
`TEXTURE_BINDING` while creating a fresh durable output. A pending sample locks
its source against transfer, copy, detach, and unref until completion; missing or
stale producers fail closed and release only the new target.

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
texture-color source-over or opaque replacement draw may reference one resident
sampled color resource instead of the normal `<=64x64` CPU snapshot. It cannot
rekey its destination, combine two textures, use depth, sample scanout, or share
a batch.

This is an eligibility boundary, not a promise of general resource residency.
Only a same-context, non-scanout, full copy between equal-size color targets is
also admitted; partial, overlapping, CPU-observable, and general VirGL copies
remain on the CPU path.

## Protocol, cost, and validation

[Protocol phases, cost model, and validation plan](virgl-resource-residency-validation.md)
retain the exact packet lifecycle, deferred-readback tradeoffs, and regression
matrix for this bounded protocol.

## Scope boundary

This is a private browser residency protocol for the existing bounded VirGL
subset. It does not add general OpenGL, Vulkan, standard Venus, external memory,
or native interprocess synchronization. Mesa Venus requires VirtIO resource
blobs, host-visible memory, and host external-memory support that WebGPU does
not expose to this VM.

Sources: [W3C WebGPU](https://www.w3.org/TR/webgpu/), [VirtIO GPU
specification](https://github.com/oasis-tcs/virtio-spec/blob/master/device-types/gpu/description.tex),
and [Mesa Venus requirements](https://docs.mesa3d.org/drivers/venus.html).
