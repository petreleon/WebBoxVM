# VirGL compatibility track

## Current verified capset-1 slice

WebBoxVM exposes standard VirtIO-GPU capset ID 1 before its private WBG3
capset ID 7. It reports capset 1, version 1, with a 308-byte
`virgl_caps_v1`-layout response. The response advertises only the formats,
primitive, and limits exercised by this implementation.

This is a guest-visible VirGL wire-protocol vertical slice, not a claim that
Mesa, OpenGL, or arbitrary VirGL workloads work. It supports a full-scanout
clear, one exact standard source-over blend state, and deliberately bounded
solid-color or nearest-sampled-texture triangle paths with one viewport/scissor.

| Standard boundary | Current behavior | Deliberate limit |
| --- | --- | --- |
| Capset discovery | `GET_CAPSET_INFO` index 0 reports ID 1/version 1/308 bytes | No capset 2 |
| Texture resources | Packed 2D render targets and one B8G8R8A8 sampleable form | No mip levels, arrays, blobs, or multisampling |
| Buffer resources | R8 raw storage and R32G32B32A32_FLOAT vertex buffers | R8 is not a renderable vertex format |
| Context lifecycle | capset-1 create, destroy, attach, and detach are tracked | No shared contexts or fences |
| Resource transfer/copy | 72-byte transfers and one bounded copy per submit | No explicit strides, blit, format conversion, or scanout copy |
| VirGL stream | Surface/framebuffer, canonical TGSI, vertex/sampler state, blend/rasterizer, viewport/scissor, clear, and `DRAW_VBO` | No arbitrary TGSI or fixed-function state |
| Presentation | Clear, solid triangle, or fixed nearest-texture triangle through WebGPU | No multi-draw composition, depth, arbitrary blending, or filtering |
| Completion | CPU pixels change only after browser queue completion | Lost or stale context reports an error |

## Advertised and accepted shapes

The capset render-format mask contains only B8G8R8A8, B8G8R8X8, A8R8G8B8,
and X8R8G8B8. Its vertex-buffer mask contains only
`VIRGL_FORMAT_R32G32_FLOAT` (29) and `VIRGL_FORMAT_R32G32B32A32_FLOAT` (31),
and its primitive mask contains only `PIPE_PRIM_TRIANGLES` (bit 4). No GLSL
feature level is advertised.

`RESOURCE_CREATE_3D` accepts only these exact resource forms:

- texture-2D target; one advertised packed color format and render-target bind,
  or B8G8R8A8 with sampler-view or render-and-sampler bind; depth/array one,
  level zero, zero or one sample, and no flags;
- `PIPE_BUFFER` target; R8 or R32G32B32A32_FLOAT; exactly vertex-buffer bind;
  width in bytes; height/depth/array one; level/sample zero; no flags.

R8 buffers remain useful for standard transfer, readback, and byte-copy tests.
A solid draw requires an attached format-31 VBO at stride 16. A textured draw
uses stride 24 with type-5 `VERTEX_ELEMENTS`: format 31 position at offset zero
and format 29 UV at offset 16; both use divisor zero and VBO slot zero.

Type-4 shader objects accept only canonical NUL-terminated TGSI text: the
solid passthrough/constant-RGBA pair, or the textured passthrough/one 2D
`TEX` pair. The latter has one generic UV input, one sampler/view, and one
color output. Initial `OFFSET` is the total text-byte count; a continuation
has its high bit set and names the exact next byte offset. One bounded 4 KiB
source per vertex/fragment stage may be in flight. Chunks must retain handle,
stage, and token count; parser failure leaves the cloned context unchanged.
The declared token capacity plus virglrenderer’s translation slack must fit
the recognized TGSI. Stream output, unknown stages, and unrecognized text fail.
Binding zero unbinds, and destroying a bound shader clears its stage.

Type-1 `VIRGL_OBJECT_BLEND` accepts one exact 11-word `pipe_blend_state`:
blend enabled; an RGBA color mask; RGB `ADD, SRC_ALPHA, INV_SRC_ALPHA`; and
alpha `ADD, ONE, INV_SRC_ALPHA`. A draw requires that object to be bound.
Binding zero unbinds it; every other equation, factor, mask, and independent
blend configuration is rejected.

Type-2 `VIRGL_OBJECT_RASTERIZER` accepts only the normal `DEPTH_CLIP`,
`HALF_PIXEL_CENTER`, and `BOTTOM_EDGE_RULE` bits, with or without `SCISSOR`,
plus exact unit point and line sizes. Binding zero unbinds it. Command 4
`SET_VIEWPORT_STATE` accepts one slot-zero `(scale_xyz, translate_xyz)` state;
command 15 `SET_SCISSOR_STATE` accepts one slot-zero packed lower-left
min/max rectangle. A draw requires the blend object, rasterizer, and viewport;
when the rasterizer has `SCISSOR`, it also requires the nonempty scissor.

Type-6 `VIRGL_OBJECT_SAMPLER_VIEW` accepts one attached B8G8R8A8 sampled
resource at level/layer zero with identity swizzle `0x688`. Type-7
`VIRGL_OBJECT_SAMPLER_STATE` accepts only the nine-word `0x1092` nearest,
clamp-to-edge state. Commands 10 `SET_SAMPLER_VIEWS` and 18
`BIND_SAMPLER_STATES` bind their sole handles at fragment stage 1, slot zero.
Type 8, rather than type 7, is the standard surface object type.

## Clear-plus-draw execution

The guest stream uses ordinary VirGL headers, object types 1, 2, 4, 5, 6, 7,
and 8; `SET_FRAMEBUFFER_STATE`, `SET_VIEWPORT_STATE`, `SET_SCISSOR_STATE`,
generic `CLEAR` or `CLEAR_SURFACE`, `SET_VERTEX_BUFFERS`, command 29
`BIND_SHADER`, command 10 sampler-view binding, command 18 sampler-state
binding, and command 8 `DRAW_VBO`. Parsing is bounded to 64 KiB; all context
mutations occur on a clone and commit only after the complete stream validates.

The single accepted draw has the standard 12-word `DRAW_VBO` payload with a
non-indexed count of three, one instance, `PIPE_PRIM_TRIANGLES`, zero bias,
zero start instance, no primitive restart, and no stream-output count. The
non-indexed restart and min/max hint fields are accepted but do not influence
the bounded renderer. It must follow one clear in the same submission and use
the same full current scanout framebuffer target. Clear/copy and draw/copy
mixtures, a second clear, or a second draw fail transactionally.

At draw validation Rust snapshots exactly three positions from the attached
VBO. They must be finite, have `x`, `y`, and `z` in `[-1, 1]`, `w == 1`, and
form a nondegenerate triangle. The texture route additionally snapshots three
finite UVs in `[-8, 8]` and the attached B8G8R8A8 source, limited to 64×64;
feedback into the target is rejected. Later buffer, texture, or state mutation
cannot alter queued browser work. Solid color and sampled texels use the
required source-over blend object.

After validation Rust sends a private `VGD1` envelope to the browser. `VGD1`
is not a guest ABI or VirGL command. Schema 2 is 144 bytes: its original
sequence, canvas size, colors, 48 vertex bytes, viewport, and optional
canonical top-origin scissor. Schema 3 appends 72 position/UV bytes, a 2D
texture size, and exact raw BGRA texels. The browser retains schema-1 parsing
only for old packets, independently validates schemas 2/3, converts VirGL `z`
from `[-w,w]` to WebGPU's `[0,w]`, flips `v` to raw top-origin storage, uses a
nearest clamp sampler, applies equivalent viewport/scissor, and waits for
`GPUQueue.onSubmittedWorkDone()`.

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
Gallium/TGSI, shader compilation, clipping, indexed or instanced draws,
multiple vertex attributes beyond the fixed position/UV layout, arbitrary
sampling/filtering or blending, depth/stencil, multi-target rendering, general
readback, or a broad VirGL renderer.

It also does not establish Vulkan or Venus. Venus needs blob resources,
external-memory semantics, synchronization, and context initialization support
that this capset deliberately does not advertise.

## Validation retained in the repository

Rust tests prove the capset bits, transactional no-clear rejection, exact
source-over and sampler setup, rasterizer unbind rejection, schema-2/3 `VGD1`
payloads, texture snapshot isolation, viewport/scissor bounds, deferred
acknowledgment, CPU clipped source-over raster results, and `WBGF` damage.
Browser tests prove private-envelope framing, malformed state rejection, exact
WebGPU blend/sampler descriptors, padded BGRA upload, viewport/scissor calls,
cached pipelines, no depth texture, `draw(3)`, and queue-gated completion.

`scripts/virgl_guest_transport_smoke.sh` separately proves native Linux
VirtIO-GPU/DRM/KMS transport for capset discovery, R8 buffer transfer/copy,
color transfer/readback, and the standard clear/fence path. It then creates
the exact R32G32B32A32 VBO, canonical TGSI state, exact type-1 source-over
blend and type-2 scissor-rasterizer objects, viewport/scissor state, and
`DRAW_VBO` stream; validates its schema-2 `VGD1` envelope; resolves the
deferred fence; and reads the blended `143,160,48,255` center plus the clear
outside-scissor pixel back through the Linux driver. It then creates an attached
B8G8R8A8 sampler-view texture and 24-byte position/UV VBO, validates schema-3,
completes it, and reads exact BGRA `10,20,30,255` at the center. It does not
claim native Mesa, a native OpenGL context, or browser WebGPU execution from
that harness.

## Next compatibility milestones

1. Expand only after proving additional resource formats, sampler slots, and
   texture-coordinate behavior with the same native/CPU/WebGPU agreement.
2. Design blob, external-memory, and synchronization contracts before any
   Venus capset or Vulkan claim.

## Sources

- [VirtIO GPU device specification](https://docs.oasis-open.org/virtio/virtio/v1.3/virtio-v1.3.pdf)
- [Linux VirtIO-GPU wire UAPI](https://github.com/torvalds/linux/blob/master/include/uapi/linux/virtio_gpu.h)
- [VirGL hardware formats and bindings](https://android.googlesource.com/platform/external/virglrenderer/+/68429e8e1106d0861d9f9f180583bd8381b8bf96/src/virgl_hw.h)
- [VirGL protocol commands](https://android.googlesource.com/platform/external/virglrenderer/+/056b3873e41c015249499dbf9f761c8e9a78b720/src/virgl_protocol.h)
- [Mesa VirGL encoder](https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/gallium/drivers/virgl/virgl_encode.c) and [VirGL renderer shader decoder](https://gitlab.freedesktop.org/virgl/virglrenderer/-/blob/main/src/vrend/vrend_renderer.c)
- [Mesa blend-state definitions](https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/util/blend.h)
- [Mesa VirGL architecture](https://docs.mesa3d.org/drivers/virgl.html)
- [Mesa Venus architecture](https://docs.mesa3d.org/drivers/venus.html)
