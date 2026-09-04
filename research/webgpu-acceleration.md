# Research: WebGPU acceleration for WebBoxVM

## Question

Where can WebGPU accelerate WebBoxVM without weakening ARM64 correctness or
claiming acceleration that only exists in a host-side demo?

## Decision

Use WebGPU as the presentation backend for a guest-visible VirtIO-GPU device.
Keep ARM64 execution, address translation, interrupts, and guest RAM authoritative in Rust/Wasm.

The display foundation is the standard VirtIO-GPU 2D command path:

```text
Linux DRM / virtio-gpu
        | controlq commands
        v
Rust VirtioGpu device
        | host-private resources and coalesced scanout damage
        v
Wasm host adapter / VM worker
        | transferable BGRA8 dirty rectangles
        v
Persistent WebGPU texture -> browser canvas
```

This is a guest-visible display path with host-side WebGPU presentation.
Linux discovers device ID 16 and uses its unmodified VirtIO-GPU driver. Resource creation, backing DMA, format normalization, and transfers stay on the CPU in Rust/Wasm.
WebGPU only uploads the resulting dirty BGRA8 rectangle, samples a host texture,
and presents it on the browser canvas; it does not accelerate guest rendering.

The original 3D path uses the same stock Linux driver but an explicitly private, experimental capability set.
Its bounded guest-command path is not VirGL, Venus, OpenGL, Vulkan, or general-purpose compute compatibility.

## Why the vCPU does not move to WebGPU

- WGSL has concrete `i32`, `u32`, `f32`, and optional `f16` scalar types, but no native 64-bit integer type.
  ARM64 integer operations would become multiword shader operations.
- WGSL atomics are limited to `atomic<i32>` and `atomic<u32>` and do not model ARM acquire/release, exclusives, or LSE semantics directly.
- Workgroups have no global execution barrier or guaranteed launch order. A persistent shader cannot safely act as the VM scheduler.
- WebGPU buffers cannot alias WebBoxVM's sparse 4 KiB guest-memory pages.
  Dispatch therefore adds explicit upload and, for CPU-visible results, asynchronous readback.
- The current workload has only a few sequential vCPUs. Fetch/decode/MMU/MMIO
  control flow has too little independent work to amortize GPU submission and synchronization.

Moving the vCPU loop to WebGPU would make the correctness boundary harder and is unlikely to improve wall-clock performance.
The existing Wasm64 basic-block JIT remains the appropriate CPU accelerator.

## VirtIO-GPU slice

The device implements the Linux-facing 2D lifecycle defined by VirtIO:

1. `GET_DISPLAY_INFO` reports scanout 0 at 1024x768.
2. `RESOURCE_CREATE_2D` creates a bounded host-private resource.
3. `RESOURCE_ATTACH_BACKING` records bounded guest scatter/gather memory.
4. `TRANSFER_TO_HOST_2D` copies only the requested rectangle into the host resource and normalizes supported formats to BGRA8.
5. `SET_SCANOUT` selects the resource and source rectangle.
6. `RESOURCE_FLUSH` coalesces visible damage.
7. The Wasm host adapter exports a compact dirty-rectangle packet; the browser keeps a persistent `bgra8unorm` texture and uploads only that rectangle.

The binary host packet is deliberately not a guest ABI; it has a versioned 32-byte little-endian header:

```text
"WBGF", version, scanout_width, scanout_height, x, y, width, height
```

Tightly packed BGRA8 pixels follow the header.

## Experimental 3D slice

WebBoxVM advertises `VIRTIO_GPU_F_VIRGL`, `VIRTIO_GPU_F_RESOURCE_BLOB`, `VIRTIO_GPU_F_BLOB_ALIGNMENT`, and `VIRTIO_GPU_F_CONTEXT_INIT`. It exposes a bounded host-visible aperture, guest/default blob profiles, and a private context-local allocation ledger, separately documented in [Venus foundations](venus-foundations.md).
The device now separately exposes a deliberately narrow standard VirGL capset-1 clear, solid-triangle, generic vertex-color, fragment-constant texture, texture-times-vertex-color, and fixed sampled-texture path, documented in [VirGL compatibility](virgl-compatibility.md).
Capset ID 7 is deliberately private and unregistered, its data starts with
`WBG3`, and generic Mesa must not select or interpret it.

A small guest library selects capset 7 with `DRM_VIRTGPU_CONTEXT_INIT` and submits a bounded frame through `DRM_VIRTGPU_EXECBUFFER`.
The opaque `SUBMIT_3D` payload contains:

```text
WBG3, version, indexed-draw opcode, sequence,
canvas width/height, vertex count, index count, clear RGBA,
4x4 MVP matrix,
xyz+rgba vertices,
u16 triangle indices
```

The browser validates the packet again, reuses the 2D presenter's WebGPU adapter/device/canvas, uploads vertex/index/uniform buffers, enables a depth attachment, and issues `drawIndexed`.
This is enough for a guest-originated, orthographically rotated, depth-tested
cube while keeping the parser and resource budget auditable.

Current hard bounds are a 256-entry queue, one fixed 1024x768 scanout, at most 64 2D resources, 64 MiB per resource and 128 MiB total, 16,384 backing entries per resource, and 64 contexts.
WBG3 additionally allows at most 16 pending submissions and 2 MiB of pending packet bytes, an 8192x8192 canvas, 4,096 vertices, and 12,288 `u16` indices.

Every accepted WBG3 `SUBMIT_3D` descriptor stays out of the used ring until its host acknowledgment.
In the browser, success follows `GPUQueue.onSubmittedWorkDone()`; the VM worker then completes the saved virtqueue response and raises the GPU interrupt.
Unsupported or failed draws receive a negative acknowledgment and complete with an error.

The guest demo requests no fence or sync objects, so its `EXECBUFFER` ioctl and PASS marker are asynchronous submission evidence and can precede the browser acknowledgment.
Browser draw telemetry and a pixel check remain necessary to prove WebGPU execution and visible output.
A completed WBG3 v1 draw owns full-canvas presentation until reset, device loss, or a later failed draw; later WBGF packets update the CPU shadow without overwriting active 3D.
There is no v1 guest-release or mixed-composition opcode.

Full VirGL would additionally require a Gallium state machine and TGSI-to-WGSL translator.
Venus still requires host-3D/host-visible synchronization and host Vulkan external-memory primitives that WebGPU does not expose; neither name is used for the private capset.

## Known transport limitations

- Both queues are exposed, but only control-queue commands are implemented; `UPDATE_CURSOR` and `MOVE_CURSOR` on the cursor queue return an error.
- Feature pages retain the driver's 64-bit selection. `FEATURES_OK` clears for an unsupported mask or a missing `VERSION_1`; resource-blob creation is gated on that accepted feature.
- Guest/default blobs, host-only mapped staging, and a private `WBL1` allocation ledger coexist with WBG3 plus narrow capset-1 triangles. There is no real Venus renderer object, external-memory sharing, general VirGL/Venus shader/state, or compute API.

## Invariants

- A physical address belongs to at most one MMIO device.
- Resource ID zero is never a live resource; live IDs are unique.
- Width, height, byte counts, descriptor chains, and backing-entry totals are
  checked before allocation or DMA.
- Queue traversal is bounded by the configured device queue size, including malformed
  or cyclic chains.
- Guest memory remains authoritative. WebGPU never receives the general 1 GiB
  RAM image.
- The browser owns WebGPU objects. The device module does not depend on Wasm,
  JavaScript, canvas, or browser state.
- Scanout damage is coalesced between host polls. Emitted frame packets are
  applied in order; VM replacement/reset rejects packets from an older VM.
- VM replacement or a guest VirtIO-GPU reset changes a host-observed generation
  and invalidates queued frames; stale work cannot retain or reclaim the canvas.
- If initial WebGPU setup fails before the canvas is claimed, 2D uses Canvas2D; neither 3D path has a Canvas2D fallback.
  After WebGPU claims the canvas, it cannot switch to Canvas2D. Device loss preserves the CPU shadow framebuffer and attempts reconstruction.
  If reconstruction fails, the VM continues and later frames trigger retries while presentation remains unavailable.

## Performance model

For a damaged rectangle of width `w` and height `h`, host transfer cost is
`O(w*h)` bytes rather than `O(scanout_width*scanout_height)`. Resource lookup
is expected `O(1)` for `r` live resources; virtqueue traversal is `O(d)` for
`d` descriptors and is bounded by the queue size. The browser keeps texture
allocation out of the steady-state frame loop.

The optimization is accepted only if an A/B browser benchmark shows a material
end-to-end improvement over the Canvas2D fallback without worsening p95 VM input
latency. Shader or queue timing alone is insufficient.

## Validation boundary

The confirmed native installed-disk smoke observed both DRM nodes, seven full
WBGF frames, the exact WBG3 packet and host completion, and no `0x1205` device
response. It proves Linux-driver transport and native scanout, but supplies its
own success acknowledgment and does not execute the browser WebGPU pipeline.
An isolated Chrome run of fresh serial Wasm proved both DRM nodes, three completed guest draws, zero errors, and 160,358 non-clear readback pixels on a non-fallback Apple/Metal adapter.
A 30-pair 1024x768 microbenchmark measured 1.210 ms median WebGPU submit versus 2.157 ms Canvas2D, but WebGPU queue completion was 2.795 ms; it excludes Rust/worker and input latency, so it is not end-to-end acceptance.
The live guest cube remained visible across later 2D polls under the explicit WBG3 ownership rule.

Repository validation should retain all of the following:

- Rust tests for device identity, feature/config registers, queue bounds,
  command responses, resource lifecycle, DMA, used-ring advancement, IRQ,
  damage coalescing, reset, MMIO routing, and DTB discovery.
- Browser tests for packet validation, dirty-row padding, persistent texture
  updates, capability failure, stale-frame rejection, fallback, and device
  loss.
- A fresh serial and threaded Wasm build, not only native/Node tests.
- A real isolated-browser run that proves a hardware WebGPU adapter was chosen,
  Linux bound `virtio_gpu`, `/dev/dri/card0` exists, and a deterministic guest
  pattern reached the canvas with a verified pixel checksum or screenshot.
- Repeated software-versus-WebGPU measurements for the same guest frames,
  including upload and presentation time.

A successful browser run of the private-capset cube would prove only this
bounded guest-originated WebGPU draw. It would not establish Mesa, OpenGL,
Vulkan, or compute compatibility; those require their own standard guest API,
complete translated command path, and end-to-end A/B result.

## Sources

- VirtIO [1.4 GPU specification](https://docs.oasis-open.org/virtio/virtio/v1.4/virtio-v1.4.pdf) and [1.2 GPU section](https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html#x1-2900007)
- W3C [WebGPU](https://www.w3.org/TR/webgpu/) and WGSL [scalar/atomic types](https://www.w3.org/TR/WGSL/#plain-types-section) and [compute workgroups](https://www.w3.org/TR/WGSL/#compute-shader-stage)
- Linux 6.12 VirtIO-GPU [userspace ABI](https://github.com/torvalds/linux/blob/v6.12/include/uapi/drm/virtgpu_drm.h) and [wire ABI](https://github.com/torvalds/linux/blob/v6.12/include/uapi/linux/virtio_gpu.h)
- Mesa [VirGL](https://docs.mesa3d.org/drivers/virgl.html) and [Venus](https://docs.mesa3d.org/drivers/venus.html) architecture
