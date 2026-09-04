# VirGL compatibility track

## Current verified capset-1 slice

WebBoxVM exposes standard VirtIO-GPU capset ID 1 before its private WBG3
capset ID 7. It reports capset 1, version 1, with a 308-byte
`virgl_caps_v1`-layout response. The response advertises only the formats,
primitive, and limits exercised by this implementation.

This is a guest-visible VirGL wire-protocol vertical slice, not a claim that
Mesa, OpenGL, or arbitrary VirGL workloads work. It supports a full-scanout
clear, one exact standard source-over blend state, and deliberately bounded
solid-color; one generic per-vertex-RGBA triangle; nearest-clamp/repeat or
linear-clamp one-texture; texture-times-vertex-color; or two-texture paths with each sampler from that finite set and one viewport/scissor.

| Standard boundary | Current behavior | Deliberate limit |
| --- | --- | --- |
| Capset discovery | `GET_CAPSET_INFO` index 0 reports ID 1/version 1/308 bytes | No capset 2 |
| Texture resources | Packed 2D targets plus two B8G8R8A8 or R8G8B8A8 sampled resources | No mip levels, arrays, blobs, or multisampling |
| Buffer resources | R8 raw vertex/index storage and R32G32B32A32_FLOAT vertex buffers | R8 is not a renderable vertex format |
| Context lifecycle | capset-1 create, destroy, attach, and detach are tracked | No shared contexts or fences |
| Resource transfer/copy | 72-byte transfers and one bounded copy per submit | No explicit strides, blit, format conversion, or scanout copy |
| VirGL stream | Surface/framebuffer, canonical TGSI, vertex/index/sampler state, blend/rasterizer, viewport/scissor, clear, and `DRAW_VBO` | No arbitrary TGSI or fixed-function state |
| Presentation | Clear, solid, fixed vertex-color, texture-times-color, or one/two-texture sampled triangles through WebGPU | No multi-draw composition, depth, arbitrary blending, or sampler state |
| Completion | CPU pixels change only after browser queue completion | Lost or stale context reports an error |

## Advertised and accepted shapes

The capset render-format mask contains only B8G8R8A8, B8G8R8X8, A8R8G8B8,
and X8R8G8B8. Its sampler mask adds B8G8R8A8 (1) and R8G8B8A8 (67); its vertex-buffer mask contains only
`VIRGL_FORMAT_R32G32_FLOAT` (29) and `VIRGL_FORMAT_R32G32B32A32_FLOAT` (31),
and its primitive mask contains only `PIPE_PRIM_TRIANGLES` (bit 4). No GLSL
feature level is advertised.

`RESOURCE_CREATE_3D` accepts only these exact resource forms:

- texture-2D target; packed render target, B8G8R8A8 sampler-view/render-and-sampler,
  or R8G8B8A8 sampler-view-only; depth/array one,
  level zero, zero or one sample, and no flags;
- `PIPE_BUFFER` target; R32G32B32A32_FLOAT with exactly vertex-buffer bind, or
  R8 with exactly vertex- or index-buffer bind; width in bytes; height/depth/array one; level/sample zero; no flags.

R8 index storage binds through command 11 `SET_INDEX_BUFFER` at index size 2 or 4 and accepts aligned byte offsets.
Solid draws use an attached format-31 VBO at stride 16; textured draws use stride 24 with type-5 `VERTEX_ELEMENTS`:
format-31 position at offset zero and format-29 UV at offset 16. Vertex-color draws use stride 32 with
format-31 position and RGBA at offsets zero and 16; texture-color draws add format-29 UV at offset 32 with stride 40; all accepted elements have divisor zero in VBO slot zero.

Type-4 shader objects accept only canonical NUL-terminated TGSI text: the
solid passthrough/constant-RGBA pair; a generic RGBA-passthrough fragment pair; a two-generic texture-times-color pair;
a one-2D-`TEX` pair; or a two-2D-`TEX`, `MUL` pair. The latter has one generic UV input, two sampler/views, and one
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

Type-6 `VIRGL_OBJECT_SAMPLER_VIEW` accepts one attached B8G8R8A8 or R8G8B8A8 sampled
resource at level/layer zero with identity swizzle `0x688`; R8G8B8A8 normalizes on transfer before VGD1. Type-7
`VIRGL_OBJECT_SAMPLER_STATE` accepts exact nine-word `0x1092` nearest clamp, `0x1080` nearest
S/T-repeat (R remains clamp), or `0x3292` linear clamp; each accepted state can occupy either slot. Commands 10
`SET_SAMPLER_VIEWS` and 18 `BIND_SAMPLER_STATES` bind one or two variable-length handles at
fragment stage 1, starting at slot zero or one; the implementation caps the range at two
slots. Type 8, rather than type 7, is the standard surface object type.

## Clear-plus-draw execution

The guest stream uses ordinary VirGL headers, object types 1, 2, 4, 5, 6, 7,
and 8; `SET_FRAMEBUFFER_STATE`, `SET_VIEWPORT_STATE`, `SET_SCISSOR_STATE`,
generic `CLEAR` or `CLEAR_SURFACE`, `SET_VERTEX_BUFFERS`, command 11
`SET_INDEX_BUFFER`, command 29 `BIND_SHADER`, commands 10/18 sampler bindings,
and command 8 `DRAW_VBO`. Parsing is bounded to 64 KiB; all context mutations
occur on a clone and commit only after the complete stream validates.

The accepted standard 12-word `DRAW_VBO` has count three, one instance,
`PIPE_PRIM_TRIANGLES`, zero bias/start-instance/restart, and no stream-output count.
Its indexed field is zero for consecutive VBO records, or one for a command-11
binding that resolves exactly three little-endian u16 or u32 indices from `start`.
Restart and min/max hint fields are accepted but do not influence the bounded
renderer. Each draw follows one clear in the same submission against the current
full-scanout framebuffer; clear/copy mixing, repeat clear, and repeat draw fail transactionally.

At draw validation Rust snapshots exactly three positions from the attached VBO, directly or through bounded index-buffer lookups.
They must be finite, have `x`, `y`, and `z` in `[-1, 1]`, `w == 1`, and form a nondegenerate triangle.
Vertex-color and texture-color routes snapshot three finite normalized RGBA values; texture routes snapshot finite UVs in `[-8, 8]` and
one or two attached B8G8R8A8 or R8G8B8A8 sources, each limited to 64×64. Feedback into the target is rejected.
Schema 6 carries independent exact sampler state; schema 4 remains the legacy nearest-clamp pair. Later buffer, texture, or state mutation cannot alter queued browser work.
Solid color, interpolated vertex color, sampled texels, and texture-times-interpolated-color use the required source-over blend object.

After validation Rust sends a private `VGD1` envelope to the browser. `VGD1`
is not a guest ABI or VirGL command. Schema 2 is 144 bytes: its original
sequence, canvas size, colors, 48 vertex bytes, viewport, and optional
canonical top-origin scissor. Schema 3 appends 72 position/UV bytes, one 2D texture size, and canonical BGRA texels; schema 4 appends two sizes and paired
texels for legacy nearest-clamp multiplication; schema 5 carries exact one-texture state `0x1080` or `0x3292`, its size, and texels. Schema 6 carries two sampler words, sizes, and texels.
Schema 7 is exactly 192 bytes with three position/RGBA vertices plus viewport/scissor. Schema 8 is `228 + 4WH` bytes with three position/RGBA/UV vertices, viewport/scissor, one exact sampler, and one `W,H <= 64` BGRA texture. The browser retains schema-1 parsing only for old packets, independently validates schemas 2/3/4/5/6/7/8,
converts VirGL `z` from `[-w,w]` to WebGPU's `[0,w]`, flips `v` to raw top-origin storage, uses
a matching address/filter sampler, applies equivalent viewport/scissor, and waits for
`GPUQueue.onSubmittedWorkDone()`.

Only a successful browser acknowledgment changes authoritative CPU state. Rust
rechecks the VirGL context generation, clears the target in its canonical BGRA
storage, rasterizes the same bounded triangle with source-over, and emits ordinary
scanout damage. Thus `TRANSFER_FROM_HOST_3D` sees a defined result after completion;
failed, stale, or unacknowledged browser work changes no guest pixels.

The clear-only route remains a smaller private `VGC1` envelope with the same deferred completion rule but no pipeline, buffers, or textures.
Browser diagnostics distinguish clear, draw, texture, and dual-texture paths from private capset-7 WBG3 geometry.

## What this does not establish

This slice does not establish Mesa initialization, an OpenGL context, arbitrary
Gallium/TGSI, shader compilation, clipping, unbounded or instanced draws,
multiple vertex attributes beyond fixed position/UV, position/RGBA, or position/RGBA/UV layouts, arbitrary
sampling/filtering or blending, depth/stencil, multi-target rendering, general
readback, or a broad VirGL renderer.

It also does not establish Vulkan or Venus. Guest-only blobs and a bounded
host-visible staging map profile exist, but Venus needs guest-shadow blobs,
external-memory semantics, synchronization, and a matching capset that this
renderer does not advertise.

## Validation retained in the repository

Rust tests prove capset bits, transactional no-clear and malformed-index rejection,
exact source-over and sampler setup, rasterizer unbind rejection, schema-2/3/4/5/6/7/8 `VGD1`
payloads, normalized per-vertex RGBA interpolation and texture modulation, repeat-at-one, clamp-linear midpoint, and independent two-sampler CPU sampling, R8G8B8A8-to-BGRA normalization, nonzero-offset indexes, deferred
acknowledgment, clipped source-over raster results, viewport/scissor bounds, and `WBGF` damage.
Browser tests prove private-envelope framing, malformed sampler rejection, exact
independent WebGPU clamp/repeat/linear descriptors, fixed RGBA and RGBA/UV attributes, one/two padded BGRA uploads, viewport/scissor calls,
cached pipelines, no depth texture, `draw(3)`, and queue-gated completion.

`scripts/virgl_guest_transport_smoke.sh` separately proves native Linux
VirtIO-GPU/DRM/KMS transport for capset discovery, R8 buffer transfer/copy,
color transfer/readback, and the standard clear/fence path. It then creates
the exact R32G32B32A32 VBO plus an eight-byte R8 index buffer, canonical TGSI state,
exact type-1 source-over blend and type-2 scissor-rasterizer objects, viewport/scissor,
command-11 `SET_INDEX_BUFFER` at byte offset two, and indexed `DRAW_VBO`; it validates the schema-2
`VGD1` envelope with the guest's `[2,1,0]` reordered vertices, resolves the
deferred fence, and reads the blended `143,160,48,255` center plus the clear
outside-scissor pixel back through the Linux driver. It then creates two attached
R8G8B8A8 sampler-view textures and a 24-byte position/UV VBO. It validates schema-5's
transfer-normalized BGRA and first `0x1080` at `u == 1`, then `0x3292` at `u == .5`,
completes both, and reads `10,20,30,255` then `25,35,45,255` at the center. It then validates schema-6
with left clamp-linear and right repeat-nearest at `[u,v] == [1,.625]`, reads `55,65,75,255`, then validates schema-7's 96-byte position/RGBA VBO and reads `64,64,127,255`, followed by schema-8's 120-byte position/RGBA/UV VBO and opaque gray texture, reading the modulated `32,32,64,255` center.
This does not claim native Mesa, a native OpenGL context, or browser WebGPU execution from that harness.

## Next compatibility milestones

1. Expand the generic vertex/fragment contract only with the same native/CPU/WebGPU agreement.
2. Design blob, external-memory, and synchronization contracts before any
   Venus capset or Vulkan claim.

## Sources

- [VirtIO GPU device specification](https://docs.oasis-open.org/virtio/virtio/v1.3/virtio-v1.3.pdf)
- [Linux VirtIO-GPU wire UAPI](https://github.com/torvalds/linux/blob/master/include/uapi/linux/virtio_gpu.h)
- [VirGL hardware formats and bindings](https://android.googlesource.com/platform/external/virglrenderer/+/68429e8e1106d0861d9f9f180583bd8381b8bf96/src/virgl_hw.h)
- [VirGL protocol commands](https://android.googlesource.com/platform/external/virglrenderer/+/056b3873e41c015249499dbf9f761c8e9a78b720/src/virgl_protocol.h)
- [Mesa VirGL encoder](https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/gallium/drivers/virgl/virgl_encode.c), [TGSI helper shaders](https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/gallium/auxiliary/util/u_simple_shaders.c), and [VirGL renderer shader decoder](https://gitlab.freedesktop.org/virgl/virglrenderer/-/blob/main/src/vrend/vrend_renderer.c)
- [Mesa blend-state definitions](https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/util/blend.h)
- [Mesa VirGL architecture](https://docs.mesa3d.org/drivers/virgl.html)
- [Mesa Venus architecture](https://docs.mesa3d.org/drivers/venus.html)
