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
| VirGL stream | surface create/destroy, framebuffer binding, `CLEAR`, and `CLEAR_SURFACE` are decoded | No shaders, state, draws, or transfers |
| Presentation | a validated full current scanout clear becomes a WebGPU render-pass clear | No composition or sub-rectangle clear |
| Completion | Rust applies CPU-side pixels only after browser WebGPU completion | A lost/stale context reports an error |

The standard guest stream uses the ordinary VirGL command header, object type
7 (`SURFACE`), `SET_FRAMEBUFFER_STATE`, generic `CLEAR`, and command 62
(`CLEAR_SURFACE`). Parsing is bounded to 64 KiB, each record is fully framed
before use, and the projected context is committed only after the whole stream
validates. A malformed stream therefore cannot partially create a surface,
leave a dangling framebuffer binding, or modify scanout pixels.

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

Rust tests prove the exact capset response, generic framebuffer and
`CLEAR_SURFACE` resource-to-scanout lifecycles, deferred mutation until a
successful browser acknowledgment, and rejection without mutation for malformed
or stale-surface streams. Browser tests prove that a `VGC1` clear produces one
WebGPU submission, uses the requested canvas size and clear color, returns
success after queue completion, and leaves WBG3 rendering objects unused.

`scripts/virgl_guest_transport_smoke.sh` adds a native Linux guest proof. It
builds `guest/virgl-clear-demo`, loads the real `virtio_gpu` driver in the
installed Debian fixture, obtains capset 1 through `DRM_IOCTL_VIRTGPU_GET_CAPS`,
creates a capset-1 resource, binds it to the legacy KMS primary plane, submits
the normal surface/framebuffer/`CLEAR` stream through `EXECBUFFER`, and waits
on the resource fence. The harness validates the exact `VGC1` packet before it
returns the positive host completion, then verifies that the resulting full
`WBGF` readback contains only BGRA `[191, 128, 64, 255]`; only then does it
accept the guest's fence-gated PASS marker.

That native proof validates Linux DRM/KMS transport and post-ack CPU-side
readback. It intentionally uses the native completion adapter, not a browser
WebGPU device, so it complements rather than replaces the browser queue tests.

## Next compatibility milestones

The guest-side capset-1/KMS probe prerequisite is complete. Next:

1. Implement enough resource transfer, surface, state, shader, and draw
   semantics to support a deliberately small Mesa/OpenGL acceptance test.
2. Expand capability reporting only when each advertised feature has matching
   parser, resource-lifetime, browser, and negative-path coverage.
3. Investigate Venus only after blob-resource, external-memory, and sync
   semantics have a sound browser-compatible design; do not advertise it early.

## Sources

- [VirtIO GPU device specification](https://docs.oasis-open.org/virtio/virtio/v1.3/virtio-v1.3.pdf)
- [Linux VirtIO-GPU wire UAPI](https://github.com/torvalds/linux/blob/master/include/uapi/linux/virtio_gpu.h)
- [Linux VirtIO-GPU DRM UAPI](https://github.com/torvalds/linux/blob/master/include/uapi/drm/virtgpu_drm.h) and [KMS UAPI](https://github.com/torvalds/linux/blob/master/include/uapi/drm/drm_mode.h)
- [Mesa VirGL clear context](https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/gallium/drivers/virgl/virgl_context.c) and [encoder](https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/gallium/drivers/virgl/virgl_encode.c)
- [Mesa VirGL architecture](https://docs.mesa3d.org/drivers/virgl.html)
- [Mesa Venus architecture](https://docs.mesa3d.org/drivers/venus.html)
