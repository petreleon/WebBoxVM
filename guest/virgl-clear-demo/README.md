# WebBoxVM standard VirGL uniform, texture, color, depth, and batch probe

This freestanding AArch64 Linux program is a deliberately small guest-side
proof for a conservative standard capset-1 vertex, buffer, copy, upload, clear,
source-over blend, rasterizer, viewport/scissor, inline-constant and bounded vertex/fragment-uniform triangles, interpolated vertex color, sampled texture, texture-modulated color, canonical depth-less and depth-equal triangles, and ordered non-depth and depth-tested solid-draw batches,
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
`DRAW_VBO`s; the ordered source-over center is `0,128,64,255` in BGRA. It then clears color/depth once and draws half-alpha red near before half-alpha green far; `LESS` leaves the center at `0,0,128,255`. A final standard `DEPTH_TEST|DEPTH_WRITE|EQUAL` state clears depth to one, draws a half-alpha blue triangle at z=1, and requires `128,0,0,255` BGRA.

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
VIRGL_TEXTURE_DEMO_PASS card0 capset=1 rings=2:ring1-clear mesh=2x-constant-uniform-triangle constant=121,115,134,255 blob=guest+host-map+default-shadow+renderer-local texture=10,20,30,255 linear=25,35,45,255 pair=55,65,75,255 vertex=64,64,127,255 modulate=32,32,64,255 uniform-inline-vertex=147,141,58,255 depth-less=58,102,20,255 solid-batch=0,128,64,255 depth-batch=0,0,128,255 depth-equal=128,0,0,255
```

That marker appears only after all guest fences resolve. The native harness
first validates the scanout upload `WBGF`, then validates and completes `VGC1`,
captures its clear `WBGF`, validates schema-2 `VGD1` with the inline color, six reordered vertices,
viewport, scissor, and its indexed two-triangle batch, then requires its `121,115,134,255` `WBGF`.
It next validates two schema-5 packets: repeat at `u == 1`, then clamp/linear at
`u == .5`, each with its position/UV VBO and normalized 2×2 BGRA snapshot from raw
RGBA. Schema 6 then verifies independent left-linear/right-repeat sampling at `[1,.625]`
and requires `55,65,75,255` at the center. Schema 7 then checks the generic
position/RGBA VBO and requires interpolated `64,64,127,255`; schema 8 then checks the 40-byte position/RGBA/UV stride, gray sampler snapshot, and modulated `32,32,64,255` center before the marker.
Finally, after the guest's two command-9 write/readback proofs, it accepts another
schema-2 packet only with the offset-four UBO color and shifted vertex positions,
then accepts schema 9 only with Z32 depth state, clear-one, and near-before-far
vertices. It requires the one-blend `58,102,20,255` center, then validates the
private `VGB1` envelope from the two standard draws and its ordered
`0,128,64,255` center, then validates VGB1 v2 with standard clear-one depth and near-before-far records, requiring the `0,0,128,255` center. It finally validates schema 10 with standard `EQUAL`, z=1 vertices, black clear, and `128,0,0,255` BGRA before PASS.

The fixed dimensions, one scanout target, one small byte buffer, and one small
off-screen copy are intentional. A mode, format, KMS, resource, or command
mismatch fails at the named stage instead of being silently treated as general
graphics compatibility.
