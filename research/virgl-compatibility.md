# VirGL compatibility track

## Current verified capset-1 slice

WebBoxVM exposes standard VirtIO-GPU capset ID 1 before its private WBG3
capset ID 7. It reports capset 1, version 1, with a 308-byte
`virgl_caps_v1`-layout response. The response advertises only the formats,
primitive, and limits exercised by this implementation.

This is a guest-visible VirGL wire-protocol vertical slice, not a claim that
Mesa, OpenGL, or arbitrary VirGL workloads work. It supports a full-scanout
clear, one exact standard source-over blend state, and one deliberately bounded
clear-plus-triangle draw path.

| Standard boundary | Current behavior | Deliberate limit |
| --- | --- | --- |
| Capset discovery | `GET_CAPSET_INFO` index 0 reports ID 1/version 1/308 bytes | No capset 2 |
| Texture resources | Four packed 2D color targets with render-target binding | No mip levels, arrays, blobs, or sampled textures |
| Buffer resources | R8 raw storage and R32G32B32A32_FLOAT vertex buffers | R8 is not a renderable vertex format |
| Context lifecycle | capset-1 create, destroy, attach, and detach are tracked | No shared contexts or fences |
| Resource transfer/copy | 72-byte transfers and one bounded copy per submit | No explicit strides, blit, format conversion, or scanout copy |
| VirGL stream | Surface/framebuffer, canonical TGSI, vertex state, exact type-1 source-over blend, clear, and one `DRAW_VBO` | No arbitrary TGSI or fixed-function state |
| Presentation | A clear or one source-over triangle is rendered through WebGPU | No multi-draw composition, depth, arbitrary blending, or textures |
| Completion | CPU pixels change only after browser queue completion | Lost or stale context reports an error |

## Advertised and accepted shapes

The capset render-format mask contains only B8G8R8A8, B8G8R8X8, A8R8G8B8,
and X8R8G8B8. Its vertex-buffer mask contains only
`VIRGL_FORMAT_R32G32B32A32_FLOAT` (format 31), and its primitive mask contains
only `PIPE_PRIM_TRIANGLES` (bit 4). No GLSL feature level is advertised.

`RESOURCE_CREATE_3D` accepts only these exact resource forms:

- texture-2D target; one advertised packed color format; render-target bind;
  depth/array one; level zero; zero or one sample; no flags;
- `PIPE_BUFFER` target; R8 or R32G32B32A32_FLOAT; exactly vertex-buffer bind;
  width in bytes; height/depth/array one; level/sample zero; no flags.

R8 buffers remain useful for standard transfer, readback, and byte-copy tests.
A rendered draw requires the R32G32B32A32_FLOAT form, one attached VBO at
stride 16 and an aligned byte offset, plus one type-5 `VERTEX_ELEMENTS` object
with source offset/divisor zero, VBO slot zero, and format 31.

Type-4 shader objects accept only canonical NUL-terminated TGSI text: a
passthrough vertex program (`VERT`, one input, `POSITION`, `MOV`, `END`) and a
solid normalized-finite RGBA fragment program (`FRAG`, `COLOR`, `IMM`, `MOV`,
`END`). Continuations, stream output, unknown stages, and unrecognized text
fail before the cloned context commits. Binding zero unbinds, and destroying a
bound shader clears its stage.

Type-1 `VIRGL_OBJECT_BLEND` accepts one exact 11-word `pipe_blend_state`:
blend enabled; an RGBA color mask; RGB `ADD, SRC_ALPHA, INV_SRC_ALPHA`; and
alpha `ADD, ONE, INV_SRC_ALPHA`. A draw requires that object to be bound.
Binding zero unbinds it; every other equation, factor, mask, and independent
blend configuration is rejected.

## Clear-plus-draw execution

The guest stream uses ordinary VirGL headers, object types 1, 4, 5, and 7,
`SET_FRAMEBUFFER_STATE`, generic `CLEAR` or `CLEAR_SURFACE`,
`SET_VERTEX_BUFFERS`, command 29 `BIND_SHADER`, and command 8 `DRAW_VBO`.
Parsing is bounded to 64 KiB; all context mutations occur on a clone and the
clone is committed only after the complete stream validates.

The single accepted draw has the standard 12-word `DRAW_VBO` payload with a
non-indexed count of three, one instance, `PIPE_PRIM_TRIANGLES`, zero bias,
zero start instance, no primitive restart, and no stream-output count. The
non-indexed restart and min/max hint fields are accepted but do not influence
the bounded renderer. It must follow one clear in the same submission and use
the same full current scanout framebuffer target. Clear/copy and draw/copy
mixtures, a second clear, or a second draw fail transactionally.

At draw validation Rust snapshots exactly three four-float vertices from the
attached VBO. Each must be finite, have `x` and `y` in `[-1, 1]`, `z` in
`[0, 1]`, `w == 1`, and form a nondegenerate triangle. It also snapshots the
fragment color. Later buffer mutations cannot alter the queued browser work.
The accepted fragment color is composited over the clear using the required
source-over blend object; its alpha is therefore semantically significant.

After validation Rust sends a private `VGD1` envelope to the browser. `VGD1`
is not a guest ABI or VirGL command; it carries the sequence, canvas size,
clear RGBA, fragment RGBA, and the 48 vertex bytes. The browser independently
validates the envelope, reuses a WebGPU `float32x4` triangle pipeline with the
matching source-over target blend, vertex buffer, uniform buffer, and bind
group for the device generation, then submits one clear-and-draw render pass
and waits for `GPUQueue.onSubmittedWorkDone()`.

Only a successful browser acknowledgment changes authoritative CPU state. Rust
rechecks the VirGL context generation, clears the target in its canonical BGRA
storage, rasterizes the same bounded triangle with source-over, and emits ordinary
scanout damage. Thus `TRANSFER_FROM_HOST_3D` sees a defined result after completion;
failed, stale, or unacknowledged browser work changes no guest pixels.

The clear-only route remains a smaller private `VGC1` envelope. It shares the
same deferred completion rule but allocates no pipeline, buffers, or textures.
Browser diagnostics distinguish `webgpu-virgl-capset1-clear` from
`webgpu-virgl-capset1-draw` and from private capset-7 WBG3 geometry.

## What this does not establish

This slice does not establish Mesa initialization, an OpenGL context, arbitrary
Gallium/TGSI, shader compilation, clipping, viewport/scissor state, indexed or
instanced draws, multiple attributes, arbitrary blending, depth/stencil,
textures, multi-target rendering, general readback, or a broad VirGL renderer.

It also does not establish Vulkan or Venus. Venus needs blob resources,
external-memory semantics, synchronization, and context initialization support
that this capset deliberately does not advertise.

## Validation retained in the repository

Rust tests prove the capset bits, transactional no-clear rejection, exact
source-over setup and unbind rejection, one `VGD1` payload, deferred
acknowledgment, BGRA clear result, CPU source-over raster result, and normal
`WBGF` damage. Browser tests prove private-envelope framing, malformed
color/position rejection, the exact WebGPU blend descriptor, one cached
pipeline, two buffers, no depth texture, a `draw(3)`, and completion only after
the queue resolves.

`scripts/virgl_guest_transport_smoke.sh` separately proves native Linux
VirtIO-GPU/DRM/KMS transport for capset discovery, R8 buffer transfer/copy,
color transfer/readback, and the standard clear/fence path. It then creates
the exact R32G32B32A32 VBO, canonical TGSI state, exact type-1 source-over
blend object, and `DRAW_VBO` stream; validates its `VGD1` envelope; resolves
the deferred fence; and reads the blended `143,160,48,255` center pixel back
through the Linux driver. It does not claim native
Mesa, a native OpenGL context, or a browser WebGPU execution from that harness.

## Next compatibility milestones

1. Add one bounded viewport/scissor state with matching capability reporting,
   browser rendering, CPU mirroring, and negative-path coverage.
2. Design blob, external-memory, and synchronization contracts before any
   Venus capset or Vulkan claim.

## Sources

- [VirtIO GPU device specification](https://docs.oasis-open.org/virtio/virtio/v1.3/virtio-v1.3.pdf)
- [Linux VirtIO-GPU wire UAPI](https://github.com/torvalds/linux/blob/master/include/uapi/linux/virtio_gpu.h)
- [VirGL hardware formats and bindings](https://android.googlesource.com/platform/external/virglrenderer/+/68429e8e1106d0861d9f9f180583bd8381b8bf96/src/virgl_hw.h)
- [VirGL protocol commands](https://android.googlesource.com/platform/external/virglrenderer/+/056b3873e41c015249499dbf9f761c8e9a78b720/src/virgl_protocol.h)
- [Mesa VirGL encoder](https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/gallium/drivers/virgl/virgl_encode.c)
- [Mesa blend-state definitions](https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/util/blend.h)
- [Mesa VirGL architecture](https://docs.mesa3d.org/drivers/virgl.html)
- [Mesa Venus architecture](https://docs.mesa3d.org/drivers/venus.html)
