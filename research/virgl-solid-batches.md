# Bounded VirGL solid and depth batches

## Scope

This is a compatibility increment, not a general VirGL, OpenGL, Vulkan, or
Venus implementation. A valid standard capset-1 stream may now contain one
color clear followed by two through sixteen solid `DRAW_VBO` commands. A
batch is either entirely non-depth or entirely `LESS` depth-tested after a
clear-one depth attachment. Each draw retains the standard state active at
that exact command: fragment constant, expanded vertices, viewport, and
optional scissor.

The restriction is intentional. A singleton continues through the established
`VGD1` material routes (solid, texture, vertex color, texture-color, depth).
A batch is only a source-over solid sequence against one target; a depth batch
also shares one Z32 attachment and exact `LESS` state. Clear/copy mixing,
repeated clears, mixed depth state, or any non-solid batch record is rejected
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
| 4–27 | version, sequence, width, height, count, flags | v1 non-depth or v2 depth; nonzero sequence; 2–16 draws; flags zero |
| 28–47 | clear RGBA and depth clear | finite normalized color; v1 depth exactly zero; v2 depth exactly one |
| each 60-byte record | count, solid RGBA, viewport, scissor | 3–3,063 vertices; triangle-list count; normalized/fitting state |
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
when absent, and issues `draw()` records in original order. V2 additionally
owns one `depth24plus` texture with `less` and depth writes. Only a successful
browser completion permits the CPU-authoritative clear and raster sequence;
lost context, stale generation, validation failure, or failed completion leaves
guest pixels unchanged.

## Demonstrated case

The AArch64 guest proof exercises both versions. V1 clears black, then draws
half-alpha red and half-alpha green; its ordered BGRA center is `0,128,64,255`.
V2 clears color/depth, draws near half-alpha red before far half-alpha green,
and its `LESS` depth result is `0,0,128,255`. The native harness verifies each
exact 264-byte two-record `VGB1` packet, completes its sequence, and checks
that readback. Rust tests cover parser layout, depth ordering, and the 16-record
cap; browser tests prove two draws occur in one source-over WebGPU pass.

## Deliberate next boundaries

- Mixed texture, vertex-color, uniform, non-depth, or alternate-depth records
  do not batch yet.
- There is no stencil, target switch, instancing, or multi-pass model in a
  batch.
- The batch is not a promise of Mesa initialization, generic Gallium/TGSI,
  OpenGL, Vulkan, or Venus compatibility.

## Primary references

- [VirGL renderer command decode dispatch](https://android.googlesource.com/platform/external/virglrenderer/%2B/1cde1fc0a7e1ee9ee03eeac4eb7af330bb53742b/src/vrend_decode.c)
- [VirGL protocol `DRAW_VBO` fields](https://android.googlesource.com/platform/external/virglrenderer/%2B/e2d45bd07834e5a4a8e93cedb863f8eb4cf7c39c/src/virgl_protocol.h)
- [VirtIO GPU specification](https://docs.oasis-open.org/virtio/virtio/v1.3/virtio-v1.3.pdf)
