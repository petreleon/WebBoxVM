# VirGL compatibility track

## Current verified capset-1 slice

WebBoxVM exposes standard VirtIO-GPU capset ID 1 before its private WBG3
capset ID 7. It reports capset 1, version 1, with a 308-byte
`virgl_caps_v1`-layout response. The response advertises only the formats,
primitive, one UBO slot, and limits exercised by this implementation.

This is a guest-visible VirGL wire-protocol vertical slice, not a claim that
Mesa, OpenGL, or arbitrary VirGL workloads work. It supports a full-scanout
clear, one exact standard source-over blend state, deliberately bounded single
solid/inline-constant or resource-backed-fragment-constant color plus one resource-backed vertex XY offset; singleton solid depth draws with standard canonical DSA comparisons; and 2–16-draw solid-constant batches that are wholly non-depth, share one standard depth comparison, or preserve bounded ordered depth comparisons; bounded triangle lists with generic per-vertex-RGBA;
nearest-clamp/repeat or linear-clamp one-texture; texture-times-vertex-color; or two-texture paths with each sampler from that finite set and one viewport/scissor.

| Standard boundary | Current behavior | Deliberate limit |
| --- | --- | --- |
| Capset discovery | `GET_CAPSET_INFO` index 0 reports ID 1/version 1/308 bytes | No capset 2 |
| Texture resources | Packed 2D targets plus two B8G8R8A8 or R8G8B8A8 sampled resources | No mip levels, arrays, blobs, or multisampling |
| Buffer resources | R8 raw vertex/index/constant plus R32G32/R32G32B32A32 float vertex storage | R8 is not a renderable vertex format |
| Context lifecycle | capset-1 create, destroy, attach, and detach are tracked | No shared contexts or fences |
| Resource transfer/copy | 72-byte transfers, isolated bounded command-9 uniform writes, and one bounded copy per submit | No explicit strides, blit, format conversion, or scanout copy |
| VirGL stream | Surface/framebuffer, canonical TGSI, vertex/index/sampler state, inline/resource constants, blend/rasterizer, viewport/scissor, clear, and `DRAW_VBO` | No arbitrary TGSI or fixed-function state |
| Presentation | Clear; singleton material shapes with standard depth compare; 2–16 ordered solid draws, non-depth, shared-depth, or per-record depth comparison, through WebGPU | No mixed-material/depth records, arbitrary blending, or sampler state |
| Completion | CPU pixels change only after browser queue completion | Lost or stale context reports an error |

## Advertised and accepted shapes

The capset render-format mask contains only B8G8R8A8, B8G8R8X8, A8R8G8B8,
and X8R8G8B8. Its sampler mask adds B8G8R8A8 (1) and R8G8B8A8 (67); its vertex-buffer mask contains only
`VIRGL_FORMAT_R32G32_FLOAT` (29) and `VIRGL_FORMAT_R32G32B32A32_FLOAT` (31),
and its primitive mask contains `PIPE_PRIM_TRIANGLES` (bit 4), `PIPE_PRIM_TRIANGLE_STRIP` (bit 5), and `PIPE_PRIM_TRIANGLE_FAN` (bit 6). No GLSL feature level is advertised; the UBO bit and `max_uniform_blocks` are one only for canonical stage-slot-zero color and XY-offset paths.

`RESOURCE_CREATE_3D` accepts only these exact resource forms:

- texture-2D target; packed render target, B8G8R8A8 sampler-view/render-and-sampler,
  or R8G8B8A8 sampler-view-only; depth/array one,
  level zero, zero or one sample, and no flags;
- `PIPE_BUFFER` target; R32G32B32A32_FLOAT with exactly vertex-buffer bind, or
  R8 with exactly vertex-, index-, or constant-buffer bind; width in bytes; height/depth/array one; level/sample zero; no flags.

R8 index storage binds through command 11 `SET_INDEX_BUFFER` at index size 2 or 4 and accepts aligned byte offsets; R8 constant storage uses one aligned 16-byte command-27 range per accepted stage and isolated command-9 byte uploads with zero dword padding.
`SET_VERTEX_BUFFERS` accepts zero through three standard `(stride, offset, resource)` triples, resetting omitted slots.
Solid draws use a format-31 position source at stride 16; textured, vertex-color, and texture-color inputs may stay
interleaved at strides 24/32/40 or split their fixed position/RGBA/UV attributes across slots 0–2; every divisor is zero.

Type-4 shader objects accept only canonical NUL-terminated TGSI text: solid or
`CONST[0][0]` passthrough/fragment-color pairs; a generic RGBA-passthrough fragment pair; a two-generic texture-times-color pair;
a one-2D-`TEX` pair; or a two-2D-`TEX`, `MUL` pair. The latter has one generic UV input, two sampler/views, and one
color output. Initial `OFFSET` is the total text-byte count; a continuation
has its high bit set and names the exact next byte offset. One bounded 4 KiB
source per vertex/fragment stage may be in flight. The only vertex-constant form adds `CONST[0][0]` to `IN[0]`; chunks must retain handle,
stage, and token count; parser failure leaves the cloned context unchanged.
The declared token capacity plus virglrenderer’s translation slack must fit
the recognized TGSI. Stream output, unknown stages, and unrecognized text fail.
Binding zero unbinds, and destroying a bound shader clears its stage. Command 12
`SET_CONSTANT_BUFFER` accepts only fragment stage 1, slot zero, and exactly four finite normalized inline f32 values (or an empty binding to clear it); command 27 accepts vertex stage 0 or fragment stage 1 at slot zero with one attached R8 constant buffer, aligned offset, exact 16-byte range, and a zero range to clear.

Type-1 `VIRGL_OBJECT_BLEND` accepts one exact 11-word `pipe_blend_state`:
blend enabled; an RGBA color mask; RGB `ADD, SRC_ALPHA, INV_SRC_ALPHA`; and
alpha `ADD, ONE, INV_SRC_ALPHA`. A draw requires that object to be bound.
Binding zero unbinds it; every other equation, factor, mask, and independent
blend configuration is rejected. Type-0 `VIRGL_OBJECT_DSA` accepts only depth enable/write bits plus `PIPE_FUNC_NEVER` through `PIPE_FUNC_ALWAYS` in bits 2–4 (`3 | func << 2`), with all remaining DSA state zero; singleton solid draws preserve that comparison exactly.

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

## Clear-and-draw execution

The guest stream uses ordinary VirGL headers, object types 1, 2, 4, 5, 6, 7,
and 8; `SET_FRAMEBUFFER_STATE`, `SET_VIEWPORT_STATE`, `SET_SCISSOR_STATE`,
generic `CLEAR` or `CLEAR_SURFACE`, `SET_VERTEX_BUFFERS`, commands 9/11/12/27
`SET_INDEX_BUFFER`/`SET_CONSTANT_BUFFER`/`SET_UNIFORM_BUFFER`, command 29 `BIND_SHADER`, commands 10/18 sampler bindings,
and command 8 `DRAW_VBO`. Parsing is bounded to 64 KiB; all context mutations
occur on a clone and commit only after the complete stream validates.

The accepted standard 12-word `DRAW_VBO` has a source count from three through 1023, one instance,
`PIPE_PRIM_TRIANGLES` (a multiple of three), `PIPE_PRIM_TRIANGLE_STRIP`, or
`PIPE_PRIM_TRIANGLE_FAN`, zero bias/start-instance/restart, and no stream-output
count. A strip alternates its first two vertices; a fan retains its first spoke,
so at most 1,023 input vertices become at most 3,063 normalized output vertices.
Its indexed field is zero for consecutive VBO records, or one for a command-11
binding that resolves exactly that many little-endian u16 or u32 indices from `start`.
Restart and min/max hint fields are accepted but do not influence the bounded
renderer. One clear may precede one through 16 draws against the current full-scanout
framebuffer. A singleton uses existing `VGD1` material routes; a 2–16 solid sequence uses `VGB1` v1 when non-depth, legacy v2 for shared `LESS`, v3 with one shared standard comparison in flags, or v4 with one canonical comparison word per record. Clear/copy mixing, repeat clear, and mixed depth attachments fail transactionally.

At draw validation Rust snapshots selected 16-byte constant ranges and a bounded position list from attached one-to-three VBO sources, directly or through bounded index-buffer lookups, then expands a strip or fan before validation and packet construction.
Each source position must be finite, have `x`, `y`, and `z` in `[-1, 1]`, `w == 1`, and every consecutive triple must form a nondegenerate triangle. The one vertex UBO form additionally snapshots `[dx, dy, 0, 0]`, with finite `dx/dy` in `[-1, 1]`, translates local copied vertices, and repeats that validation before packet construction.
Vertex-color and texture-color routes snapshot finite normalized RGBA values; texture routes snapshot finite UVs in `[-8, 8]` and
one or two attached B8G8R8A8 or R8G8B8A8 sources, each limited to 64×64. Feedback into the target is rejected.
Schema 6 carries independent exact sampler state; schema 4 remains the legacy nearest-clamp pair. Later buffer, texture, or state mutation cannot alter queued browser work.
Solid color, interpolated vertex color, sampled texels, and texture-times-interpolated-color use the required source-over blend object.

After validation Rust sends a private `VGD1` singleton envelope to the browser. `VGD1`
is not a guest ABI or VirGL command. Each schema validates three through 3,063 normalized list vertices. Schema 2 is 144 bytes: its original
sequence, canvas size, colors, `16N` vertex bytes, viewport, and optional
canonical top-origin scissor. Schema 3 appends `24N` position/UV bytes, one 2D texture size, and canonical BGRA texels; schema 4 appends two sizes and paired
texels for legacy nearest-clamp multiplication; schema 5 carries exact one-texture state `0x1080` or `0x3292`, its size, and texels. Schema 6 carries two sampler words, sizes, and texels.
Schema 7 is `96 + 32N` bytes with position/RGBA vertices plus viewport/scissor. Schema 8 is `108 + 40N + 4WH` bytes with position/RGBA/UV vertices, viewport/scissor, one exact sampler, and one `W,H <= 64` BGRA texture. Schema 9 is the legacy clear-one `LESS` depth form; schema 10 appends one canonical `PIPE_FUNC_*` word. The browser retains schema-1 parsing only for old packets, independently validates schemas 2/3/4/5/6/7/8/9/10,
converts VirGL `z` from `[-w,w]` to WebGPU's `[0,w]`, flips `v` to raw top-origin storage, uses
a matching address/filter sampler, applies equivalent viewport/scissor, and waits for
`GPUQueue.onSubmittedWorkDone()`. `VGB1` is a separate private batch envelope: a 48-byte
header carries sequence, canvas, clear color/depth, and 2–16 records; each record is a 60-byte solid color/viewport/scissor snapshot plus `16N` position bytes, or 64 bytes in v4 with one canonical comparison word. V1 reserves depth as zero for non-depth draws; v2 is the legacy clear-one shared-`LESS` form; v3 carries one shared `PIPE_FUNC_*` value in flags; v4 clears depth to one and preserves each record's comparison. One WebGPU pass clears once and issues those draws in record order.

Only a successful browser acknowledgment changes authoritative CPU state. Rust
rechecks the VirGL context generation, clears the target in its canonical BGRA
storage, rasterizes the same bounded triangle list with source-over, and emits ordinary
scanout damage. Thus `TRANSFER_FROM_HOST_3D` sees a defined result after completion;
failed, stale, or unacknowledged browser work changes no guest pixels.

The clear-only route remains a smaller private `VGC1` envelope with the same deferred completion rule but no pipeline, buffers, or textures.
Browser diagnostics distinguish clear, draw, texture, and dual-texture paths from private capset-7 WBG3 geometry.

## What this does not establish

This slice does not establish Mesa initialization, an OpenGL context, arbitrary
Gallium/TGSI, shader compilation, arbitrary or general constant/uniform buffers, clipping, unbounded or instanced draws,
multiple vertex attributes beyond fixed position/UV, position/RGBA, or position/RGBA/UV layouts, arbitrary
sampling/filtering or blending, stencil, multi-target rendering, general
readback, or a broad VirGL renderer.

It also does not establish Vulkan or Venus. Guest/default blobs, host-only mapped
staging, and a private context-local `WBL1` ordering ledger exist, but Venus still
needs its real renderer-object protocol, external-memory semantics, synchronization,
and a matching capset that this renderer does not advertise.

## Validation retained in the repository

Rust tests prove capset bits, transactional no-clear, malformed-index, and inline/resource-constant render/rejection,
exact source-over and sampler setup, rasterizer unbind rejection, bounded batched-triangle, alternating strip, and spoke-preserving fan expansion, schemas 2/3/4/5/6/7/8/9/10 `VGD1` plus bounded `VGB1`
payloads, normalized per-vertex RGBA interpolation and texture modulation, repeat-at-one, clamp-linear midpoint, ordered non-depth/depth solid-batch blending, and independent two-sampler CPU sampling, R8G8B8A8-to-BGRA normalization, nonzero-offset indexes, deferred
acknowledgment, clipped source-over raster results, viewport/scissor bounds, non-depth/depth batch ordering, exact `EQUAL` depth, and `WBGF` damage.
Browser tests prove private-envelope framing, malformed sampler rejection, exact
independent WebGPU clamp/repeat/linear descriptors, fixed RGBA and RGBA/UV attributes, one/two padded BGRA uploads, viewport/scissor calls,
cached pipelines, standard compare-selected singleton and shared/per-record depth-batch pipelines, bounded `draw(N)`, one batch render pass, and queue-gated completion.

`scripts/virgl_guest_transport_smoke.sh` separately proves native Linux
VirtIO-GPU/DRM/KMS transport for the blob profiles, capset discovery, R8 transfer/copy,
clear/fence, indexed inline-constant, texture, vertex-color, texture-color, depth comparisons, and ordered solid-batch paths.
It also creates a 36-byte R8 constant buffer, populates its color at byte offset
four and `[dx,dy,0,0]` at byte offset 20 through two isolated standard command-9 writes plus readback, sends stage-0 and stage-1 command 27 bindings, validates distinct schema-2 `VGD1` color and translated vertices,
completes that effect, reads both `147,141,58,255` triangles through Linux, then verifies one standard clear plus two `DRAW_VBO`s as `VGB1` v1 and the ordered `0,128,64,255` center, followed by VGB1 v2 clear-one `LESS` depth draws with exact `0,0,128,255` readback, singleton VGD1 schema 10 `EQUAL` at depth one with `128,0,0,255`, VGB1 v3 shared-`EQUAL` depth with `128,0,64,255`, and VGB1 v4 `LESS` then `GREATER` with `0,128,64,255` readback.
This does not claim native Mesa, a native OpenGL context, or browser WebGPU execution
from that harness.

## Next compatibility milestones

1. Add mixed-material or more general depth-batch forms only with a new native/CPU/WebGPU agreement; do not generalize slots or shader forms implicitly.
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
