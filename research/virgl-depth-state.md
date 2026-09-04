# Bounded VirGL depth-test seam

## Purpose

This milestone adds one interoperable depth path to capset-1 rendering.  It is
deliberately a narrow proof that the guest's standard VirGL state reaches both
the CPU fallback and WebGPU with the same ordering rule.

## Advertised resource contract

- `PIPE_TEXTURE_2D` only.
- `VIRGL_FORMAT_Z32_FLOAT` (format 18) only.
- Exact `VIRGL_BIND_DEPTH_STENCIL` (bit 0) only.
- One mip level, one layer, no multisampling, sampling, transfer, or texture
  view use.
- The depth resource has the same dimensions as the scanout color target.

The capset exposes format 18 in the `depthstencil` mask, not in the ordinary
color-render-target mask.

## Standard command contract

The accepted state is the standard `VIRGL_OBJECT_DSA` object (type 0):

```text
CREATE_OBJECT DSA [handle, 7, 0, 0, 0]
BIND_OBJECT   DSA [handle]
```

`7` means depth enabled, depth writes enabled, and `PIPE_FUNC_LESS` (enum 1).
No alpha test or stencil state is accepted.  Unbinding uses handle zero;
destroying a bound state unbinds it.

The framebuffer is exactly one color buffer plus an optional depth surface:

```text
SET_FRAMEBUFFER_STATE [1, depth_surface, color_surface]
```

The color attachment remains the scanout resource.  The depth attachment is
only legal with the accepted depth resource and surface format.

## Draw and clear contract

- A depth-tested draw requires the DSA object and the matching depth surface.
- Only the existing solid-color material routes participate in this first
  slice; textures, vertex colors, alpha test, stencil, and alternate compare
  functions remain unsupported.
- Clear is exactly `CLEAR_COLOR0 | CLEAR_DEPTH`, with a depth clear value of
  `1.0`.  The queued packet snapshots all vertices and the canonical clear.
- CPU rasterization interpolates vertex Z after the accepted viewport mapping
  and applies strict less-than before source-over blending.
- The matching WebGPU renderer uses `depth24plus`, `depthCompare: "less"`,
  `depthWriteEnabled: true`, and a depth attachment cleared to `1.0`.

The depth resource is written only after browser completion, as the color
resource is.  A later resource mutation cannot alter a queued draw.

## Explicit exclusions

This is not generic OpenGL, Vulkan, VirGL, or Venus depth/stencil support.  It
does not provide stencil, depth sampling, depth copy/transfer, depth persistence
across submits, multi-draw composition, arbitrary depth formats, or compare
functions other than `LESS`.

## References

- VirGL object and command layout: `virgl_protocol.h` and `vrend_decode.c` in
  virglrenderer.
- Gallium compare values: `pipe/p_defines.h` (`PIPE_FUNC_LESS == 1`).
- WebGPU depth pipeline and render-pass requirements: WebGPU specification.
