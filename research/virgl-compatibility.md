# VirGL compatibility track

## Current verified VirGL capset slice

WebBoxVM exposes standard VirtIO-GPU capset ID 1 and capset ID 2 before its
private WBG3 capset ID 7. It reports ID 1/version 1 with a 308-byte
`virgl_caps_v1` response and ID 2/version 2 with the current 1,376-byte
growable `virgl_caps_v2` layout. Both advertise only formats, primitive, one
UBO slot, and limits exercised by this implementation.

This is a guest-visible VirGL wire-protocol vertical slice, not a claim that
Mesa, OpenGL, or arbitrary VirGL workloads work. It supports a full-scanout
clear, one exact standard source-over blend state, deliberately bounded single
solid/inline-constant or resource-backed-fragment-constant color plus one resource-backed vertex XY offset; singleton solid, interpolated or constant-modulated vertex-color, one-texture, fragment-constant-modulated texture, or texture-times-vertex-color depth draws with canonical DSA comparisons/write masks; and 2–16-draw batches of those supported material snapshots that are wholly non-depth or preserve bounded ordered DSA states; bounded triangle lists with generic per-vertex-RGBA;
nearest-clamp/repeat or linear-clamp one-texture; fragment-constant-modulated texture; texture-times-vertex-color; or two-texture paths with each sampler from that finite set and one viewport/scissor.

| Standard boundary | Current behavior | Deliberate limit |
| --- | --- | --- |
| Capset discovery | Index 0 reports ID 1/version 1/308 bytes; index 1 ID 2/version 2/1,376 bytes | No Venus capset 4 |
| Texture resources | Packed 2D targets plus two B8G8R8A8 or R8G8B8A8 sampled resources | No mip levels, arrays, blobs, or multisampling |
| Buffer resources | R8 raw vertex/index/constant plus R32G32/R32G32B32A32 float vertex storage | R8 is not a renderable vertex format |
| Context lifecycle | capset-1/2 create, destroy, attach, and detach share bounded state | No shared contexts or fences |
| Resource transfer/copy | 72-byte transfers, isolated bounded command-9 uniform writes, and one bounded copy per submit | No explicit strides, blit, format conversion, or scanout copy |
| VirGL stream | Surface/framebuffer, bounded normalized TGSI shapes, vertex/index/sampler state, inline/resource constants, blend/rasterizer, viewport/scissor, clear, and `DRAW_VBO` | No arbitrary TGSI or fixed-function state |
| Presentation | Clear; singleton shapes plus 2–16 ordered solid/vertex-color/one-/two-texture/texture-color snapshots, with per-record DSA where depth supports the material, through one WebGPU pass; `VGB1`/`VGM1` can map final GPU color | No arbitrary shader/state, blending, or sampler state |
| Completion | `VGB1`/`VGM1` color changes after mapped GPU payload validation; other paths retain deferred CPU replay | Lost, stale, malformed, or unsupported payloads report an error/fallback |

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

Type-4 shader objects accept only bounded NUL-terminated TGSI shapes: solid or
`CONST[0][0]` passthrough/fragment-color pairs; generic RGBA passthrough or generic-RGBA-times-fragment-constant pairs; a two-generic texture-times-color pair;
a one-2D-`TEX` passthrough or texture-times-`CONST[0][0]` pair; or a two-2D-`TEX`, `MUL` pair. Declaration order and full-vector `.xyzw` spellings normalize for those forms, while unknown operations, repeated/overlapping declarations, and ambiguous immediates fail. The latter pair has one generic UV input, two sampler/views, and one color output. Initial `OFFSET` is the total text-byte count; a continuation
has its high bit set and names the exact next byte offset. One bounded 4 KiB
source per vertex/fragment stage may be in flight. Vertex-constant forms add `CONST[0][0]` to `IN[0]`, optionally preserving one `GENERIC[0]` varying; chunks must retain handle,
stage, and token count; parser failure leaves the cloned context unchanged.
The declared token capacity plus virglrenderer’s translation slack must fit
the recognized shape. Stream output, unknown stages, and unrecognized text fail.
The vertex-color constant form resolves normalized inline or resource-backed values once while assembling `DrawWork`, multiplies each normalized vertex channel once, and reuses the standard vertex-color `VGD1`/`VGM1` route without a new browser pipeline. The textured constant form instead turns each fixed position/UV snapshot into a constant-RGBA texture-color snapshot, preserving the existing filtered texture route and schemas 8/14 or `VGM1` without pre-quantizing texels.
Binding zero unbinds, and destroying a bound shader clears its stage. Command 12
`SET_CONSTANT_BUFFER` accepts only fragment stage 1, slot zero, and exactly four finite normalized inline f32 values (or an empty binding to clear it); command 27 accepts vertex stage 0 or fragment stage 1 at slot zero with one attached R8 constant buffer, aligned offset, exact 16-byte range, and a zero range to clear.

Type-1 `VIRGL_OBJECT_BLEND` accepts one exact 11-word `pipe_blend_state`:
blend enabled; an RGBA color mask; RGB `ADD, SRC_ALPHA, INV_SRC_ALPHA`; and
alpha `ADD, ONE, INV_SRC_ALPHA`. A draw requires that object to be bound.
Binding zero unbinds it; every other equation, factor, mask, and independent
blend configuration is rejected. Type-0 `VIRGL_OBJECT_DSA` accepts only depth-test bit 0, write bit 1, and `PIPE_FUNC_NEVER` through `PIPE_FUNC_ALWAYS` in bits 2–4 (`1 | write << 1 | func << 2`), with all remaining state zero; singleton solid/vertex-color/one-texture/texture-color draws preserve it exactly.

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
framebuffer. An eligible non-depth singleton uses resident `VGB1` v6/v7 for a
solid or resident `VGM1` v2/v3 for any other supported material; an ineligible
singleton uses `VGD1`. A solid-only 2–16 sequence uses `VGB1` v1 when non-depth,
legacy v2 for shared `LESS`, v3 with one shared comparison in flags, v4 with
per-record comparisons, or v5 with per-record canonical DSA state. Other
supported 2–16 sequences use `VGM1`. Clear/copy mixing, repeat clear, and mixed depth attachments fail transactionally.

At draw validation Rust snapshots selected 16-byte constant ranges and a bounded position list from attached one-to-three VBO sources, directly or through bounded index-buffer lookups, then expands a strip or fan before validation and packet construction.
Each source position must be finite, have `x`, `y`, and `z` in `[-1, 1]`, `w == 1`, and every consecutive triple must form a nondegenerate triangle. Vertex UBO forms additionally snapshot `[dx, dy, 0, 0]`, with finite `dx/dy` in `[-1, 1]`, translate local copied vertices, and repeat that validation before packet construction; the generic form retains one fixed UV varying.
Vertex-color and texture-color routes snapshot finite normalized RGBA values; texture routes snapshot finite UVs in `[-8, 8]` and
one or two attached B8G8R8A8 or R8G8B8A8 sources, each limited to 64×64. Feedback into the target is rejected.
Schema 6 carries independent exact sampler state; schema 4 remains the legacy nearest-clamp pair. Later buffer, texture, or state mutation cannot alter queued browser work.
Solid color, interpolated vertex color, sampled texels, and texture-times-interpolated-color use the required source-over blend object.

After validation an ineligible singleton uses the private `VGD1` envelope; `VGD1`
is not a guest ABI or VirGL command. Each schema validates three through 3,063 normalized list vertices. Schema 2 is 144 bytes: its original
sequence, canvas size, colors, `16N` vertex bytes, viewport, and optional
canonical top-origin scissor. Schema 3 appends `24N` position/UV bytes, one 2D texture size, and canonical BGRA texels; schema 4 appends two sizes and paired
texels for legacy nearest-clamp multiplication; schema 5 carries exact one-texture state `0x1080` or `0x3292`, its size, and texels. Schema 6 carries two sampler words, sizes, and texels.
Schema 7 is `96 + 32N` bytes with position/RGBA vertices plus viewport/scissor. Schema 8 is `108 + 40N + 4WH` bytes with position/RGBA/UV vertices, viewport/scissor, one exact sampler, and one `W,H <= 64` BGRA texture. Schema 9 is legacy clear-one `LESS`; schema 10 carries a write-enabled comparison; schema 11 carries read-only solid DSA; schema 12 is schema 7 plus clear-one/canonical DSA vertex-color depth; schema 13 is `116 + 24N + 4WH` bytes with exact sampler, texture, clear-one, and canonical DSA one-texture depth; schema 14 is `116 + 40N + 4WH` bytes with the same depth tail and fixed texture-times-RGBA attributes. The browser retains schema-1 parsing only for old packets, independently validates schemas 2–14,
converts VirGL `z` from `[-w,w]` to WebGPU's `[0,w]`, flips `v` to raw top-origin storage, uses
a matching address/filter sampler, and waits for queue completion. `VGB1` remains the compact solid-only envelope. `VGM1` has a 48-byte header plus ordered 52-byte records, immutable snapshots and vertices; its depth flag requires clear-one plus canonical DSA. One WebGPU pass clears once and issues every record in order.

For `VGB1`/`VGM1`, the canvas has `COPY_SRC`; after each pass the browser copies the target to a padded `MAP_READ` buffer, strips padding after `mapAsync`, and sends a sequence-tagged BGRA/RGBA payload. Rust accepts only the exact in-flight envelope target/rect/format/byte count and context generation, then writes final GPU color to canonical BGRA storage and damage. Depth batches retain their bounded CPU depth update before color replacement. Other envelopes still replay only after a successful Boolean acknowledgment. Failed, stale, malformed, or unacknowledged work changes no guest pixels. See [bounded GPU readback](virgl-gpu-readback.md).

The clear-only route remains a smaller private `VGC1` envelope with the same deferred completion rule but no pipeline, buffers, or textures.
Browser diagnostics distinguish clear, draw, texture, and dual-texture paths from private capset-7 WBG3 geometry.

## What this does not establish

This slice does not establish Mesa initialization, an OpenGL context, arbitrary
Gallium/TGSI, shader compilation, arbitrary or general constant/uniform buffers, clipping, unbounded or instanced draws,
multiple vertex attributes beyond fixed position/UV, position/RGBA, or position/RGBA/UV layouts, arbitrary
sampling/filtering or blending, stencil, multi-target rendering, general
arbitrary-target readback, or a broad VirGL renderer.

It also does not establish Vulkan or Venus. Guest/default blobs, host-only mapped
staging, and a private context-local `WBL1` ordering ledger exist, but Venus still
needs its real renderer-object protocol, external-memory semantics, synchronization,
and a matching capset that this renderer does not advertise.

## Validation retained in the repository

Rust tests prove capset bits, transactional no-clear, malformed-index, and inline/resource-constant render/rejection,
exact source-over and sampler setup, rasterizer unbind rejection, bounded batched-triangle, alternating strip, and spoke-preserving fan expansion, schemas 2–14 `VGD1`, bounded `VGB1`, and mixed-material `VGM1`
payloads, singleton and ordered non-depth/depth batch blending, normalized per-vertex RGBA interpolation and texture modulation, repeat-at-one, clamp-linear midpoint, solid/vertex-color/one-texture/texture-color depth, and independent two-sampler CPU sampling, R8G8B8A8-to-BGRA normalization, nonzero-offset indexes, deferred
acknowledgment, clipped source-over raster results, viewport/scissor bounds, non-depth/depth batch ordering, exact `EQUAL` depth, canonical write masks, strict VGB1/VGM1 GPU-color readback, and `WBGF` damage.
Browser tests prove private-envelope framing, malformed sampler rejection, exact
independent WebGPU clamp/repeat/linear descriptors, fixed RGBA and RGBA/UV attributes, one/two padded BGRA uploads, viewport/scissor calls,
cached pipelines, exact byte-identical vertex uploads, and material bind groups, standard singleton material and `VGB1`/`VGM1` shared/per-record depth-batch pipelines, bounded `draw(N)`, one batch render pass, padded map readback, and queue-gated completion.

`scripts/virgl_guest_transport_smoke.sh` provides a native Linux transport harness for
VirtIO-GPU/DRM/KMS transport for the blob profiles, capset discovery, R8 transfer/copy,
clear/fence, indexed inline-constant, texture, vertex-color, texture-color, solid/vertex-color/one-texture/texture-color depth state, and ordered solid-batch paths.
It also creates a 36-byte R8 constant buffer, populates its color at byte offset
four and `[dx,dy,0,0]` at byte offset 20 through two isolated standard command-9 writes plus readback, sends stage-0 and stage-1 command 27 bindings, validates distinct schema-2 `VGD1` color and translated vertices,
completes that effect, reads both `147,141,58,255` triangles through Linux, then expects one standard clear plus two `DRAW_VBO`s as `VGB1` v1 and the ordered `0,128,64,255` center, followed by VGB1 v2 clear-one `LESS`, VGD1 schema 10 `EQUAL`, VGB1 v3 shared-`EQUAL`, v4 `LESS`/`GREATER`, v5 canonical state words `7`/`17`, schema 12 vertex-color DSA word `5`, schema 13 one-texture DSA word `7`, and schema 14 texture-color DSA word `7`.
The newly wired guest phase must then produce one exact 364-byte `VGM1` depth-material batch: a far solid red record followed by a near stage-0-UBO-translated `TEX`×`CONST[0][0]` record reified as texture-color, both DSA word `7`, and a `64,64,64,255` center after acknowledgment. A bounded 55-second native run on 2026-09-04 built the 52,984-byte demo and reached kernel timestamp 18 seconds, but timed out in the shell phase before guest graphics, so native execution of this newest phase remains inconclusive.
This does not claim native Mesa, a native OpenGL context, or browser WebGPU execution
from that harness.

## Next compatibility milestones

1. Obtain native end-to-end execution of the newly wired `VGM1` guest phase once shell startup is available, then extend the bounded semantic IR with explicitly validated operations and state rather than add exact TGSI strings.
2. Design blob, external-memory, and synchronization contracts before any
   Venus capset 4 or Vulkan claim; see [VirGL2 capset boundary](virgl2-capset.md).

## Sources

- [VirtIO GPU device specification](https://docs.oasis-open.org/virtio/virtio/v1.3/virtio-v1.3.pdf)
- [Linux VirtIO-GPU wire UAPI](https://github.com/torvalds/linux/blob/master/include/uapi/linux/virtio_gpu.h)
- [VirGL hardware formats and bindings](https://android.googlesource.com/platform/external/virglrenderer/+/68429e8e1106d0861d9f9f180583bd8381b8bf96/src/virgl_hw.h)
- [VirGL protocol commands](https://android.googlesource.com/platform/external/virglrenderer/+/056b3873e41c015249499dbf9f761c8e9a78b720/src/virgl_protocol.h)
- [Mesa VirGL encoder](https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/gallium/drivers/virgl/virgl_encode.c), [TGSI helper shaders](https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/gallium/auxiliary/util/u_simple_shaders.c), and [VirGL renderer shader decoder](https://gitlab.freedesktop.org/virgl/virglrenderer/-/blob/main/src/vrend/vrend_renderer.c)
- [Mesa blend-state definitions](https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/util/blend.h)
- [Mesa VirGL architecture](https://docs.mesa3d.org/drivers/virgl.html)
- [Mesa Venus architecture](https://docs.mesa3d.org/drivers/venus.html)
