# Bounded VirGL solid and depth batches

## Scope

This is a compatibility increment, not a general VirGL, OpenGL, Vulkan, or
Venus implementation. A valid standard capset-1/2 stream may now contain one
color clear followed by two through sixteen solid `DRAW_VBO` commands. A
batch is either entirely non-depth or depth-tested after a clear-one depth
attachment. V2/V3 share one standard DSA comparison; v4 preserves one comparison per record; v5 preserves each canonical DSA comparison/write-mask pair. Each draw retains the standard state active at
that exact command: fragment constant, expanded vertices, viewport, and
optional scissor.

Singleton and depth batches may use the accepted standard DSA comparison
functions and write mask. V2 remains a byte-compatible legacy `LESS` form;
V3 carries one shared write-enabled comparison, V4 carries one write-enabled
comparison per record, and V5 carries a canonical DSA word per record.

The restriction is intentional. A singleton continues through the established
`VGD1` material routes (solid, texture, vertex color, texture-color, depth).
A batch is only a source-over solid sequence against one target; a depth batch
also shares one Z32 attachment and preserves its exact comparison state. Clear/copy mixing,
repeated clears, mixed depth attachments, or any non-solid batch record is rejected
before the context clone commits.

## Why this is a real protocol step

VirGL command buffers are sequential: the maintained renderer decoder dispatches
each decoded command, including `DRAW_VBO`. WebBoxVM therefore preserves the
guest's ordering rather than treating a later draw as an invalid duplicate.
The guest probe emits ordinary `CLEAR`, `SET_CONSTANT_BUFFER`, and `DRAW_VBO`
commands; `VGB1` is only the bounded host-to-browser transport envelope.

## VGB1 envelope

`VGB1` is a private, little-endian browser packet, never a guest ABI. It is
fully parsed before GPU allocation or rendering.

| Bytes | Field | Constraint |
| --- | --- | --- |
| 0–3 | magic | `VGB1` |
| 4–27 | version, sequence, width, height, count, flags | v1 non-depth, v2 legacy `LESS`, v3 shared depth, v4 per-record comparison, or v5 per-record DSA; nonzero sequence; 2–16 draws; v1/v2/v4/v5 flags zero, v3 `PIPE_FUNC_*` 0–7 |
| 28–47 | clear RGBA and depth clear | finite normalized color; v1 depth exactly zero; v2–v5 depth exactly one |
| each record | 60-byte count/solid/viewport/scissor, or v4/v5 64-byte count/state/solid/viewport/scissor | 3–3,063 vertices; triangle-list count; normalized/fitting state; v5 state is canonical `1 | write<<1 | compare<<2` |
| record tail | `16N` position bytes | finite `x/y/z` in `[-1,1]`, `w == 1`, nondegenerate triples |

The aggregate vertex budget is `16 × 3,063`; parser bounds, target dimensions,
and all scissor/viewport limits are checked independently in Rust and
JavaScript. Later guest buffer or constant mutation cannot affect a queued
record because each record contains copied state.

## Completion invariant

```text
standard guest stream → validated snapshots → VGB1 → one WebGPU render pass
                                               │
GPUQueue.onSubmittedWorkDone() ← ordered draw calls and one clear
                                               │
                      CPU clear + ordered source-over raster + scanout damage
```

The WebGPU renderer uploads position/color-interleaved vertices so every draw
uses its own snapshotted color in a single pass. It clears once, resets scissor
when absent, and issues `draw()` records in original order. V2 uses `less`; V3
maps its shared canonical comparison to one `depth24plus` pipeline, while V4/V5 select cached pipelines per record by `(comparison, write-mask)`. Only a successful
browser completion permits the CPU-authoritative clear and raster sequence;
lost context, stale generation, validation failure, or failed completion leaves
guest pixels unchanged.

## Demonstrated case

The AArch64 guest probe is configured to exercise all versions. V1 clears black, then draws
half-alpha red and half-alpha green; its ordered BGRA center is `0,128,64,255`.
V2 clears color/depth, draws near half-alpha red before far half-alpha green,
and its `LESS` depth result is `0,0,128,255`. V3 uses `EQUAL` at z=1 for
half-alpha red then blue, producing `128,0,64,255`. V4 changes from `LESS` at
z=-.5 to `GREATER` at z=.5, producing `0,128,64,255`. V5 uses canonical
state words `7` (`LESS`, write) then `17` (`GREATER`, read-only), with the
same blended center while Rust verifies the retained near depth. The native
harness verifies each exact 264-byte v1/v2/v3 or 272-byte v4/v5 two-record
`VGB1` packet, completes its sequence, and checks that readback. Rust tests cover parser layout, comparison isolation, depth ordering,
and the 16-record cap; browser tests prove two draws occur in one source-over WebGPU pass.

## Deliberate next boundaries

- Mixed texture, vertex-color, uniform, or non-depth records
  do not batch yet.
- There is no stencil, target switch, instancing, or multi-pass model in a
  batch.
- The batch is not a promise of Mesa initialization, generic Gallium/TGSI,
  OpenGL, Vulkan, or Venus compatibility.

## Primary references

- [VirGL renderer command decode dispatch](https://android.googlesource.com/platform/external/virglrenderer/%2B/1cde1fc0a7e1ee9ee03eeac4eb7af330bb53742b/src/vrend_decode.c)
- [VirGL protocol `DRAW_VBO` fields](https://android.googlesource.com/platform/external/virglrenderer/%2B/e2d45bd07834e5a4a8e93cedb863f8eb4cf7c39c/src/virgl_protocol.h)
- [VirtIO GPU specification](https://docs.oasis-open.org/virtio/virtio/v1.3/virtio-v1.3.pdf)
