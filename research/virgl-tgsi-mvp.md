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
`CONST[0..3]` row. Standard `SET_CONSTANT_BUFFER` carries vertex-stage values.

WebBoxVM already accepts shader objects and vertex input, but it only recognized
an exact `MOV` or a two-component `ADD` offset. The first semantic-IR extension
therefore accepts the canonical four-row `DP4` form and a vertex-stage, slot-zero
matrix with exactly sixteen finite `f32` values.

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

For the first layer, Rust applies the matrix to the bounded copied vertices
before emitting the existing WebGPU packet. A finite positive homogeneous `w`
is divided into canonical clip coordinates; every resulting vertex must remain
inside the already supported clip volume. This preserves the existing no-clipping
boundary while proving standard matrix semantics. Future packet versions can
carry the matrix to a reusable WebGPU uniform buffer, eliminating this small
per-vertex host transform.

## Invariants

1. The four rows must target `x`, `y`, `z`, and `w` exactly once and read
   `CONST[0]` through `CONST[3]` respectively.
2. The vertex constant binding is stage zero, slot zero, and exactly 64 bytes;
   other sizes or stages do not widen this path.
3. A matrix may not produce a non-finite, zero/negative-`w`, or out-of-clip
   vertex. Failure leaves the transactional stream unchanged.
4. Matrix work is O(V) for at most 3,063 normalized list vertices and O(1)
   additional allocation; it is never on the unbounded guest command path.

## Boundary

This is a genuine standard VirGL TGSI and constant-buffer shape, but it is not
general TGSI, clipping, arbitrary uniforms, OpenGL conformance, Vulkan, or
Venus. Venus still requires its generated Vulkan protocol plus host external
memory and synchronization that browser WebGPU does not expose.

## Validation

- Parse canonical and reordered declaration forms into one matrix program.
- Reject duplicate output components, a non-matrix source, and incomplete
  constant ranges.
- Submit ordinary shader-object, vertex constant-buffer, and draw commands;
  verify transformed packet coordinates and raster result.
- Keep the source-file limit, Rust suite, browser suite, and wasm packages
  green before advertising the increment.

## Sources

- [VirGL decoder shader-object path](https://android.googlesource.com/platform/external/virglrenderer/+/refs/heads/main/src/vrend_decode.c)
- [VirGL protocol shader-object layout](https://android.googlesource.com/platform/external/virglrenderer/+/refs/heads/main/src/virgl_protocol.h)
- [VirGL architecture](https://docs.mesa3d.org/drivers/virgl.html)
- [Venus requirements](https://docs.mesa3d.org/drivers/venus.html)
