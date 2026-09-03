# VirGL compatibility track

## Current verified capset-1 slice

WebBoxVM now exposes standard VirtIO-GPU capset ID 1 before its private WBG3
capset ID 7. It reports capset 1, version 1, with a 308-byte
`virgl_caps_v1`-layout response. The response intentionally advertises only
the format and limits exercised by this implementation.

This is a real, guest-visible VirGL wire-protocol vertical slice, but it is not
a claim that Mesa, OpenGL, or arbitrary VirGL workloads work. Its only rendered
operation is a full-scanout color clear; it also has bounded off-screen color
and raw vertex-buffer copy paths for resource data flow.

| Standard boundary | Current behavior | Deliberate limit |
| --- | --- | --- |
| Capset discovery | `GET_CAPSET_INFO` index 0 reports ID 1/version 1/308 bytes | No capset 2 |
| Resource creation | `RESOURCE_CREATE_3D` accepts four packed 2D color targets and an R8 `PIPE_BUFFER` with vertex binding | One bounded vertex-state shape; no fetch or draw |
| Context lifecycle | capset-1 create, destroy, attach, and detach are tracked | No shared contexts or fences |
| Resource transfer | 2D color and raw R8 vertex-buffer upload/readback use standard 72-byte transfers | No blobs, arrays, mip levels, or explicit strides |
| Resource copy | `RESOURCE_COPY_REGION` copies one 2D rectangle or raw vertex-buffer byte range | No blit, format conversion, batching, or scanout copy |
| VirGL stream | clear/surface state, one vertex-input chain, and canonical TGSI shader state are decoded | No arbitrary TGSI, fixed state, or draws |
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

One additional non-renderable shape is accepted as resource storage groundwork:

- `PIPE_BUFFER` target with R8 format and exactly `VERTEX_BUFFER` binding;
- width measured in bytes, height/depth/array size of one;
- level zero, zero samples, and no resource flags.

The capset vertex-format mask remains unset until a rendered feature exists.
Before that, the stream accepts only an empty unbind or one attached R8 buffer
at byte offset within the resource with stride one, plus one type-5
`VERTEX_ELEMENTS` object whose R8 element has zero source offset/divisor and
uses buffer slot zero. Detach clears the bound buffer and destroying the bound
object clears its selection. It cannot become a surface, framebuffer, clear
target, or color-copy operand.

Type-4 `CREATE_OBJECT` and command-29 `BIND_SHADER` now prepare a deliberately
canonical TGSI-text subset: vertex passthrough (`VERT`, one input, `POSITION`,
`MOV`, `END`) and fragment solid color (`FRAG`, `COLOR`, normalized finite
`FLT32`, `MOV`, `END`). Shader continuations, stream output, other stages, and
unrecognized text fail before the cloned context commits. Binding zero unbinds;
destroying a bound shader clears its stage. This is state groundwork, not a
claim that TGSI is compiled or rendered.

`TRANSFER_TO_HOST_3D` and `TRANSFER_FROM_HOST_3D` use the exact same 72-byte
standard wire structure. In this slice each requires a live capset-1 context
and accepts only a zero-mip, zero-stride, zero-layer-stride layout. For a color
target, the `z=0`, `d=1` box and offset name pixels in the attached backing; rows
use the native packed-32-bit stride. For the R8 buffer, `y=z=0`, `h=d=1`, and
the box's `x` and width instead name a nonempty byte range. Reverse transfer
validates every scatter-backed destination before writing it, so an invalid range
cannot cause a partial guest-memory update. Neither direction implies a scanout
flush or browser submission.

The clear path's resource must additionally be the exact current VirtIO-GPU
scanout resource. `CLEAR_SURFACE` must use that resource's complete scanout
rectangle, a known surface, and finite RGBA values in `[0, 1]`.

`RESOURCE_COPY_REGION` is command 17 in the standard VirGL stream. This slice
accepts one copy per submission only when both resources are attached to the
capset-1 context, are not the active scanout, have identical formats, and use
level zero with `z=0` and depth one. Color resources use a 2D rectangle; raw
buffers instead require `y=0`, height one, and treat source/destination `x` and
width as byte positions. Resource kinds cannot be mixed. The parser validates
the complete stream before copying, snapshots the source range before writing,
and rejects clear/copy mixtures. That prevents malformed trailing records from
mutating a resource and gives defined self-overlap behavior without implying
browser-side presentation or a general renderer command queue.

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
- OpenGL contexts, actual vertex fetch, arbitrary Gallium/TGSI shaders, textures, blending,
  depth/stencil, draw calls, multi-format readback, or general transfer work;
- capset 2 or capability coverage beyond the conservative v1 response;
- Vulkan, Venus, blob resources, external memory, or synchronization support.

The private capset-7 WBG3 cube route remains separate and is documented in
[WebGPU acceleration](webgpu-acceleration.md). Browser diagnostics label the
two routes differently, and the capset-1 browser helper does not reuse WBG3's
geometry pipeline, buffers, or textures.

## Validation retained in the repository

Rust tests prove the exact capset response, 72-byte bidirectional transfer frames
for color and byte-buffer resources with context, backing, and layout rejection
paths without mutation; byte-buffer/2D same-resource copy overlap; transactional
vertex-state rejection and detach/destroy lifetime cleanup; generic framebuffer
and `CLEAR_SURFACE` resource-to-scanout lifecycles; and deferred mutation until
a successful browser acknowledgment.
Browser tests prove that a `VGC1` clear produces one WebGPU submission, uses the
requested canvas size and clear color, returns success after queue completion,
and leaves WBG3 rendering objects unused.

`scripts/virgl_guest_transport_smoke.sh` adds a native Linux guest proof. It
builds `guest/virgl-clear-demo`, loads the real `virtio_gpu` driver in the
installed Debian fixture, obtains capset 1 through `DRM_IOCTL_VIRTGPU_GET_CAPS`,
creates source and destination R8 `PIPE_BUFFER` vertex-buffer resources,
performs byte-range upload, submits type-5 vertex-element create/bind plus
`SET_VERTEX_BUFFERS`, then performs command-17 copy and readback at different
backing offsets through real Linux DRM ioctls. It then creates a B8G8R8X8
capset-1 resource, maps its backing with
`DRM_IOCTL_VIRTGPU_MAP`, and writes two distinctive BGRX pixels before issuing
`DRM_IOCTL_VIRTGPU_TRANSFER_TO_HOST`. It creates two four-pixel off-screen
resources, uploads two pixels to one, submits ordinary command-17
`RESOURCE_COPY_REGION` through `EXECBUFFER`, waits for its destination, and
checks the copied bytes through `DRM_IOCTL_VIRTGPU_TRANSFER_FROM_HOST`. It then
binds the scanout resource to the legacy KMS primary plane, submits the normal
surface/framebuffer/`CLEAR` stream, and waits for the resource fence. The
harness first verifies the exact scanout upload in a full `WBGF`, then validates
`VGC1`, captures the all-BGRA `[191, 128, 64, 255]` `WBGF` immediately at
positive completion, and accepts the guest marker only after guest-side buffer,
copy, and clear readback checks succeed.

That native proof validates Linux DRM/KMS transport and post-ack CPU-side
readback. It intentionally uses the native completion adapter, not a browser
WebGPU device, so it complements rather than replaces the browser queue tests.

## Next compatibility milestones

The guest-side capset-1/KMS probe prerequisite is complete. Next:

1. Add one bounded draw path that consumes the verified vertex-input and
   canonical shader state, then add fixed state with renderer coverage.
2. Expand capability reporting only when each advertised feature has matching
   parser, resource-lifetime, browser, and negative-path coverage.
3. Investigate Venus only after blob-resource, external-memory, and sync
   semantics have a sound browser-compatible design; do not advertise it early.

## Sources

- [VirtIO GPU device specification](https://docs.oasis-open.org/virtio/virtio/v1.3/virtio-v1.3.pdf)
- [Linux VirtIO-GPU wire UAPI](https://github.com/torvalds/linux/blob/master/include/uapi/linux/virtio_gpu.h)
- [Linux VirtIO-GPU DRM UAPI](https://github.com/torvalds/linux/blob/master/include/uapi/drm/virtgpu_drm.h) and [KMS UAPI](https://github.com/torvalds/linux/blob/master/include/uapi/drm/drm_mode.h)
- [VirGL hardware formats and bindings](https://android.googlesource.com/platform/external/virglrenderer/+/68429e8e1106d0861d9f9f180583bd8381b8bf96/src/virgl_hw.h) and [protocol commands](https://android.googlesource.com/platform/external/virglrenderer/+/056b3873e41c015249499dbf9f761c8e9a78b720/src/virgl_protocol.h)
- [QEMU VirGL transfer implementation](https://github.com/qemu/qemu/blob/master/hw/display/virtio-gpu-virgl.c)
- [Mesa VirGL clear context](https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/gallium/drivers/virgl/virgl_context.c) and [encoder](https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/gallium/drivers/virgl/virgl_encode.c)
- [Mesa VirGL architecture](https://docs.mesa3d.org/drivers/virgl.html)
- [Mesa Venus architecture](https://docs.mesa3d.org/drivers/venus.html)
