# WebBoxVM standard VirGL upload, clear, and readback probe

This freestanding AArch64 Linux program is a deliberately small guest-side
proof for a conservative standard capset-1 upload, clear, and readback path. It is not
Mesa, OpenGL, or Vulkan.

It opens `/dev/dri/card0`, reads the Linux `virtgpu` capset-1 response, creates
a B8G8R8A8 render-target resource, maps its backing with `DRM_IOCTL_VIRTGPU_MAP`,
writes two BGRA pixels, transfers them with `DRM_IOCTL_VIRTGPU_TRANSFER_TO_HOST`,
then views the resource through KMS's XRGB primary plane. It submits standard
VirGL `OBJECT_SURFACE`, `SET_FRAMEBUFFER_STATE`, and generic `CLEAR` commands,
waits for the resource fence, then obtains and checks two clear pixels through
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
VIRGL_TRANSFER_READBACK_DEMO_PASS card0 capset=1 upload=10,20,30,255 clear=64,128,191,255 readback=64,128,191,255
```

That marker appears only after the guest fence resolves. The native harness
first validates the two-pixel upload `WBGF`, then validates `VGC1`, captures
the clear `WBGF` at completion before the console can issue a later KMS update,
and only then accepts the marker emitted after guest-side transfer readback.

The fixed dimensions and one color target are intentional. A mode, format,
KMS, resource, or command mismatch fails at the named stage instead of being
silently treated as general graphics compatibility.
