# Bounded VirGL depth-state seam

## Purpose

This compatibility slice carries the supported standard VirGL depth subset
unchanged from the guest DSA object through queued CPU rasterization and the
WebGPU depth-stencil descriptor. It covers all standard comparison functions
and both enabled and disabled depth writes; it is not full depth/stencil.

## Resource and framebuffer contract

- `PIPE_TEXTURE_2D`, `VIRGL_FORMAT_Z32_FLOAT` (format 18), and exact
  `VIRGL_BIND_DEPTH_STENCIL` (bit 0) only.
- One mip, one layer, no multisampling, sampling, transfer, or depth views.
- The depth texture exactly matches the scanout color target dimensions.
- The framebuffer is one color surface plus an optional matching depth
  surface: `SET_FRAMEBUFFER_STATE [1, depth_surface, color_surface]`.

The capset advertises format 18 in `depthstencil`, not in the ordinary color
render-target mask.

## Standard DSA contract

The accepted object is `VIRGL_OBJECT_DSA` (type 0):

```text
CREATE_OBJECT DSA [handle, state, 0, 0, 0]
BIND_OBJECT   DSA [handle]
```

`state` is canonical when only these low five bits are set:

```text
bit 0       depth test enabled (must be 1)
bit 1       depth write enabled
bits 2..4   PIPE_FUNC_* comparison, 0 through 7
```

The canonical word is `1 | (write << 1) | (compare << 2)`. For example,
`7` is enabled/write-enabled `LESS`; `17` is enabled/write-disabled
`GREATER`. Alpha test and stencil payloads are rejected. Unbinding uses handle
zero; destroying a bound state unbinds it.

## Draw, transport, and completion contract

- A depth-tested solid draw requires both a bound canonical DSA state and the
  matching depth surface. Clear is exactly `CLEAR_COLOR0 | CLEAR_DEPTH` with
  depth clear `1.0`.
- CPU rasterization compares interpolated Z after viewport mapping and writes
  the stored depth only when the DSA write bit is set; source-over blending is
  still performed after a passing read-only test.
- WebGPU uses `depth24plus`; its pipeline key is the pair
  `(depthCompare, depthWriteEnabled)`, preventing read-only draws from sharing
  a write-enabled pipeline.
- `VGD1` v9 remains byte-compatible for write-enabled `LESS`; v10 remains the
  write-enabled non-`LESS` comparison form; v11 carries the canonical DSA
  word for read-only depth states.
- `VGB1` v1–v4 remain byte-compatible. V5 carries one canonical DSA word per
  ordered record whenever any record disables depth writes.

Completion remains fail-closed: a browser failure, stale generation, or
validation error leaves the CPU-authoritative color/depth effect unapplied.

## Demonstrated boundary

The guest probe is configured to emit VGB1 v5 with a write-enabled `LESS` near
red draw followed by a read-only `GREATER` far green draw. Its harness expects
the exact `7` then `17` DSA record words and the blended `0,128,64,255` BGRA
result. Rust depth tests separately assert that the stored center depth remains
`0.25` after the read-only second draw.

## Explicit exclusions

This does not implement stencil, alpha test, depth sampling/copy/transfer,
cross-submit depth persistence, alternate depth formats, generic Gallium,
OpenGL, Vulkan, or Venus.

## References

- VirGL object and command layout: `virgl_protocol.h` and `vrend_decode.c` in
  virglrenderer.
- Gallium comparison values: `pipe/p_defines.h`.
- WebGPU depth pipeline and render-pass requirements: WebGPU specification.
