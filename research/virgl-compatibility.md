# VirGL compatibility track

## Current verified capset-1 slice

WebBoxVM now exposes standard VirtIO-GPU capset ID 1 before its private WBG3
capset ID 7. It reports capset 1, version 1, with a 308-byte
`virgl_caps_v1`-layout response. The response intentionally advertises only
the format and limits exercised by this implementation.

This is a real, guest-visible VirGL wire-protocol vertical slice, but it is not
a claim that Mesa, OpenGL, or arbitrary VirGL workloads work. Its only rendered
operation is a full-scanout color clear.

| Standard boundary | Current behavior | Deliberate limit |
| --- | --- | --- |
| Capset discovery | `GET_CAPSET_INFO` index 0 reports ID 1/version 1/308 bytes | No capset 2 |
| Resource creation | `RESOURCE_CREATE_3D` accepts a B8G8R8A8 2D render target | One target, one level, no multisampling |
| Context lifecycle | capset-1 create, destroy, attach, and detach are tracked | No shared contexts or fences |
| VirGL stream | surface create/destroy and `CLEAR_SURFACE` are decoded | No shaders, state, draws, or transfers |
| Presentation | a validated full current scanout clear becomes a WebGPU render-pass clear | No composition or sub-rectangle clear |
| Completion | Rust applies CPU-side pixels only after browser WebGPU completion | A lost/stale context reports an error |

The standard guest stream uses the ordinary VirGL command header, object type
7 (`SURFACE`), and command 62 (`CLEAR_SURFACE`). Parsing is bounded to 64 KiB,
each record is fully framed before use, and the projected context is committed
only after the whole stream validates. A malformed stream therefore cannot
partially create a surface or modify scanout pixels.

## Resource and completion model

The accepted `RESOURCE_CREATE_3D` shape is deliberately restrictive:

- texture-2D target;
- VirGL B8G8R8A8 format;
- render-target binding;
- depth and array size of one;
- level zero, zero or one sample, and no resource flags.

The resource must be attached to the capset-1 context and be the exact current
VirtIO-GPU scanout resource. `CLEAR_SURFACE` must use that resource's complete
scanout rectangle, a known surface, and finite RGBA values in `[0, 1]`.

After validation, Rust queues a private `VGC1` delivery envelope to the browser.
`VGC1` is not a guest ABI and is not a VirGL command: it exists only between the
Rust device and browser renderer. The browser emits a WebGPU render pass with
the requested clear color, waits for `GPUQueue.onSubmittedWorkDone()`, and then
acknowledges the VirtIO descriptor. Only a successful acknowledgment applies
the matching BGRA clear to the authoritative CPU-side resource and emits normal
scanout damage. Context generation and resource membership are rechecked at
that point, so stale browser work cannot alter a replacement context.

## What this does and does not establish

The slice establishes that the standard capset-1 discovery, 3D resource,
context attachment, and a narrowly decoded VirGL command can traverse the
guest-to-WebGPU path without relabeling private WBG3 packets as VirGL.

It does **not** establish any of the following:

- Mesa's VirGL driver can initialize or render;
- OpenGL contexts, Gallium state, TGSI, shaders, textures, buffers, blending,
  depth/stencil, draw calls, readback, or general resource transfer work;
- capset 2 or capability coverage beyond the conservative v1 response;
- Vulkan, Venus, blob resources, external memory, or synchronization support.

The private capset-7 WBG3 cube route remains separate and is documented in
[WebGPU acceleration](webgpu-acceleration.md). Browser diagnostics label the
two routes differently, and the capset-1 browser helper does not reuse WBG3's
geometry pipeline, buffers, or textures.

## Validation retained in the repository

Rust tests prove the exact capset response, a complete resource-to-scanout
clear lifecycle, deferred mutation until successful browser acknowledgment, and
rejection without mutation for malformed streams. Browser tests prove that a
`VGC1` clear produces one WebGPU submission, uses the requested canvas size and
clear color, returns success after queue completion, and leaves WBG3 rendering
objects unused.

These tests are host-level protocol evidence. A future guest proof must create
the standard 3D resource through the Linux `virtgpu` UAPI, attach it through an
ordinary `EXECBUFFER`, connect it to a KMS scanout, submit the surface clear,
and verify the browser result. That work is intentionally separate from this
first compatibility milestone.

## Next compatibility milestones

1. Add a guest-side capset-1/KMS probe with visible and readback verification.
2. Implement enough resource transfer, surface, state, shader, and draw
   semantics to support a deliberately small Mesa/OpenGL acceptance test.
3. Expand capability reporting only when each advertised feature has matching
   parser, resource-lifetime, browser, and negative-path coverage.
4. Investigate Venus only after blob-resource, external-memory, and sync
   semantics have a sound browser-compatible design; do not advertise it early.

## Sources

- [VirtIO GPU device specification](https://docs.oasis-open.org/virtio/virtio/v1.3/virtio-v1.3.pdf)
- [Linux VirtIO-GPU wire UAPI](https://github.com/torvalds/linux/blob/master/include/uapi/linux/virtio_gpu.h)
- [Mesa VirGL architecture](https://docs.mesa3d.org/drivers/virgl.html)
- [Mesa Venus architecture](https://docs.mesa3d.org/drivers/venus.html)
