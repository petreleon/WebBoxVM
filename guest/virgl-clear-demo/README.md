# WebBoxVM standard VirGL vertex, buffer, viewport/scissor, and triangle probe

This freestanding AArch64 Linux program is a deliberately small guest-side
proof for a conservative standard capset-1 vertex, buffer, copy, upload, clear,
source-over blend, rasterizer, viewport/scissor, triangle, and readback path.
It is not Mesa, OpenGL, or Vulkan.

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
`R32G32B32A32_FLOAT` vertex buffer, uploads three fixed clip-space vertices,
and submits one standard type-1 source-over blend object, type-2 scissor
rasterizer, type-4 shader objects, type-5 vertex elements, shader binds, one
slot-zero viewport, one packed lower-left scissor, a generic clear, and one
non-indexed `DRAW_VBO`. It waits again and requires the scanout center pixel to
read back as source-over BGRA (`143,160,48,255`), while a point inside the
viewport but outside the scissor remains clear (`191,128,64,255`).

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
VIRGL_TRIANGLE_DEMO_PASS card0 capset=1 triangle=143,160,48,255
```

That marker appears only after both guest fences resolve. The native harness
first validates the scanout upload `WBGF`, then validates and completes `VGC1`,
captures its clear `WBGF`, validates the schema-2 `VGD1` sequence, viewport,
scissor, and three uploaded vertices, completes it, and requires a triangle
`WBGF` whose top-left and outside-scissor pixels are clear while its center is
the blended source-over result. It accepts the marker only after guest-side
vertex state, byte-buffer copy, color copy, clear, and draw readbacks succeed.

The fixed dimensions, one scanout target, one small byte buffer, and one small
off-screen copy are intentional. A mode, format, KMS, resource, or command
mismatch fails at the named stage instead of being silently treated as general
graphics compatibility.
