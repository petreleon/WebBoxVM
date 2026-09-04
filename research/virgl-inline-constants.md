# Research: bounded VirGL inline constants

## Question

What standard VirGL state can replace a hard-coded fragment color without
claiming resource-backed uniforms, Mesa, OpenGL, or Venus?

## Findings

`VIRGL_CCMD_SET_CONSTANT_BUFFER` carries a shader stage at payload word one,
an index at word two, and inline dwords after that. virglrenderer forwards
those dwords to its constant setter. Mesa's Gallium test uses the canonical
fragment TGSI form `DCL CONST[0][0]` followed by `MOV OUT[0], CONST[0][0]`.

Resource-backed uniform buffers are a different protocol path. They need
resource binding, byte ranges, and later synchronization/external-memory work;
accepting the inline command does not imply that path is present.

## Application

WebBoxVM accepts only command 12 with fragment stage 1, slot zero, and either
four finite normalized f32 dwords or no dwords to clear that binding. The sole
new TGSI program reads `CONST[0][0]` into `OUT[0]`; validation snapshots the
values and reuses the existing bounded solid-color raster and `VGD1` schema-2
route. Invalid state rejects the full cloned submission, leaving prior state
unchanged.

The native Linux guest probe emits this exact state before indexed `DRAW_VBO`.
Its host smoke validates the distinct schema-2 color and the deferred BGRA
readback, so guest DRM transport is covered in addition to Rust/browser tests.

## Deliberate limits

Vertex constants, other slots, arrays, non-color values, resource-backed
uniform buffers, arbitrary TGSI, and any capset expansion remain unsupported.
This is a narrow standard capset-1 transport seam, not Vulkan/Venus support.

## Sources

- [VirGL protocol command layout](https://android.googlesource.com/platform/external/virglrenderer/+/af3d1de900fb0bf9094ed5d6ed361d8c2c71a543/src/virgl_protocol.h)
- [virglrenderer constant-buffer decode](https://android.googlesource.com/platform/external/virglrenderer/+/5fbaeb95136674231649045fa197897ad9bd3ff4/src/vrend_decode.c)
- [Mesa Gallium constant-buffer TGSI test](https://chromium.googlesource.com/chromiumos/third_party/mesa/+/refs/heads/stabilize-13982.70.B-chromeos-amd/src/gallium/auxiliary/util/u_tests.c)
- [virglrenderer resource constant-buffer binding](https://android.googlesource.com/platform/external/virglrenderer/+/refs/tags/android-14.0.0_r25/src/virglrenderer.h)
