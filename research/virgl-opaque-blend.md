# Opaque VirGL blend state

## Question

What is the smallest standard VirGL blend state that lets the bounded WebGPU
renderer write a fragment directly instead of always using source-over?

## Findings

`VIRGL_OBJECT_BLEND` encodes each render target in the S2 word. Bit zero is
`blend_enable` and bits 27 through 30 are the RGBA color mask. Therefore the
standard word `0x78000000` means blending disabled with all four color channels
writable. The VirGL renderer test creates that state by zeroing
`pipe_blend_state` and setting only `rt[0].colormask = PIPE_MASK_RGBA`.

## Application to WebBoxVM

The stream accepts exactly that full-mask replace state in addition to the
existing source-over state. A uniform non-depth replace batch uses private
`VGB1` version 8 for solid draws or `VGM1` version 4 for other supported
materials. The browser validates those versions and omits WebGPU's `blend`
descriptor, so fragment RGBA replaces the target as the standard state
requires.

All records in one private batch must have the same blend mode. A mixed
source-over/replace batch is rejected transactionally rather than silently
encoding one mode as the other. Replace batches currently remain readback
backed and non-resident; the CPU fallback also fails closed instead of applying
source-over by mistake.

## Boundary

This is non-depth, full-RGBA replace only. Partial color masks, blend equations
or factors other than source-over, logic operations, independent targets,
opaque depth rendering, general OpenGL, Vulkan, and Venus remain outside the
implemented subset.

## Validation

Rust tests construct the standard wire word, require a `VGB1` version-8
readback completion, and check RGBA-to-BGRA delivery. Browser tests validate
both version-8 solid and version-4 mixed-material packets and prove their
WebGPU targets have no source-over descriptor.

## Sources

- [VirGL blend-state protocol fields](https://android.googlesource.com/platform/external/virglrenderer/+/68429e8e1106d0861d9f9f180583bd8381b8bf96/src/virgl_protocol.h)
- [VirGL renderer opaque blend test](https://android.googlesource.com/platform/external/virglrenderer/+/68429e8e1106d0861d9f9f180583bd8381b8bf96/tests/test_virgl_cmd.c)
- [WebGPU color-target state](https://www.w3.org/TR/webgpu/#color-target-state)
