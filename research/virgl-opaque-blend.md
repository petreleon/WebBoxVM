# Opaque VirGL blend state

## Question

What is the smallest standard VirGL blend state that lets the bounded WebGPU
renderer write a fragment directly instead of always using source-over?

## Findings

`VIRGL_OBJECT_BLEND` encodes each render target in the S2 word. Bit zero is
`blend_enable` and bits 27 through 30 are the RGBA color mask. Therefore the
standard word `0x78000000` means blending disabled with all four color channels
writable, while `0x38000000` means blending disabled with R, G, and B writable.
Every other nonzero four-bit mask is the same standard state with its selected
channels writable. The VirGL renderer test creates the full-mask state by
zeroing `pipe_blend_state` and setting only `rt[0].colormask = PIPE_MASK_RGBA`.

## Application to WebBoxVM

The stream accepts every nonzero opaque mask in addition to source-over. A
uniform full-mask batch uses private `VGB1` v8/v9 or `VGM1` v4/v5; RGB-only
uses `VGB1` v10/v11 or `VGM1` v6/v7; every other mask uses `VGB1` v12/v13 or
`VGM1` v8/v9 for non-depth/depth solid or other material draws. The browser
omits WebGPU's `blend` descriptor and carries the exact bit value to
`writeMask`, leaving unselected channels unchanged by a draw.

All records in one private batch must have the same blend mode. A mixed
source-over/replace batch is rejected transactionally rather than silently
encoding one mode as the other. Replace batches currently remain readback
backed and non-resident; the CPU fallback also fails closed instead of applying
source-over by mistake.

## Boundary

This is every nonzero RGBA mask with bounded depth testing only. A zero mask,
blend equations or factors other than source-over, logic operations,
independent targets, general OpenGL, Vulkan, and Venus remain outside the
implemented subset.

## Validation

Rust tests construct full, RGB-only, and partial standard wire words, require
`VGB1` v8/v9/v10/v12 readback completion, and check RGBA-to-BGRA delivery plus
the bounded depth shadow. Browser tests validate v8–v13 solid and v4–v9
material packets and prove exact nonzero WebGPU masks with no source-over.

## Sources

- [VirGL blend-state protocol fields](https://android.googlesource.com/platform/external/virglrenderer/+/68429e8e1106d0861d9f9f180583bd8381b8bf96/src/virgl_protocol.h)
- [VirGL renderer opaque blend test](https://android.googlesource.com/platform/external/virglrenderer/+/68429e8e1106d0861d9f9f180583bd8381b8bf96/tests/test_virgl_cmd.c)
- [WebGPU color-target state](https://www.w3.org/TR/webgpu/#color-target-state)
