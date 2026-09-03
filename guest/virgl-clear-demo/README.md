# WebBoxVM standard VirGL solid and texture triangle probe

This freestanding AArch64 Linux program is a deliberately small guest-side
proof for a conservative standard capset-1 vertex, buffer, copy, upload, clear,
source-over blend, rasterizer, viewport/scissor, solid triangle, sampled texture,
and readback path. It is not Mesa, OpenGL, or Vulkan.

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
`DRM_IOCTL_VIRTGPU_TRANSFER_FROM_HOST`. Finally it creates a 48-byte
`R32G32B32A32_FLOAT` vertex buffer and eight-byte R8 index buffer, uploads three
fixed clip-space vertices plus a two-byte pad then little-endian `[2,1,0]` indices, and submits one
standard type-1 source-over blend object, type-2 scissor rasterizer, type-4
shader objects, type-5 vertex elements, shader binds, command-11 index binding at byte offset two,
one viewport/scissor, a generic clear, and one indexed `DRAW_VBO`. It waits again
and requires the scanout center pixel to read back as source-over BGRA
(`143,160,48,255`), while a point inside the viewport but outside the scissor remains clear. It then
reuses the scanout surface, binds distinct persistent-state handles, and creates
one R8G8B8A8 sampler-view texture plus a 72-byte interleaved position/UV VBO.
The type-7 nearest S/T-repeat sampler uses `u == 1`, so the type-6 identity
view must wrap to the first canonical BGRA texel. It then switches to standard
clamp/linear at `u == .5`, whose exact 2×2 midpoint must read `25,35,45,255`.
The solid pair declares 11/14 TGSI words and the texture pair 17/25, so
virglrenderer's text translator gets real capacity. It first requires
`10,20,30,255`, then `25,35,45,255` after their fences.

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
VIRGL_TEXTURE_DEMO_PASS card0 capset=1 texture=10,20,30,255 linear=25,35,45,255
```

That marker appears only after all guest fences resolve. The native harness
first validates the scanout upload `WBGF`, then validates and completes `VGC1`,
captures its clear `WBGF`, validates schema-2 `VGD1` with three reordered vertices,
viewport, scissor, and its indexed solid triangle, then requires its blended `WBGF`.
It next validates two schema-5 packets: repeat at `u == 1`, then clamp/linear at
`u == .5`, each with its position/UV VBO and normalized 2×2 BGRA snapshot from raw
RGBA. It completes them in order and requires `10,20,30,255` then `25,35,45,255` at the center.
It accepts the marker only after all guest-side buffer, copy, clear, indexed solid, and texture readbacks.

The fixed dimensions, one scanout target, one small byte buffer, and one small
off-screen copy are intentional. A mode, format, KMS, resource, or command
mismatch fails at the named stage instead of being silently treated as general
graphics compatibility.
