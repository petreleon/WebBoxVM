# WebBoxVM standard VirGL buffer, copy, upload, clear, and readback probe

This freestanding AArch64 Linux program is a deliberately small guest-side
proof for a conservative standard capset-1 buffer, copy, upload, clear, and readback path. It is not
Mesa, OpenGL, or Vulkan.

It opens `/dev/dri/card0`, reads the Linux `virtgpu` capset-1 response, creates
a B8G8R8X8 render-target resource and an R8 `PIPE_BUFFER` vertex-buffer resource.
It maps the buffer backing with `DRM_IOCTL_VIRTGPU_MAP`, writes eight bytes at a
nonzero backing offset, transfers them through a standard byte-range
`DRM_IOCTL_VIRTGPU_TRANSFER_TO_HOST`, and reads them back at a different backing
offset through `DRM_IOCTL_VIRTGPU_TRANSFER_FROM_HOST`. It then maps the color
resource, writes two BGRX pixels, transfers them with `DRM_IOCTL_VIRTGPU_TRANSFER_TO_HOST`,
uploads two pixels into a four-pixel off-screen source, submits standard
VirGL `RESOURCE_COPY_REGION`, waits for the destination resource, and verifies
the destination through `DRM_IOCTL_VIRTGPU_TRANSFER_FROM_HOST`. It then views
the scanout resource through KMS's XRGB primary plane, submits standard
`OBJECT_SURFACE`, `SET_FRAMEBUFFER_STATE`, and generic `CLEAR` commands, waits
for the resource fence, then obtains and checks two clear pixels through
`DRM_IOCTL_VIRTGPU_TRANSFER_FROM_HOST` before closing the DRM file.

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
VIRGL_BUFFER_COPY_DEMO_PASS card0 capset=1 buffer=9,8,7,6,5,4,3,2 copy=10,20,30,255:40,50,60,255 clear=64,128,191,255
```

That marker appears only after the guest fence resolves. The native harness
first validates the scanout upload `WBGF`, then validates `VGC1`, captures the
clear `WBGF` at completion before the console can issue a later KMS update, and
only then accepts the marker emitted after the guest-side byte-buffer,
off-screen copy, and clear readback checks.

The fixed dimensions, one scanout target, one small byte buffer, and one small
off-screen copy are intentional. A mode, format, KMS, resource, or command
mismatch fails at the named stage instead of being silently treated as general
graphics compatibility.
