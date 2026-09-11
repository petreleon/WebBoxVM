# WebBoxVM standard VirGL uniform, texture, color, depth, and material-batch probe

This freestanding AArch64 Linux program is a deliberately small guest-side
proof for a conservative standard capset-1 vertex, buffer, copy, upload, clear,
source-over blend, rasterizer, viewport/scissor, inline-constant and bounded vertex/fragment-uniform triangles, interpolated vertex color, sampled texture, fragment-constant texture, texture-modulated color, canonical solid/vertex-color/sampled-texture/texture-color depth state, and ordered non-depth plus shared/per-draw-comparison/write-mask depth-tested solid-draw and mixed-material batches,
and readback path. It is not Mesa, OpenGL, or Vulkan.

It also proves Linux's `cmd_size` blob path: a private `WBL1` opaque command is
sent by the kernel before one nonzero-`blob_id` default blob creation. That
tests context-local allocation ordering only; it is not a Venus command.

It opens `/dev/dri/card0`, reads the Linux `virtgpu` capset-1 response, creates
a B8G8R8X8 render-target resource and two R8 `PIPE_BUFFER` vertex-buffer
resources. It maps the source buffer backing with `DRM_IOCTL_VIRTGPU_MAP`,
writes eight bytes at a nonzero backing offset, transfers them through a
standard byte-range `DRM_IOCTL_VIRTGPU_TRANSFER_TO_HOST`, submits one standard
type-5 vertex-element create/bind plus `SET_VERTEX_BUFFERS` stream, submits
`RESOURCE_COPY_REGION` to the destination buffer, and reads it back at a
different backing offset through `DRM_IOCTL_VIRTGPU_TRANSFER_FROM_HOST`. It then maps the color
resource, writes two BGRX pixels, transfers them with `DRM_IOCTL_VIRTGPU_TRANSFER_TO_HOST`,
uploads two pixels into a four-pixel off-screen source, submits standard
VirGL `RESOURCE_COPY_REGION`, waits for the destination resource, and verifies
the destination through `DRM_IOCTL_VIRTGPU_TRANSFER_FROM_HOST`. It then views
the scanout resource through KMS's XRGB primary plane, submits standard
`OBJECT_SURFACE`, `SET_FRAMEBUFFER_STATE`, and generic `CLEAR` commands, waits
for the resource fence, then obtains and checks two clear pixels through
`DRM_IOCTL_VIRTGPU_TRANSFER_FROM_HOST`. Finally it creates a 96-byte
`R32G32B32A32_FLOAT` vertex buffer and 14-byte R8 index buffer, uploads six
fixed clip-space vertices plus a two-byte pad then little-endian `[2,1,0,5,4,3]` indices, and submits one
standard type-1 source-over blend object, type-2 scissor rasterizer, type-4
`CONST[0][0]` shader object, type-5 vertex elements, shader binds, command-11 index binding at byte offset two,
command-12 fragment-slot-zero inline constants, one viewport/scissor, a generic clear, and one indexed `DRAW_VBO`. It waits again
and requires both triangles to read back as source-over BGRA
(`121,115,134,255`) while their center gap remains clear. It then
reuses the scanout surface, binds distinct persistent-state handles, and creates
two R8G8B8A8 sampler-view textures plus a 72-byte interleaved position/UV VBO.
The type-7 nearest S/T-repeat sampler uses `u == 1`, so the type-6 identity
view must wrap to the first canonical BGRA texel. It then switches to standard
clamp/linear at `u == .5`, whose exact 2×2 midpoint must read `25,35,45,255`.
Finally it binds left clamp/linear and right repeat/nearest views at `[u,v] ==
`[1,.625]`; the distinct right texture proves wrap while the left interpolates
to `55,65,75,255` after its fence. A final 96-byte position/RGBA VBO uses a
generic TGSI varying, and its barycentric center reads `64,64,127,255` in BGRA.
It finally creates a 36-byte R8 constant buffer, writes RGBA at byte four and a
`[-.015625,0,0,0]` vertex offset at byte 20 through isolated standard command-9
`RESOURCE_INLINE_WRITE` submissions, reads the bytes back through
`DRM_IOCTL_VIRTGPU_TRANSFER_FROM_HOST`, emits stage-0 and stage-1 command-27
`SET_UNIFORM_BUFFER` bindings, and requires both shifted indexed triangles to
read `147,141,58,255` while the center gap is clear. Finally it creates a
Z32_FLOAT depth-stencil resource, binds it with one scanout color surface,
creates standard DSA state `DEPTH_TEST|DEPTH_WRITE|LESS`, clears color plus
depth to one, and draws a near triangle before an overlapping far triangle.
The final center must be a single source-over blend `58,102,20,255`. It then
clears once and emits half-alpha red then half-alpha green standard solid
`DRAW_VBO`s; the ordered source-over center is `0,128,64,255` in BGRA. It then clears color/depth once and draws half-alpha red near before half-alpha green far; `LESS` leaves the center at `0,0,128,255`. A final standard `DEPTH_TEST|DEPTH_WRITE|EQUAL` state clears depth to one, draws a half-alpha blue triangle at z=1, and requires `128,0,0,255` BGRA. It then repeats `EQUAL` at z=1 for half-alpha red then blue, requiring the shared-comparison batch center `128,0,64,255`, then switches `LESS` z=-.5 to `GREATER` z=.5 and requires `0,128,64,255`. Its V5 batch uses write-enabled `LESS` state 7 followed by read-only `GREATER` state 17, then schema-12 read-only `LESS` position/RGBA depth state, retaining `64,64,127,255`. Finally it uses sampled position/UV and position/RGBA/UV VBOs at z=-.5 under write-enabled `LESS` DSA 7, clears depth to one, and requires `10,20,30,255` then `32,32,64,255` while proving guest texture and modulation DSA transport. It then clears once more and emits a far half-alpha red solid followed by a near stage-0-UBO-translated gray texture × constant draw; their depth-tested `VGM1` batch retains `64,64,64,255` at the center.

The wait matters: WebBoxVM completes the guest submission only after the
browser WebGPU queue reports completion. Closing the context before that point
would invalidate the pending standard-VirGL effect.

## Build

```sh
make -C guest/virgl-clear-demo
```

The default cross toolchain is `aarch64-elf-gcc`; set `CROSS` for another
freestanding AArch64 GNU toolchain. The verifier checks ELF linkage, ABI-sized
ioctl structures, no undefined symbols, the 64 KiB artifact cap, and the
180-line maintained-file limit.

## Guest result

Inject the built program into an installed WebBoxVM Debian guest after loading
`virtio_gpu`, then run it as the DRM master on the serial console. Success is:

```text
VIRGL_TEXTURE_DEMO_PASS card0 capset=1 rings=2:ring1-clear mesh=2x-constant-uniform-triangle constant=121,115,134,255 blob=guest+host-map+default-shadow+renderer-local texture=10,20,30,255 linear=25,35,45,255 pair=55,65,75,255 vertex=64,64,127,255 modulate=32,32,64,255 uniform-inline-vertex=147,141,58,255 depth-less=58,102,20,255 solid-batch=0,128,64,255 depth-batch=0,0,128,255 depth-equal=128,0,0,255 depth-equal-batch=128,0,64,255 depth-mixed-batch=0,128,64,255 depth-write-mask-batch=0,128,64,255 depth-vertex-color=64,64,127,255 depth-texture=10,20,30,255 depth-texture-color=32,32,64,255 depth-material-constant-offset=64,64,64,255
```

That marker appears only after all guest fences resolve. The native harness
first validates the scanout upload `WBGF`, then accepts exact initial resident
candidates: `VGC1 v2` with zero predecessor; `VGB1 v6` inline/UBO singleton
and two-draw forms; and `VGM1 v2` repeat/linear texture, paired texture,
vertex-color, and texture-color forms. It completes those packets through
`complete_gpu_3d`, deliberately exercising CPU fallback and exact `WBGF`
pixel checks. This is native guest-transport evidence, not browser WebGPU
evidence and not a call to `gpu_3d_complete_resident`.

It then validates the legacy depth and batch forms: schema 9 Z32 `LESS`,
VGB1 depth, schema 10 `EQUAL`, VGB1 v3/v4/v5 comparisons, schemas 12–14
vertex/material forms, and the exact 364-byte depth-tested `VGM1` solid/
texture-constant sequence. Their expected BGRA results—including the final
`64,64,64,255` center—must all be present before PASS.

The fixed dimensions, one scanout target, one small byte buffer, and one small
off-screen copy are intentional. A mode, format, KMS, resource, or command
mismatch fails at the named stage instead of being silently treated as general
graphics compatibility.
