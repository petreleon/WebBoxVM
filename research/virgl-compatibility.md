# VirGL compatibility track

## Current verified capset-1 slice

WebBoxVM now exposes standard VirtIO-GPU capset ID 1 before its private WBG3
capset ID 7. It reports capset 1, version 1, with a 308-byte
`virgl_caps_v1`-layout response. The response intentionally advertises only
the format and limits exercised by this implementation.

This is a real, guest-visible VirGL wire-protocol vertical slice, but it is not
a claim that Mesa, OpenGL, or arbitrary VirGL workloads work. Its only rendered
operation is a full-scanout color clear; it also has one bounded off-screen copy
operation for resource data flow.

| Standard boundary | Current behavior | Deliberate limit |
| --- | --- | --- |
| Capset discovery | `GET_CAPSET_INFO` index 0 reports ID 1/version 1/308 bytes | No capset 2 |
| Resource creation | `RESOURCE_CREATE_3D` accepts B8G8R8A8, B8G8R8X8, A8R8G8B8, and X8R8G8B8 2D render targets | One target, one level, no multisampling |
| Context lifecycle | capset-1 create, destroy, attach, and detach are tracked | No shared contexts or fences |
| Resource transfer | `TRANSFER_TO_HOST_3D` uploads and `TRANSFER_FROM_HOST_3D` returns a classic capset-1 2D BGRA resource | No blobs, arrays, mip levels, or explicit strides |
| Resource copy | `RESOURCE_COPY_REGION` copies one rectangle between attached off-screen, same-format 2D resources | No blit, format conversion, batching, or scanout copy |
| VirGL stream | surface create/destroy, framebuffer binding, `CLEAR`, and `CLEAR_SURFACE` are decoded | No shaders, state, or draws |
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
- VirGL B8G8R8A8, B8G8R8X8, A8R8G8B8, or X8R8G8B8 format;
- render-target binding;
- depth and array size of one;
- level zero, zero or one sample, and no resource flags.

`TRANSFER_TO_HOST_3D` and `TRANSFER_FROM_HOST_3D` use the exact same 72-byte
standard wire structure. In this slice each requires a live capset-1 context
and accepts only a zero-mip, `z=0`, `d=1` box with zero stride and layer stride.
Its offset names the first source or destination pixel in the attached guest
backing; rows use the resource's native packed-32-bit stride, matching the
classic-resource path used by Linux. Reverse transfer validates every
scatter-backed destination before writing it, so an invalid range cannot cause a
partial guest-memory update. Neither direction implies a scanout flush or browser
submission.

The clear path's resource must additionally be the exact current VirtIO-GPU
scanout resource. `CLEAR_SURFACE` must use that resource's complete scanout
rectangle, a known surface, and finite RGBA values in `[0, 1]`.

`RESOURCE_COPY_REGION` is command 17 in the standard VirGL stream. This slice
accepts one copy per submission only when both resources are attached to the
capset-1 context, are not the active scanout, have identical formats, and use
level zero with `z=0` and depth one. The parser validates the complete stream
before copying, snapshots the source rectangle before writing, and rejects
clear/copy mixtures. That prevents malformed trailing records from mutating a
resource and gives defined self-overlap behavior without implying browser-side
presentation or a general renderer command queue.

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
  depth/stencil, draw calls, multi-format readback, or general resource-transfer work;
- capset 2 or capability coverage beyond the conservative v1 response;
- Vulkan, Venus, blob resources, external memory, or synchronization support.

The private capset-7 WBG3 cube route remains separate and is documented in
[WebGPU acceleration](webgpu-acceleration.md). Browser diagnostics label the
two routes differently, and the capset-1 browser helper does not reuse WBG3's
geometry pipeline, buffers, or textures.

## Validation retained in the repository

Rust tests prove the exact capset response, 72-byte bidirectional transfer frames and their
context, backing, and layout rejection paths without mutation, same-resource
overlap and transactional rejection for command-17 copy, generic framebuffer
and `CLEAR_SURFACE` resource-to-scanout lifecycles, and deferred mutation until
a successful browser acknowledgment. Browser tests prove that a `VGC1` clear
produces one WebGPU submission, uses the requested canvas size and clear color,
returns success after queue completion, and leaves WBG3 rendering objects
unused.

`scripts/virgl_guest_transport_smoke.sh` adds a native Linux guest proof. It
builds `guest/virgl-clear-demo`, loads the real `virtio_gpu` driver in the
installed Debian fixture, obtains capset 1 through `DRM_IOCTL_VIRTGPU_GET_CAPS`,
creates a B8G8R8X8 capset-1 resource, maps its backing with
`DRM_IOCTL_VIRTGPU_MAP`, and writes two distinctive BGRX pixels before issuing
`DRM_IOCTL_VIRTGPU_TRANSFER_TO_HOST`. It creates two four-pixel off-screen
resources, uploads two pixels to one, submits ordinary command-17
`RESOURCE_COPY_REGION` through `EXECBUFFER`, waits for its destination, and
checks the copied bytes through `DRM_IOCTL_VIRTGPU_TRANSFER_FROM_HOST`. It then
binds the scanout resource to the legacy KMS primary plane, submits the normal
surface/framebuffer/`CLEAR` stream, and waits for the resource fence. The
harness first verifies the exact scanout upload in a full `WBGF`, then validates
`VGC1`, captures the all-BGRA `[191, 128, 64, 255]` `WBGF` immediately at
positive completion, and accepts the guest marker only after both guest-side
copy and clear readback checks succeed.

That native proof validates Linux DRM/KMS transport and post-ack CPU-side
readback. It intentionally uses the native completion adapter, not a browser
WebGPU device, so it complements rather than replaces the browser queue tests.

## Next compatibility milestones

The guest-side capset-1/KMS probe prerequisite is complete. Next:

1. Add enough resource variants, surface, state, shader, and draw semantics
   for a deliberately small Mesa/OpenGL acceptance test.
2. Expand capability reporting only when each advertised feature has matching
   parser, resource-lifetime, browser, and negative-path coverage.
3. Investigate Venus only after blob-resource, external-memory, and sync
   semantics have a sound browser-compatible design; do not advertise it early.

## Sources

- [VirtIO GPU device specification](https://docs.oasis-open.org/virtio/virtio/v1.3/virtio-v1.3.pdf)
- [Linux VirtIO-GPU wire UAPI](https://github.com/torvalds/linux/blob/master/include/uapi/linux/virtio_gpu.h)
- [Linux VirtIO-GPU DRM UAPI](https://github.com/torvalds/linux/blob/master/include/uapi/drm/virtgpu_drm.h) and [KMS UAPI](https://github.com/torvalds/linux/blob/master/include/uapi/drm/drm_mode.h)
- [QEMU VirGL transfer implementation](https://github.com/qemu/qemu/blob/master/hw/display/virtio-gpu-virgl.c)
- [Mesa VirGL clear context](https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/gallium/drivers/virgl/virgl_context.c) and [encoder](https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/gallium/drivers/virgl/virgl_encode.c)
- [Mesa VirGL architecture](https://docs.mesa3d.org/drivers/virgl.html)
- [Mesa Venus architecture](https://docs.mesa3d.org/drivers/venus.html)
