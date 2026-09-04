# VirGL TGSI MVP foundation

## Question

What is the smallest standard VirGL shader and state expansion that moves
WebBoxVM beyond pre-transformed clip-space triangles toward ordinary OpenGL
vertex transforms?

## Upstream contract

VirGL transports Gallium TGSI shader text in a standard shader object. The
upstream decoder passes that text to the renderer shader path, which translates
it for the host graphics API. A canonical vertex transform is four `DP4`
instructions: each `OUT[0]` component is the dot product of `IN[0]` and one
`CONST[0..3]` row. Standard `SET_CONSTANT_BUFFER` or resource-backed
`SET_UNIFORM_BUFFER` carries vertex-stage values.

WebBoxVM already accepts shader objects and vertex input, but it only recognized
an exact `MOV` or a two-component `ADD` offset. The first semantic-IR extension
therefore accepts the canonical four-row `DP4` form, zero through two exact
generic `MOV` passthroughs, and a vertex-stage, slot-zero matrix with exactly
sixteen finite `f32` values either inline or in one attached 64-byte buffer.

## Applied contract

```text
DCL IN[0], POSITION
DCL CONST[0..3]
DCL OUT[0], POSITION
DP4 OUT[0].x, IN[0], CONST[0]
DP4 OUT[0].y, IN[0], CONST[1]
DP4 OUT[0].z, IN[0], CONST[2]
DP4 OUT[0].w, IN[0], CONST[3]
END
```

One or two fixed generic outputs may follow or interleave with the four rows:

```text
MOV OUT[1], IN[1]
MOV OUT[2], IN[2]
```

They retain the existing fixed color/UV layouts while the position undergoes
the matrix transform.

Rust always applies the matrix to a bounded copied vertex snapshot for deferred
CPU replay. A finite positive homogeneous `w` is divided into canonical clip
coordinates; every resulting vertex must remain inside the already supported
clip volume. For the exact non-depth solid form that reaches a nonresident
singleton, private `VGD1` v15 additionally carries the original 16-byte
positions and the row-major 64-byte matrix. The browser rechecks the projected
bounds, writes the rows plus solid color into an 80-byte WebGPU uniform, and
uses four row-dot-products in the vertex shader. Generic-varying, texture, and
depth forms retain the transformed CPU-packet route. This preserves the
no-clipping boundary without claiming a general shader compiler.

## Invariants

1. The four rows must target `x`, `y`, `z`, and `w` exactly once and read
   `CONST[0]` through `CONST[3]` respectively.
2. A varying form has only exact `OUT[1..2] = IN[1..2]` moves; it neither
   introduces a general register machine nor changes the fixed vertex layouts.
3. The vertex matrix binding is stage zero, slot zero, and exactly 64 bytes;
   it may be inline or resource-backed, while other sizes or stages do not
   widen this path.
4. A matrix may not produce a non-finite, zero/negative-`w`, or out-of-clip
   vertex. Failure leaves the transactional stream unchanged.
5. Matrix work is O(V) for at most 3,063 normalized list vertices; the GPU lane
   retains one bounded raw snapshot for browser presentation and never opens an
   unbounded guest command path.

## Boundary

This is a genuine standard VirGL TGSI and constant-buffer shape, but it is not
general TGSI, clipping, arbitrary uniforms, OpenGL conformance, Vulkan, or
Venus. Venus still requires its generated Vulkan protocol plus host external
memory and synchronization that browser WebGPU does not expose.

## Validation

- Parse canonical and reordered declaration/instruction forms into matrix-only
  or exact generic-varying programs.
- Reject duplicate output components, a non-matrix source, and incomplete
  constant ranges.
- Submit ordinary shader-object, inline or resource-backed vertex matrix
  bindings, solid, and textured draw commands; verify v15 raw matrix/vertex
  snapshots, browser WebGPU uniform rows, transformed packet coordinates, and
  raster results.
- Keep the source-file limit, Rust suite, browser suite, and wasm packages
  green before advertising the increment.

## Sources

- [VirGL decoder shader-object path](https://android.googlesource.com/platform/external/virglrenderer/+/refs/heads/main/src/vrend_decode.c)
- [VirGL protocol shader-object layout](https://android.googlesource.com/platform/external/virglrenderer/+/refs/heads/main/src/virgl_protocol.h)
- [VirGL architecture](https://docs.mesa3d.org/drivers/virgl.html)
- [Venus requirements](https://docs.mesa3d.org/drivers/venus.html)
