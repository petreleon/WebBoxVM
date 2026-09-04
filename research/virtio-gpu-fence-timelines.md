# VirtIO-GPU fence timelines

## Purpose

This note records the narrow standards-facing fence layer used by the browser
3D transport. It improves completion correctness; it does not make the device
a Venus, Vulkan, or general VirGL renderer.

## Guest wire contract

`virtio_gpu_ctrl_hdr` defines `FENCE` as bit 0 and `INFO_RING_IDX` as bit 1.
The latter carries an eight-bit ring index followed by three padding bytes.
Linux emits `INFO_RING_IDX` only for a non-default context timeline and
associates the fence with that ring.

WebBoxVM accepts only those two bits. It rejects unknown bits, nonzero padding,
an unadvertised `INFO_RING_IDX`, and a ring-info flag without `FENCE`. Plain
fences retain the default ring (zero). This intentionally gives malformed
headers a deterministic error response instead of reinterpreting padding.

## Completion model

Each pending browser-owned 3D packet stores its `(ctx_id, generation, ring_idx)` timeline.
`gpu_3d_complete` refuses an acknowledgement for a later pending packet when
an earlier packet on that same timeline still has a guest completion attached.
After the earlier acknowledgement, the later one can complete normally.

A context recreation receives a new internal generation, so reuse of a numeric
context ID cannot make an old packet block its replacement's fence timeline.

Different rings are independent. This matches the reason ring information
exists: separate guest fence timelines must not accidentally serialize each
other. The control response retains the request's flags, fence ID, context ID,
and ring byte, so Linux can match the returned fence.

The browser currently serializes `GuestDisplay.present3d` calls and each
renderer waits for `GPUQueue.onSubmittedWorkDone()`. Therefore an accepted
same-timeline acknowledgement represents a completed WebGPU queue boundary,
rather than presentation merely having been scheduled.

## Evidence

`emulator/src/devices/virtio_gpu/tests/fence.rs` covers:

- feature-gated ring information and canonical header encoding;
- rejected out-of-order acknowledgements on one ring;
- exact returned fence fields; and
- independent completion of two negotiated rings.

The native AArch64 smoke guest creates a two-ring DRM context and issues its
deferred clear on ring 1. Linux accepts that `EXECBUF_RING_IDX` request only
after context-ring setup; a passing clear/readback therefore also exercises the
real driver emission and device acceptance of `FENCE | INFO_RING_IDX`.

The Linux UAPI defines the flags and byte-sized ring field, while the Linux
virtio-gpu fence driver shows that fence IDs signal prior fences on the same
DMA-fence context.

## Boundary and next work

This only governs WebBoxVM packets that already wait for a browser completion.
It does not yet schedule arbitrary VirtIO-GPU commands behind renderer work,
implement a capset-2/Venus command stream, translate Vulkan, or expose Vulkan
external-memory primitives. Those require a real renderer protocol and host
resource interop, neither of which browser WebGPU supplies today.

Sources: [Linux VirtIO-GPU UAPI](https://github.com/torvalds/linux/blob/master/include/uapi/linux/virtio_gpu.h), [Linux fence implementation](https://github.com/torvalds/linux/blob/master/drivers/gpu/drm/virtio/virtgpu_fence.c), [Linux submit path](https://github.com/torvalds/linux/blob/master/drivers/gpu/drm/virtio/virtgpu_submit.c), and [OASIS VirtIO GPU specification](https://docs.oasis-open.org/virtio/virtio/v1.3/virtio-v1.3.html).
